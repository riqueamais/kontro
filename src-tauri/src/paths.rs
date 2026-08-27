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

/// Le um arquivo de dados do app, tolerando a marca de ordem de bytes.
///
/// Quem abre o arquivo num editor, ou o reescreve por um script do PowerShell, salva com
/// o BOM na frente. O `serde_json` recusa a primeira chave e o arquivo inteiro vira
/// padrao em silencio -- perdendo tudo o que estava configurado, sem nada na tela dizendo
/// que algo deu errado.
pub fn ler(nome: &str) -> Option<String> {
    let bruto = std::fs::read_to_string(arquivo(nome)).ok()?;
    Some(bruto.trim_start_matches('\u{feff}').to_string())
}
