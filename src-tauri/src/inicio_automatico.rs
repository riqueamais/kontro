use std::path::{Path, PathBuf};

use windows::Win32::System::Registry::HKEY_CURRENT_USER;

use crate::registro;

const CHAVE: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APROVACAO: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const NOME: &str = "Kontro";

const APROVADO_PELO_WINDOWS: [u8; 12] = [0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub fn ligado() -> bool {
    linha_registrada().is_some() && aprovado_pelo_windows()
}

fn aprovado_pelo_windows() -> bool {
    aprovado(registro::bytes(HKEY_CURRENT_USER, APROVACAO, NOME).and_then(|b| b.first().copied()))
}

fn aprovado(byte0: Option<u8>) -> bool {
    byte0.is_none_or(|b| b % 2 == 0)
}

pub fn conferir() {
    let Ok(atual) = std::env::current_exe() else { return };
    let Some(registrado) = executavel_registrado() else { return };

    if mesmo_arquivo(&registrado, &atual) {
        return;
    }

    if !registrado.exists() || mesma_pasta(&registrado, &atual) {
        gravar_linha();
    }
}

pub fn definir(ligar: bool) -> bool {
    if !ligar {
        return registro::apagar(HKEY_CURRENT_USER, CHAVE, NOME) || !ligado();
    }

    if !gravar_linha() {
        return false;
    }
    aprovado_pelo_windows()
        || registro::gravar_bytes(HKEY_CURRENT_USER, APROVACAO, NOME, &APROVADO_PELO_WINDOWS)
}

fn gravar_linha() -> bool {
    match linha_de_comando() {
        Some(comando) => registro::gravar_texto(HKEY_CURRENT_USER, CHAVE, NOME, &comando),
        None => false,
    }
}

fn linha_registrada() -> Option<String> {
    registro::texto(HKEY_CURRENT_USER, CHAVE, NOME)
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
    fn o_veredito_do_windows_e_o_primeiro_byte() {
        assert!(aprovado(None));
        assert!(aprovado(Some(0x02)));
        assert!(aprovado(Some(0x06)));
        assert!(!aprovado(Some(0x03)));
        assert!(!aprovado(Some(0x07)));
    }

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
}
