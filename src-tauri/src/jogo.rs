use std::path::{Path, PathBuf};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::tela;

const FORA: &[&str] = &[
    "kontro.exe",
    "explorer.exe",
    "applicationframehost.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "textinputhost.exe",
    "lockapp.exe",
    "dwm.exe",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jogo {
    pub caminho: PathBuf,
    pub nome: String,
}

pub fn em_foco() -> Option<Jogo> {
    if !tela::em_tela_cheia() {
        return None;
    }
    de(&executavel_em_foco()?)
}

pub fn de(caminho: &Path) -> Option<Jogo> {
    if ignorado(caminho) {
        return None;
    }
    Some(Jogo { nome: batizar(caminho), caminho: caminho.to_path_buf() })
}

pub fn ignorado(caminho: &Path) -> bool {
    let Some(arquivo) = caminho.file_name().map(|n| n.to_string_lossy().to_lowercase()) else {
        return true;
    };
    FORA.contains(&arquivo.as_str())
}

pub fn batizar(caminho: &Path) -> String {
    descricao(caminho).filter(|d| aceitavel(d)).unwrap_or_else(|| pelo_arquivo(caminho))
}

fn aceitavel(descricao: &str) -> bool {
    let limpo = descricao.trim();
    !limpo.is_empty() && limpo.len() <= 64 && !limpo.eq_ignore_ascii_case("application")
}

fn pelo_arquivo(caminho: &Path) -> String {
    let bruto = caminho.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    let palavras: Vec<String> = bruto
        .split(|c: char| c == '-' || c == '_' || c == '.')
        .filter(|p| !p.is_empty())
        .map(maiuscula_no_comeco)
        .collect();

    if palavras.is_empty() {
        "Jogo".to_string()
    } else {
        palavras.join(" ")
    }
}

fn maiuscula_no_comeco(palavra: &str) -> String {
    let mut letras = palavra.chars();
    match letras.next() {
        Some(primeira) => primeira.to_uppercase().collect::<String>() + letras.as_str(),
        None => String::new(),
    }
}

fn executavel_em_foco() -> Option<PathBuf> {
    unsafe {
        let janela = GetForegroundWindow();
        if janela.is_invalid() {
            return None;
        }

        let mut processo = 0u32;
        GetWindowThreadProcessId(janela, Some(&mut processo));
        if processo == 0 {
            return None;
        }

        let alca = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, processo).ok()?;

        let mut espaco = [0u16; 512];
        let mut quantos = espaco.len() as u32;
        let leu = QueryFullProcessImageNameW(
            alca,
            PROCESS_NAME_WIN32,
            PWSTR(espaco.as_mut_ptr()),
            &mut quantos,
        );

        let _ = CloseHandle(alca);
        leu.ok()?;

        let texto = String::from_utf16_lossy(&espaco[..quantos as usize]);
        (!texto.is_empty()).then(|| PathBuf::from(texto))
    }
}

fn descricao(caminho: &Path) -> Option<String> {
    let arquivo: Vec<u16> =
        caminho.as_os_str().to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    let arquivo = PCWSTR(arquivo.as_ptr());

    unsafe {
        let tamanho = GetFileVersionInfoSizeW(arquivo, None);
        if tamanho == 0 {
            return None;
        }

        let mut bloco = vec![0u8; tamanho as usize];
        GetFileVersionInfoW(arquivo, None, tamanho, bloco.as_mut_ptr() as *mut _).ok()?;

        let idioma = primeira_traducao(&bloco)?;
        let chave: Vec<u16> = format!("\\StringFileInfo\\{idioma}\\FileDescription")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut texto: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut letras = 0u32;
        if !VerQueryValueW(
            bloco.as_ptr() as *const _,
            PCWSTR(chave.as_ptr()),
            &mut texto,
            &mut letras,
        )
        .as_bool()
            || texto.is_null()
            || letras == 0
        {
            return None;
        }

        let fatia = std::slice::from_raw_parts(texto as *const u16, letras as usize);
        Some(String::from_utf16_lossy(fatia).trim_end_matches('\0').trim().to_string())
    }
}

fn primeira_traducao(bloco: &[u8]) -> Option<String> {
    let chave: Vec<u16> =
        "\\VarFileInfo\\Translation".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut dados: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut bytes = 0u32;
        if !VerQueryValueW(
            bloco.as_ptr() as *const _,
            PCWSTR(chave.as_ptr()),
            &mut dados,
            &mut bytes,
        )
        .as_bool()
            || dados.is_null()
            || bytes < 4
        {
            return None;
        }

        let par = std::slice::from_raw_parts(dados as *const u16, 2);
        Some(format!("{:04x}{:04x}", par[0], par[1]))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_proprio_kontro_nunca_conta_como_jogo() {
        assert!(ignorado(Path::new(r"D:\Kontro\kontro.exe")));
        assert!(ignorado(Path::new(r"C:\Windows\explorer.exe")));
        assert!(!ignorado(Path::new(r"D:\Games\eldenring.exe")));
    }

    #[test]
    fn sem_descricao_o_nome_sai_do_arquivo() {
        assert_eq!(pelo_arquivo(Path::new(r"D:\Games\eldenring.exe")), "Eldenring");
        assert_eq!(pelo_arquivo(Path::new(r"D:\Games\hollow-knight.exe")), "Hollow Knight");
        assert_eq!(pelo_arquivo(Path::new(r"D:\Games\dark_souls_3.exe")), "Dark Souls 3");
    }

    #[test]
    fn caminho_sem_nome_ainda_devolve_alguma_coisa() {
        assert_eq!(pelo_arquivo(Path::new("")), "Jogo");
    }

    #[test]
    fn descricao_generica_demais_nao_vira_nome() {
        assert!(!aceitavel(""));
        assert!(!aceitavel("   "));
        assert!(!aceitavel("Application"));
        assert!(aceitavel("ELDEN RING"));
    }

    #[test]
    fn o_recurso_de_versao_de_um_exe_de_verdade_e_lido() {
        let conhecido = Path::new(r"C:\Windows\System32\notepad.exe");
        if !conhecido.exists() {
            return;
        }
        let lida = descricao(conhecido);
        assert!(lida.is_some(), "nao consegui ler o FileDescription do notepad");
        assert!(!lida.unwrap().trim().is_empty());
    }

    #[test]
    fn descricao_gigante_e_recusada() {
        assert!(!aceitavel(&"a".repeat(80)));
    }
}
