//! Saber que saiu versao nova.
//!
//! Nao e atualizacao silenciosa: isso exigiria assinar cada pacote e guardar uma chave
//! privada, decisao que nao se toma de passagem. O que da para fazer com honestidade e
//! avisar -- e avisar pela notificacao do sistema, que fica na Central de Acoes para ser
//! lida depois, em vez do balao antigo que pisca e some.

use serde::Deserialize;

const REPOSITORIO: &str = "riqueamais/kontro";

/// De quanto em quanto tempo perguntar.
///
/// A consulta e uma requisicao pequena. Vinte e quatro horas, como era antes, significa
/// saber de uma release quase um dia depois dela sair.
pub const JANELA_MS: i64 = 3 * 60 * 60 * 1000;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

#[derive(Debug, Clone)]
pub struct Novidade {
    pub versao: String,
    pub pagina: String,
}

/// Consulta a ultima release publicada. Nulo quando ja estamos nela ou a rede falhou.
pub fn procurar() -> Option<Novidade> {
    let url = format!("https://api.github.com/repos/{REPOSITORIO}/releases/latest");

    let resposta = ureq::get(&url)
        .header("User-Agent", "Kontro")
        .header("Accept", "application/vnd.github+json")
        .call()
        .ok()?;

    let release: Release = resposta.into_body().read_json().ok()?;
    let remota = release.tag_name.trim_start_matches(['v', 'V']).to_string();

    mais_nova(&remota, env!("CARGO_PKG_VERSION")).then(|| Novidade {
        versao: remota,
        pagina: release.html_url,
    })
}

/// Compara so a parte numerica; um sufixo de pre-lancamento perde do lancamento final.
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
    // mesma versao numerica: o lancamento final ganha do pre-lancamento
    b_pre && !a_pre
}

#[cfg(test)]
mod testes {
    use super::mais_nova;

    #[test]
    fn compara_versoes() {
        assert!(mais_nova("2.1.0", "2.0.1"));
        assert!(mais_nova("2.0.2", "2.0.1"));
        assert!(!mais_nova("2.0.1", "2.0.1"));
        assert!(!mais_nova("1.9.9", "2.0.0"));
        // o final ganha do pre-lancamento de mesmo numero
        assert!(mais_nova("2.0.1", "2.0.1-beta"));
        assert!(!mais_nova("2.0.1-beta", "2.0.1"));
    }
}
