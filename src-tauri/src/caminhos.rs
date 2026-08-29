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

pub fn ler(nome: &str) -> Option<String> {
    let bruto = std::fs::read_to_string(arquivo(nome)).ok()?;
    Some(bruto.trim_start_matches('\u{feff}').to_string())
}
