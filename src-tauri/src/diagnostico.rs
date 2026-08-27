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
    secao_vigia(&mut t);
    secao_monitor(&mut t);

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

/// O vigia esta de pe?
///
/// E ele que manda a varredura acontecer no instante em que o Windows publica ou retira a
/// interface do controle. Com ele mudo, o app so percebe a mudanca no relogio de
/// seguranca, e volta a demorar para notar que o controle saiu.
fn secao_vigia(t: &mut String) {
    let _ = writeln!(t, "=== Vigia de dispositivos ===");

    let (aviso, avisos) = std::sync::mpsc::channel();
    let vigia = crate::device::vigia::observar(aviso);
    let _ = writeln!(t, "   observadores de pe: {} (esperado: 4)", vigia.quantos());

    // A enumeracao inicial anuncia cada dispositivo que ja esta ligado. Nenhum aviso aqui
    // com controle ligado significa que os eventos nao estao chegando.
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = writeln!(t, "   avisos na enumeracao inicial: {}", avisos.try_iter().count());
    let _ = writeln!(t);
}

/// O que o monitor conclui, com as regras que valem em producao.
///
/// As secoes acima mostram o que cada via responde. Esta mostra o que o app faz com
/// isso: qual via ele escolheu, se a leitura conta como de agora e de que hora ela e.
/// Sem ela, um relato de "mostra a carga errada" nao tinha como distinguir uma fonte que
/// mente de uma regra que escolheu mal.
///
/// O ciclo roda pelo tempo real de confirmacao: a primeira leitura de uma conexao fica
/// em observacao, e parar antes disso mostraria toda leitura como ainda nao confirmada.
fn secao_monitor(t: &mut String) {
    use crate::model::LinkMode;

    let _ = writeln!(t, "=== O que o monitor conclui ===");

    let mut monitor = crate::monitor::Monitor::novo();
    let mut panorama = None;
    let comeco = crate::tempo::agora();

    while crate::tempo::agora() - comeco < 15_000 {
        if let Some(p) = monitor.ciclo() {
            panorama = Some(p);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    let Some(panorama) = panorama else {
        let _ = writeln!(t, "   (o ciclo nao chegou a produzir estado)");
        return;
    };

    for estado in &panorama.todos {
        let principal = if estado.key == panorama.principal.key { "  <- principal" } else { "" };
        let _ = writeln!(t, "   {}{principal}", estado.device_name);
        let _ = writeln!(t, "      chave={}", estado.key);
        let _ = writeln!(
            t,
            "      via={:?}   carga={}   precisao={:?}",
            estado.mode, estado.texto_da_carga, estado.precisao
        );
        let quando = match estado.read_at {
            Some(ms) => crate::tempo::para_texto(ms),
            None => "nunca".to_string(),
        };
        let _ = writeln!(
            t,
            "      lido em {quando}   {}",
            if estado.stale { "(ultima conhecida)" } else { "(ao vivo)" }
        );
        if estado.mode == LinkMode::Cable {
            let _ = writeln!(t, "      carregando={}", estado.charging);
        }
        if let Some(a) = &estado.autonomia {
            let _ = writeln!(t, "      autonomia: {a}");
        }
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
