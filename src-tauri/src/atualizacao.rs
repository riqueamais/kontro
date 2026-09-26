use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::caminhos;
use crate::tempo;

pub const JANELA_MS: i64 = 24 * 60 * 60 * 1000;

const CANAL_ESTAVEL: &str =
    "https://github.com/riqueamais/kontro/releases/latest/download/latest.json";
const CANAL_BETA: &str = "https://github.com/riqueamais/kontro/releases/download/beta/beta.json";

#[derive(Debug, Clone)]
pub struct Novidade {
    pub versao: String,
    pub notas: Option<String>,
    pub beta: bool,
}

pub enum Consulta {
    Nova(Novidade, Update),
    EmDia,
    Falhou(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Marca {
    ultima_checagem_ms: i64,
}

#[derive(Serialize, Clone)]
#[serde(tag = "etapa", rename_all = "camelCase")]
pub enum Andamento {
    Baixando { bytes: u64, total: Option<u64> },
    Instalando,
}

pub fn procurar_bloqueando(app: &AppHandle, beta: bool) -> Consulta {
    tauri::async_runtime::block_on(procurar(app, beta))
}

pub async fn procurar(app: &AppHandle, beta: bool) -> Consulta {
    let mut canais = vec![CANAL_ESTAVEL];
    if beta {
        canais.push(CANAL_BETA);
    }

    let mut melhor: Option<Update> = None;
    let mut falha = None;
    for canal in canais {
        match consultar(app, canal).await {
            Ok(Some(achado)) => {
                let supera = melhor.as_ref().map(|m| mais_nova(&achado.version, &m.version));
                if supera.unwrap_or(true) {
                    melhor = Some(achado);
                }
            }
            Ok(None) => {}
            Err(e) if canal == CANAL_BETA && sem_release(&e) => {}
            Err(e) => falha = Some(descrever(&e)),
        }
    }

    match (melhor, falha) {
        (Some(u), _) => {
            marcar_checagem();
            let novidade = Novidade {
                versao: u.version.clone(),
                notas: u.body.clone(),
                beta: e_beta(&u.version),
            };
            Consulta::Nova(novidade, u)
        }
        (None, Some(motivo)) => Consulta::Falhou(motivo),
        (None, None) => {
            marcar_checagem();
            Consulta::EmDia
        }
    }
}

async fn consultar(
    app: &AppHandle,
    canal: &str,
) -> Result<Option<Update>, tauri_plugin_updater::Error> {
    let url = canal.parse().map_err(tauri_plugin_updater::Error::UrlParse)?;
    app.updater_builder().endpoints(vec![url])?.build()?.check().await
}

pub async fn instalar(app: &AppHandle, atualizacao: Update) -> Result<(), String> {
    let mut baixado: u64 = 0;
    let progresso = app.clone();
    let fim = app.clone();
    atualizacao
        .download_and_install(
            move |pedaco, total| {
                baixado += pedaco as u64;
                let _ = progresso
                    .emit("kontro://atualizacao", Andamento::Baixando { bytes: baixado, total });
            },
            move || {
                let _ = fim.emit("kontro://atualizacao", Andamento::Instalando);
            },
        )
        .await
        .map_err(|e| descrever(&e))
}

fn mais_nova(a: &str, b: &str) -> bool {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => a > b,
        _ => false,
    }
}

pub fn e_beta(versao: &str) -> bool {
    semver::Version::parse(versao).map(|v| !v.pre.is_empty()).unwrap_or(false)
}

fn sem_release(erro: &tauri_plugin_updater::Error) -> bool {
    matches!(erro, tauri_plugin_updater::Error::ReleaseNotFound)
}

fn descrever(erro: &tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;

    match erro {
        Error::Reqwest(_) | Error::Io(_) | Error::Network(_) => {
            "não foi possível falar com o GitHub".into()
        }
        Error::ReleaseNotFound => "o GitHub respondeu sem uma release legível".into(),
        Error::Serialization(_) | Error::Semver(_) => "a resposta do GitHub veio ilegível".into(),
        Error::TargetNotFound(_) | Error::TargetsNotFound(_) => {
            "a release publicada não traz pacote para este sistema".into()
        }
        Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_) => {
            "a assinatura do pacote não confere".into()
        }
        outro => outro.to_string(),
    }
}

pub fn ultima_checagem() -> i64 {
    caminhos::ler("atualizacao.json")
        .and_then(|t| serde_json::from_str::<Marca>(&t).ok())
        .map(|m| m.ultima_checagem_ms)
        .unwrap_or(0)
}

fn marcar_checagem() {
    caminhos::garantir_dir();
    let marca = Marca { ultima_checagem_ms: tempo::agora() };
    if let Ok(t) = serde_json::to_string_pretty(&marca) {
        let _ = std::fs::write(caminhos::arquivo("atualizacao.json"), t);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn a_beta_da_proxima_versao_e_mais_nova_que_a_estavel_atual() {
        assert!(mais_nova("2.14.0-beta.3", "2.13.4"));
    }

    #[test]
    fn a_estavel_supera_a_propria_beta() {
        assert!(mais_nova("2.14.0", "2.14.0-beta.9"));
        assert!(!mais_nova("2.14.0-beta.9", "2.14.0"));
    }

    #[test]
    fn versao_com_sufixo_e_beta() {
        assert!(e_beta("2.14.0-beta.1"));
        assert!(!e_beta("2.14.0"));
        assert!(!e_beta("lixo"));
    }
}
