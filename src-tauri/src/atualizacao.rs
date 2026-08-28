use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::paths;
use crate::tempo;

pub const JANELA_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone)]
pub struct Novidade {
    pub versao: String,
    pub notas: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Consulta {
    Nova(Novidade),
    EmDia,
    Falhou(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Marca {
    ultima_checagem_ms: i64,
}

pub fn procurar(app: &AppHandle) -> Consulta {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => return Consulta::Falhou(descrever(&e)),
    };

    match tauri::async_runtime::block_on(updater.check()) {
        Ok(achado) => {
            marcar_checagem();
            match achado {
                Some(u) => Consulta::Nova(Novidade { versao: u.version, notas: u.body }),
                None => Consulta::EmDia,
            }
        }
        Err(e) => Consulta::Falhou(descrever(&e)),
    }
}

fn descrever(erro: &tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;

    match erro {
        Error::Reqwest(_) | Error::Io(_) => "não foi possível falar com o GitHub".into(),
        Error::ReleaseNotFound => "o GitHub respondeu sem uma release legível".into(),
        Error::Serialization(_) | Error::Semver(_) => {
            "a resposta do GitHub veio ilegível".into()
        }
        Error::TargetNotFound(_) | Error::TargetsNotFound(_) => {
            "a release publicada não traz pacote para este sistema".into()
        }
        outro => outro.to_string(),
    }
}

pub fn ultima_checagem() -> i64 {
    paths::ler("atualizacao.json")
        .and_then(|t| serde_json::from_str::<Marca>(&t).ok())
        .map(|m| m.ultima_checagem_ms)
        .unwrap_or(0)
}

fn marcar_checagem() {
    paths::garantir_dir();
    let marca = Marca { ultima_checagem_ms: tempo::agora() };
    if let Ok(t) = serde_json::to_string_pretty(&marca) {
        let _ = std::fs::write(paths::arquivo("atualizacao.json"), t);
    }
}
