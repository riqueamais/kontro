use std::path::{Path, PathBuf};

use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const CHAVE: windows::core::PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const NOME: windows::core::PCWSTR = w!("Kontro");

pub fn ligado() -> bool {
    linha_registrada().is_some()
}

pub fn conferir() {
    let Ok(atual) = std::env::current_exe() else { return };
    let Some(registrado) = executavel_registrado() else { return };

    if mesmo_arquivo(&registrado, &atual) {
        return;
    }

    if !registrado.exists() || mesma_pasta(&registrado, &atual) {
        definir(true);
    }
}

pub fn definir(ligar: bool) -> bool {
    let Some(chave) = abrir(KEY_WRITE.0) else { return false };

    let ok = if ligar {
        match linha_de_comando() {
            Some(comando) => {
                let mut unidades: Vec<u16> = comando.encode_utf16().collect();
                unidades.push(0);
                let bytes: Vec<u8> = unidades.iter().flat_map(|u| u.to_le_bytes()).collect();
                unsafe { RegSetValueExW(chave, NOME, None, REG_SZ, Some(&bytes)).is_ok() }
            }
            None => false,
        }
    } else {
        unsafe { RegDeleteValueW(chave, NOME).is_ok() || !ligado() }
    };

    unsafe {
        let _ = RegCloseKey(chave);
    }
    ok
}

fn abrir(acesso: u32) -> Option<HKEY> {
    let mut chave = HKEY::default();
    let ok = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            CHAVE,
            None,
            windows::Win32::System::Registry::REG_SAM_FLAGS(acesso),
            &mut chave,
        )
    };
    ok.is_ok().then_some(chave)
}

fn linha_registrada() -> Option<String> {
    let chave = abrir(KEY_READ.0)?;

    let mut tamanho = 0u32;
    let medida = unsafe { RegQueryValueExW(chave, NOME, None, None, None, Some(&mut tamanho)) };

    let linha = if medida.is_ok() && tamanho > 0 {
        let mut bytes = vec![0u8; tamanho as usize];
        let leitura = unsafe {
            RegQueryValueExW(chave, NOME, None, None, Some(bytes.as_mut_ptr()), Some(&mut tamanho))
        };
        let lidos = (tamanho as usize).min(bytes.len());
        leitura.is_ok().then(|| decodificar(&bytes[..lidos]))
    } else {
        None
    };

    unsafe {
        let _ = RegCloseKey(chave);
    }
    linha
}

fn decodificar(bytes: &[u8]) -> String {
    let unidades: Vec<u16> =
        bytes.chunks_exact(2).map(|par| u16::from_le_bytes([par[0], par[1]])).collect();
    String::from_utf16_lossy(&unidades).trim_end_matches('\0').to_string()
}

fn executavel_registrado() -> Option<PathBuf> {
    caminho_da_linha(&linha_registrada()?)
}

fn caminho_da_linha(linha: &str) -> Option<PathBuf> {
    let linha = linha.trim();

    let caminho = match linha.strip_prefix('"') {
        Some(resto) => resto.split('"').next()?,
        None => linha.split_whitespace().next()?,
    };

    (!caminho.is_empty()).then(|| PathBuf::from(caminho))
}

fn mesmo_arquivo(um: &Path, outro: &Path) -> bool {
    let real = |p: &Path| {
        p.canonicalize().unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().to_lowercase()
    };
    real(um) == real(outro)
}

fn mesma_pasta(um: &Path, outro: &Path) -> bool {
    match (um.parent(), outro.parent()) {
        (Some(a), Some(b)) => {
            a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
        }
        _ => false,
    }
}

fn linha_de_comando() -> Option<String> {
    let caminho = std::env::current_exe().ok()?;
    Some(format!("\"{}\" --minimizado", caminho.display()))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn a_linha_com_aspas_devolve_so_o_executavel() {
        let linha = r#""D:\Kontro\kontro.exe" --minimizado"#;
        assert_eq!(caminho_da_linha(linha), Some(PathBuf::from(r"D:\Kontro\kontro.exe")));
    }

    #[test]
    fn a_linha_sem_aspas_para_no_primeiro_espaco() {
        assert_eq!(
            caminho_da_linha(r"D:\Kontro\kontro.exe --minimizado"),
            Some(PathBuf::from(r"D:\Kontro\kontro.exe"))
        );
    }

    #[test]
    fn a_linha_vazia_nao_vira_caminho() {
        assert_eq!(caminho_da_linha("   "), None);
        assert_eq!(caminho_da_linha(r#""" --minimizado"#), None);
    }

    #[test]
    fn o_binario_renomeado_mora_na_mesma_pasta() {
        let velho = PathBuf::from(r"D:\Kontro\kontro-bandeja.exe");
        let novo = PathBuf::from(r"D:\kontro\kontro.exe");
        assert!(mesma_pasta(&velho, &novo));
        assert!(!mesmo_arquivo(&velho, &novo));
    }

    #[test]
    fn uma_build_de_desenvolvimento_mora_em_outra_pasta() {
        let instalado = PathBuf::from(r"D:\Kontro\kontro.exe");
        let feito_agora = PathBuf::from(r"C:\kontro\src-tauri\target\debug\kontro.exe");
        assert!(!mesma_pasta(&instalado, &feito_agora));
    }

    #[test]
    fn o_texto_do_registro_perde_o_zero_do_fim() {
        let bytes: Vec<u8> = "oi\0".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        assert_eq!(decodificar(&bytes), "oi");
    }
}
