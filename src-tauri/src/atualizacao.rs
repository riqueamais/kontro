use serde::{Deserialize, Serialize};

use crate::paths;
use crate::tempo;

const REPOSITORIO: &str = "riqueamais/kontro";

const MANIFESTO: &str =
    "https://github.com/riqueamais/kontro/releases/latest/download/latest.json";

pub const JANELA_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Deserialize)]
struct Manifesto {
    version: String,
}

#[derive(Debug, Clone)]
pub struct Novidade {
    pub versao: String,
    pub pagina: String,
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

pub fn procurar() -> Consulta {
    let resposta = match ureq::get(MANIFESTO)
        .header("User-Agent", "Kontro")
        .header("Accept", "application/json")
        .call()
    {
        Ok(r) => r,
        Err(e) => return Consulta::Falhou(descrever(&e)),
    };

    let manifesto: Manifesto = match resposta.into_body().read_json() {
        Ok(m) => m,
        Err(_) => return Consulta::Falhou("a resposta do GitHub veio ilegível".into()),
    };

    let remota = manifesto.version.trim_start_matches(['v', 'V']).to_string();
    if remota.is_empty() {
        return Consulta::Falhou("a resposta do GitHub veio sem versão".into());
    }

    marcar_checagem();

    if mais_nova(&remota, env!("CARGO_PKG_VERSION")) {
        Consulta::Nova(Novidade {
            pagina: format!("https://github.com/{REPOSITORIO}/releases/tag/v{remota}"),
            versao: remota,
        })
    } else {
        Consulta::EmDia
    }
}

fn descrever(erro: &ureq::Error) -> String {
    match erro {
        ureq::Error::StatusCode(codigo) => format!("o GitHub respondeu {codigo}"),
        _ => "não foi possível falar com o GitHub".into(),
    }
}

pub fn ultima_checagem() -> i64 {
    std::fs::read_to_string(paths::arquivo("atualizacao.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Marca>(&t).ok())
        .map(|m| m.ultima_checagem_ms)
        .unwrap_or(0)
}

fn marcar_checagem() {
    paths::garantir_dir();
    let marca = Marca {
        ultima_checagem_ms: tempo::agora(),
    };
    if let Ok(t) = serde_json::to_string_pretty(&marca) {
        let _ = std::fs::write(paths::arquivo("atualizacao.json"), t);
    }
}

pub fn mais_nova(candidata: &str, atual: &str) -> bool {
    fn partes(v: &str) -> ([u32; 3], bool) {
        let pre = v.contains('-') || v.contains('+');
        let numerica = v.split(['-', '+']).next().unwrap_or("");
        let mut saida = [0u32; 3];
        for (i, p) in numerica.split('.').take(3).enumerate() {
            saida[i] = p.parse().unwrap_or(0);
        }
        (saida, pre)
    }

    let (a, a_pre) = partes(candidata);
    let (b, b_pre) = partes(atual);

    if a != b {
        return a > b;
    }
    b_pre && !a_pre
}

#[cfg(test)]
mod testes {
    use super::mais_nova;

    #[test]
    fn compara_versoes() {
        assert!(mais_nova("2.1.0", "2.0.1"));
        assert!(mais_nova("2.0.2", "2.0.1"));
        assert!(mais_nova("2.2.0", "2.1.6"));
        assert!(!mais_nova("2.0.1", "2.0.1"));
        assert!(!mais_nova("1.9.9", "2.0.0"));
        assert!(mais_nova("2.0.1", "2.0.1-beta"));
        assert!(!mais_nova("2.0.1-beta", "2.0.1"));
    }
}
