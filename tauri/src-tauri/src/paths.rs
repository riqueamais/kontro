//! Onde o app guarda o que e do usuario.
//!
//! Fica em Roaming e nao em Local por um motivo aprendido na marra: a pasta de
//! instalacao vive em Local, e dado do usuario debaixo dela some quando o app se
//! atualiza. Os nomes de arquivo sao os mesmos da versao anterior, entao configuracao
//! e historico atravessam a migracao sem o usuario perceber.

use std::path::PathBuf;

pub fn dir_de_dados() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("Kontro")
}

pub fn arquivo(nome: &str) -> PathBuf {
    dir_de_dados().join(nome)
}

pub fn garantir_dir() {
    let _ = std::fs::create_dir_all(dir_de_dados());
}
