use std::fmt::Write;

use crate::dispositivo::{descoberta, gatt, hid, pnp, xinput};
use crate::historico::Sessao;
use crate::modelo::EstadoDoControle;

pub struct AoVivo<'a> {
    pub principal: &'a str,
    pub estados: &'a [EstadoDoControle],
    pub sessoes: &'a [Sessao],
    pub pilula: Option<isize>,
}

pub fn escrever(caminho: &str, ao_vivo: Option<AoVivo>) -> std::io::Result<()> {
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
    secao_tela(&mut t, ao_vivo.as_ref().and_then(|v| v.pilula));
    match ao_vivo {
        Some(v) => secao_do_app(&mut t, &v),
        None => {
            secao_vigia(&mut t);
            secao_monitor(&mut t);
        }
    }

    std::fs::write(caminho, t)
}

fn secao_versao(t: &mut String) {
    let _ = writeln!(t, "=== Versao ===");
    let _ = writeln!(t, "   instalada: {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(t, "   publicadas em github.com/riqueamais/kontro/releases");
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
        let _ = writeln!(t, "   {}: {}%   ({quando})", no.nome, no.percentual);
        let _ = writeln!(t, "      instancia={}", no.instancia);
        let _ = writeln!(t, "      container={}", no.container);
    }
    let _ = writeln!(t);
}

fn secao_hid(t: &mut String) {
    let _ = writeln!(t, "=== HID: carga informada pelo proprio dispositivo ===");
    let _ = writeln!(t, "   E por aqui que muitos controles de dongle publicam a bateria.");

    let controles = descoberta::descobrir();
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

fn secao_vigia(t: &mut String) {
    let _ = writeln!(t, "=== Vigia de dispositivos ===");

    let (aviso, avisos) = std::sync::mpsc::channel();
    let vigia = crate::dispositivo::vigia::observar(aviso);
    let _ = writeln!(t, "   observadores de pe: {} (esperado: 4)", vigia.quantos());

    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = writeln!(t, "   avisos na enumeracao inicial: {}", avisos.try_iter().count());
    let _ = writeln!(t);
}

fn secao_monitor(t: &mut String) {
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

    let sessoes: Vec<Sessao> = monitor.historico().sessoes(&panorama.principal.chave);
    secao_do_app(
        t,
        &AoVivo {
            principal: &panorama.principal.chave,
            estados: &panorama.todos,
            sessoes: &sessoes,
            pilula: None,
        },
    );
}

fn secao_tela(t: &mut String, pilula: Option<isize>) {
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowLongPtrW, IsWindowVisible, GWL_EXSTYLE, WS_EX_APPWINDOW,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    };

    use crate::tela::{self, Tela};

    let _ = writeln!(t, "=== Tela ===");
    let _ =
        writeln!(t, "   Quem entra aqui e so o nome do programa, nunca o caminho nem o titulo.");

    let estado = Tela::atual();
    let _ = writeln!(t, "   QUNS={}   {}", estado.bruto(), estado.descrever());
    let _ = writeln!(t, "   conta como jogo: {}", estado.conta_como_jogo());

    let em_foco = unsafe { GetForegroundWindow() };
    if em_foco.is_invalid() {
        let _ = writeln!(t, "   em foco: (nenhuma janela)");
    } else {
        let nome = crate::jogo::executavel_de(em_foco)
            .map(|c| crate::jogo::batizar(&c))
            .unwrap_or_else(|| "(sem acesso ao processo)".into());
        let _ = writeln!(t, "   em foco: {nome}");

        let mut info =
            MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let monitor = unsafe { MonitorFromWindow(em_foco, MONITOR_DEFAULTTONEAREST) };
        let tem_monitor = unsafe { GetMonitorInfoW(monitor, &mut info).as_bool() };
        match (tela::retangulo(em_foco), tem_monitor) {
            (Some(j), true) => {
                let m = info.rcMonitor;
                let cobre = j.left <= m.left
                    && j.top <= m.top
                    && j.right >= m.right
                    && j.bottom >= m.bottom;
                let _ = writeln!(
                    t,
                    "   janela {}x{} em ({},{})   monitor {}x{} em ({},{})   cobre o monitor: {cobre}",
                    j.right - j.left,
                    j.bottom - j.top,
                    j.left,
                    j.top,
                    m.right - m.left,
                    m.bottom - m.top,
                    m.left,
                    m.top
                );
            }
            _ => {
                let _ = writeln!(t, "   janela em foco sem retangulo legivel");
            }
        }
    }

    let Some(valor) = pilula else {
        let _ = writeln!(t, "   (sem pilula: modo --diagnose)");
        let _ = writeln!(t);
        return;
    };

    let alvo = crate::janelas::hwnd_de_valor(valor);
    let estilo = unsafe { GetWindowLongPtrW(alvo, GWL_EXSTYLE) } as u32;
    let nomes = [
        (WS_EX_TOPMOST.0, "TOPMOST"),
        (WS_EX_TOOLWINDOW.0, "TOOLWINDOW"),
        (WS_EX_NOACTIVATE.0, "NOACTIVATE"),
        (WS_EX_LAYERED.0, "LAYERED"),
        (WS_EX_TRANSPARENT.0, "TRANSPARENT"),
        (WS_EX_APPWINDOW.0, "APPWINDOW"),
    ];
    let ligados: Vec<&str> =
        nomes.iter().filter(|(bit, _)| estilo & bit != 0).map(|(_, nome)| *nome).collect();
    let _ = writeln!(t, "   pilula: EXSTYLE=0x{estilo:08x}   {}", ligados.join(" | "));
    let _ = writeln!(t, "   pilula visivel: {}", unsafe { IsWindowVisible(alvo).as_bool() });

    let por_cima = tela::alguem_por_cima(alvo)
        .map(|j| {
            crate::jogo::executavel_de(j)
                .map(|c| crate::jogo::batizar(&c))
                .unwrap_or_else(|| "(sem acesso ao processo)".into())
        })
        .unwrap_or_else(|| "ninguem".into());
    let _ = writeln!(t, "   por cima da pilula: {por_cima}");
    let _ = writeln!(t);
}

fn secao_do_app(t: &mut String, v: &AoVivo) {
    use crate::modelo::Via;

    let _ = writeln!(t, "=== O que o monitor conclui ===");

    for estado in v.estados {
        let marca = if estado.chave == v.principal { "  <- principal" } else { "" };
        let _ = writeln!(t, "   {}{marca}", estado.nome);
        let _ = writeln!(t, "      chave={}", estado.chave);
        let _ = writeln!(
            t,
            "      via={:?}   carga={}   precisao={:?}",
            estado.via,
            if estado.texto_da_carga.is_empty() { "nenhuma" } else { &estado.texto_da_carga },
            estado.precisao
        );
        let quando = match estado.lido_em {
            Some(ms) => crate::tempo::para_texto(ms),
            None => "nunca".to_string(),
        };
        let _ = writeln!(
            t,
            "      lido em {quando}   {}",
            if estado.leitura_antiga { "(ultima conhecida)" } else { "(ao vivo)" }
        );
        if estado.via == Via::Cabo {
            let _ = writeln!(t, "      carregando={}", estado.carregando);
        }
        if let Some(a) = &estado.autonomia {
            let _ = writeln!(t, "      autonomia: {a}");
        }

        if estado.chave != v.principal {
            continue;
        }
        if v.sessoes.is_empty() {
            let _ = writeln!(t, "      sessoes: nenhuma registrada");
        }
        for s in v.sessoes.iter().take(6) {
            let minutos = (s.fim - s.inicio) / 60_000;
            let _ = writeln!(
                t,
                "      sessao {} -> {}   {}% -> {}%   {minutos} min",
                crate::tempo::para_texto(s.inicio),
                crate::tempo::para_texto(s.fim),
                s.de,
                s.ate
            );
        }
    }
    let _ = writeln!(t);
}

fn secao_descoberta(t: &mut String) {
    let _ = writeln!(t, "=== Resultado final da descoberta (o que o app mostra) ===");
    let controles = descoberta::descobrir();
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
