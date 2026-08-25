//! Um retrato do que este computador oferece sobre os controles ligados.
//!
//! Existe porque a leitura principal do app depende do Bluetooth, e controle ligado por
//! dongle de radio nao aparece como dispositivo Bluetooth. Quando alguem diz "nao mostra
//! a bateria", este arquivo responde onde a carga esta -- ou prova que ela nao esta em
//! lugar nenhum, que tambem e resposta.

use std::fmt::Write;

use crate::device::{discovery, gatt, hid, pnp, xinput};

pub fn escrever(caminho: &str) -> std::io::Result<()> {
    let mut t = String::new();

    let _ = writeln!(t, "Kontro {} - diagnostico", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(t, "{}", "=".repeat(60));
    let _ = writeln!(t);

    secao_versao(&mut t);
    secao_xinput(&mut t);
    secao_bluetooth(&mut t);
    secao_pnp(&mut t);
    secao_hid(&mut t);
    secao_descoberta(&mut t);

    std::fs::write(caminho, t)
}

/// A consulta de versao entra aqui porque e onde ela e verificavel.
///
/// O executavel e do subsistema grafico e nao tem console: imprimir na tela nao chega a
/// lugar nenhum. Num arquivo, o resultado sobrevive -- e de quebra vem junto quando
/// alguem manda o diagnostico.
fn secao_versao(t: &mut String) {
    let _ = writeln!(t, "=== Versao ===");
    let _ = writeln!(t, "   instalada: {}", env!("CARGO_PKG_VERSION"));
    match crate::atualizacao::procurar() {
        crate::atualizacao::Consulta::Nova(n) => {
            let _ = writeln!(t, "   publicada: {}  (mais nova)", n.versao);
            let _ = writeln!(t, "   {}", n.pagina);
        }
        crate::atualizacao::Consulta::EmDia => {
            let _ = writeln!(t, "   publicada: nenhuma mais nova");
        }
        crate::atualizacao::Consulta::Falhou(motivo) => {
            let _ = writeln!(t, "   publicada: nao consegui verificar -- {motivo}");
        }
    }
    let _ = writeln!(t);
}

fn secao_xinput(t: &mut String) {
    let _ = writeln!(t, "=== XInput (cabo, adaptador sem fio e dongle) ===");
    let linhas = xinput::descrever();
    if linhas.is_empty() {
        let _ = writeln!(t, "   (nenhum controle visivel ao XInput)");
    } else {
        for l in linhas {
            let _ = writeln!(t, "   {l}");
        }
    }
    let _ = writeln!(t, "   slots ocupados: {:?}", xinput::slots_conectados());
    let _ = writeln!(t, "   algum no cabo : {}", xinput::alguem_no_cabo());
    let _ = writeln!(t, "   algum na bateria: {}", xinput::alguem_na_bateria());
    let _ = writeln!(t);
}

fn secao_bluetooth(t: &mut String) {
    let _ = writeln!(t, "=== Bluetooth LE pareados ===");
    let pareados = gatt::pareados();
    if pareados.is_empty() {
        let _ = writeln!(t, "   (nenhum dispositivo pareado)");
    }
    for (endereco, nome) in pareados {
        let estado = if gatt::conectado(endereco) { "conectado" } else { "desconectado" };
        let _ = writeln!(t, "   {nome}   [{endereco:012x}]   {estado}");
    }
    let _ = writeln!(t);
}

fn secao_pnp(t: &mut String) {
    let _ = writeln!(t, "=== Carga que o Windows guarda, por dispositivo ===");
    let _ = writeln!(t, "   A data importa: valor anterior a ligacao atual e de outra sessao.");
    let nos = pnp::nos_com_bateria();
    if nos.is_empty() {
        let _ = writeln!(t, "   (nenhum dispositivo expoe carga por esta via)");
    }
    for no in nos {
        let quando = match no.medido_em {
            Some(ms) => crate::tempo::para_texto(ms),
            None => "sem data".to_string(),
        };
        let _ = writeln!(t, "   {}: {}%   ({quando})", no.nome, no.percent);
        let _ = writeln!(t, "      instancia={}", no.instancia);
        let _ = writeln!(t, "      container={}", no.container);
    }
    let _ = writeln!(t);
}

fn secao_hid(t: &mut String) {
    let _ = writeln!(t, "=== HID: carga informada pelo proprio dispositivo ===");
    let _ = writeln!(t, "   E por aqui que muitos controles de dongle publicam a bateria.");

    let controles = discovery::descobrir();
    let com_hid: Vec<_> = controles.iter().filter(|c| !c.id_hid.is_empty()).collect();
    if com_hid.is_empty() {
        let _ = writeln!(t, "   (nenhum controle HID presente)");
    }
    for c in com_hid {
        let leitura = hid::ler(&c.id_hid);
        let resposta = if leitura.tem() {
            format!("{}%", leitura.valor)
        } else {
            "sem controle de carga".to_string()
        };
        let _ = writeln!(t, "   {}: {resposta}", c.nome);
    }
    let _ = writeln!(t);
}

fn secao_descoberta(t: &mut String) {
    let _ = writeln!(t, "=== Resultado final da descoberta (o que o app mostra) ===");
    let controles = discovery::descobrir();
    if controles.is_empty() {
        let _ = writeln!(t, "   (nenhum controle ligado agora)");
    }
    for c in controles {
        let via = if c.endereco != 0 {
            "Bluetooth".to_string()
        } else if c.slot_xinput >= 0 {
            format!("somente XInput, slot {}", c.slot_xinput)
        } else {
            "HID".to_string()
        };
        let _ = writeln!(t, "   {}   [{}]", c.nome, c.endereco_bonito().unwrap_or(via.clone()));
        let _ = writeln!(t, "      chave={}   via={via}", c.chave());
        if !c.container.is_empty() {
            let _ = writeln!(t, "      container={}", c.container);
        }
        if !c.id_hid.is_empty() {
            let _ = writeln!(t, "      hid={}", c.id_hid);
        }
    }
}
