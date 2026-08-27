//! As quatro janelas do app e onde cada uma se ancora.
//!
//! Elas nao sao variacoes de uma so: cada uma tem um contrato diferente com o sistema.
//! A sobreposicao nao pode receber clique nem roubar foco; o aviso nasce e morre
//! sozinho; o painel segue a bandeja; a janela de configuracoes e a unica comum.

use tauri::window::Monitor;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
            WebviewWindowBuilder};

use crate::settings::{OverlayCorner, Settings};
use crate::tela;

/// Largura do painel da bandeja. A altura quem decide e o proprio conteudo.
///
/// Inclui as folgas laterais que a sombra ocupa: o painel visivel tem 328, e o resto e
/// espaco transparente para o desfoque se dissipar sem bater no limite da janela.
pub const LARGURA_DO_PAINEL: f64 = 392.0;

/// Folga transparente em volta do painel, igual as margens do CSS.
const FOLGA_LATERAL_DO_PAINEL: f64 = 32.0;
const FOLGA_INFERIOR_DO_PAINEL: f64 = 56.0;

/// O mesmo para o aviso.
const FOLGA_SUPERIOR_DO_AVISO: f64 = 28.0;

pub const PRINCIPAL: &str = "principal";
pub const PAINEL: &str = "painel";
pub const SOBREPOSICAO: &str = "sobreposicao";
pub const AVISO: &str = "aviso";

/// Respiro entre a sobreposicao e a borda da tela.
const MARGEM_SOBREPOSICAO: f64 = 24.0;

/// Tamanho da janela da sobreposicao na escala 1. A pilula desenhada e menor: o resto e
/// folga transparente para a sombra.
const LARGURA_DA_SOBREPOSICAO: f64 = 200.0;
const ALTURA_DA_SOBREPOSICAO: f64 = 72.0;
const MARGEM_AVISO_TOPO: f64 = 24.0;

pub fn criar_todas(app: &AppHandle) -> tauri::Result<()> {
    criar_principal(app)?;
    criar_painel(app)?;
    criar_sobreposicao(app)?;
    criar_aviso(app)?;
    Ok(())
}

fn criar_principal(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let janela =
        WebviewWindowBuilder::new(app, PRINCIPAL, WebviewUrl::App("index.html?janela=principal".into()))
            .title("Kontro")
            .inner_size(840.0, 600.0)
            .min_inner_size(720.0, 520.0)
            // A moldura e desenhada pelo app. A do sistema traz uma faixa de outro
            // material em cima do conteudo, e obriga a barra lateral a comecar abaixo
            // dela -- o app pareceria uma pagina dentro de uma janela, e nao um app.
            .decorations(false)
            .visible(false)
            .center()
            .build()?;

    arredondar_cantos(&janela);
    vestir_icone(&janela);
    Ok(janela)
}

/// Desenha o icone da janela no tamanho que esta tela pede.
///
/// O icone que vem do pacote tem tamanho fixo, e o sistema estica ou encolhe o que
/// recebeu. Desenhar no tamanho final custa alguns milissegundos uma vez e resolve o
/// borrao da barra de tarefas em telas com escala.
fn vestir_icone(janela: &WebviewWindow) {
    if let Some(icone) = crate::tray::icone_do_app(crate::tray::tamanho_do_icone_grande()) {
        let _ = janela.set_icon(icone);
    }
}

/// Devolve os cantos arredondados que a janela perdeu ao dispensar a moldura.
///
/// Sem moldura o Windows entrega um retangulo de canto vivo, que ao lado de qualquer
/// outra janela do sistema salta aos olhos. O DWM arredonda do lado dele, junto com a
/// sombra, que e o unico jeito de ficar igual ao resto do sistema.
fn arredondar_cantos(janela: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    let Ok(bruto) = janela.hwnd() else { return };
    let alvo = HWND(bruto.0 as *mut core::ffi::c_void);
    let preferencia = DWMWCP_ROUND;

    unsafe {
        let _ = DwmSetWindowAttribute(
            alvo,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preferencia as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&preferencia) as u32,
        );
    }
}

fn criar_painel(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, PAINEL, WebviewUrl::App("index.html?janela=painel".into()))
        .title("Kontro")
        // Altura inicial apenas: o painel se mede depois de desenhado e pede a altura
        // exata de que precisa.
        .inner_size(LARGURA_DO_PAINEL, 360.0)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()
}

fn criar_sobreposicao(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let janela = WebviewWindowBuilder::new(
        app,
        SOBREPOSICAO,
        WebviewUrl::App("index.html?janela=sobreposicao".into()),
    )
    .title("Kontro")
    .inner_size(LARGURA_DA_SOBREPOSICAO, ALTURA_DA_SOBREPOSICAO)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .visible(false)
    .build()?;

    // Sem isto um clique perdido pousaria nela em vez de no jogo, e o aviso roubaria o
    // foco da partida no pior momento possivel.
    let _ = janela.set_ignore_cursor_events(true);
    Ok(janela)
}

fn criar_aviso(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let janela =
        WebviewWindowBuilder::new(app, AVISO, WebviewUrl::App("index.html?janela=aviso".into()))
            .title("Kontro")
            .inner_size(384.0, 180.0)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false)
            .visible(false)
            .build()?;

    let _ = janela.set_ignore_cursor_events(true);
    Ok(janela)
}

/// Ancora a sobreposicao no canto e na tela que o usuario escolheu.
///
/// Acompanhar o foco por padrao e o certo para quem joga em uma tela so; com duas, isso
/// faz a pilula pular de monitor a cada clique fora do jogo, que e o oposto de "fixa".
/// Por isso a preferencia manda quando ela existe.
pub fn posicionar_sobreposicao(app: &AppHandle, cfg: &Settings) {
    let Some(janela) = app.get_webview_window(SOBREPOSICAO) else { return };
    let Ok(monitores) = app.available_monitors() else { return };
    if monitores.is_empty() {
        return;
    }

    // Com um monitor escolhido, ele manda. Sem escolha, a pilula segue a janela em foco
    // -- que e o proprio sentido de "segue o jogo".
    //
    // A busca antiga chamava `current_monitor`, descartava o resultado com um
    // `and_then(|_| monitores.first())` e devolvia sempre o primeiro monitor: quem
    // deixasse a opcao no padrao ficava com a pilula presa na tela 1 para sempre.
    let escolhido: Option<Monitor> = usize::try_from(cfg.overlay_monitor)
        .ok()
        .and_then(|i| monitores.get(i).cloned())
        .or_else(|| monitor_em_foco(app))
        .or_else(|| monitores.first().cloned());

    let Some(monitor) = escolhido else { return };
    let monitor = &monitor;

    // A janela acompanha a escala da pilula: mantida no tamanho de fabrica, a pilula
    // ampliada apareceria cortada nas bordas.
    let _ = janela.set_size(LogicalSize::new(
        LARGURA_DA_SOBREPOSICAO * cfg.overlay_scale,
        ALTURA_DA_SOBREPOSICAO * cfg.overlay_scale,
    ));
    let escala = monitor.scale_factor();
    let posicao = monitor.position().to_logical::<f64>(escala);
    let tamanho = monitor.size().to_logical::<f64>(escala);

    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    let esquerda = posicao.x + MARGEM_SOBREPOSICAO;
    let direita = posicao.x + tamanho.width - tam_janela.width - MARGEM_SOBREPOSICAO;
    let topo = posicao.y + MARGEM_SOBREPOSICAO;
    let base = posicao.y + tamanho.height - tam_janela.height - MARGEM_SOBREPOSICAO;

    let (x, y) = match cfg.overlay_corner {
        OverlayCorner::SuperiorEsquerdo => (esquerda, topo),
        OverlayCorner::SuperiorDireito => (direita, topo),
        OverlayCorner::InferiorEsquerdo => (esquerda, base),
        OverlayCorner::InferiorDireito => (direita, base),
    };

    let _ = janela.set_position(LogicalPosition::new(x, y));
}

/// Centraliza o aviso no topo da tela em foco.
///
/// Com dois monitores, avisar do lado errado nao avisa nada.
pub fn posicionar_aviso(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(AVISO) else { return };
    let monitor = janela
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };

    let escala = monitor.scale_factor();
    let posicao = monitor.position().to_logical::<f64>(escala);
    let tamanho = monitor.size().to_logical::<f64>(escala);
    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    // centralizar a janela centraliza o cartao junto, porque as folgas laterais sao
    // iguais; o topo precisa descontar a folga para o cartao ficar onde se pediu
    let x = posicao.x + (tamanho.width - tam_janela.width) / 2.0;
    let y = posicao.y + MARGEM_AVISO_TOPO - FOLGA_SUPERIOR_DO_AVISO;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

/// Encosta o painel no canto da bandeja.
pub fn posicionar_painel(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(PAINEL) else { return };

    // O painel nasce de um clique no icone, entao o cursor esta na tela certa por
    // definicao. Ancorar no monitor primario abria o painel do outro lado para quem tem a
    // barra de tarefas na segunda tela.
    let monitor = monitor_do_cursor(app)
        .or_else(|| janela.current_monitor().ok().flatten())
        .or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };

    let escala = monitor.scale_factor();
    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    // A area de trabalho ja desconta a barra de tarefas, esteja ela embaixo, do lado ou
    // com altura fora do comum. Antes o desconto era um "-56" fixo sobre a resolucao
    // cheia, que so acertava na configuracao mais comum.
    let area = monitor.work_area();
    let canto = area.position.to_logical::<f64>(escala);
    let util = area.size.to_logical::<f64>(escala);

    // Ancorar pela borda da janela deixaria o painel longe do canto: entre ela e a borda
    // visivel ha a folga transparente onde a sombra se dissipa.
    let x = canto.x + util.width - tam_janela.width + FOLGA_LATERAL_DO_PAINEL - 12.0;
    let y = canto.y + util.height - tam_janela.height + FOLGA_INFERIOR_DO_PAINEL - 8.0;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

/// A tela onde esta a janela em foco.
fn monitor_em_foco(app: &AppHandle) -> Option<Monitor> {
    let (x, y) = tela::centro_da_janela_em_foco()?;
    app.monitor_from_point(x, y).ok().flatten()
}

/// A tela onde esta o cursor.
fn monitor_do_cursor(app: &AppHandle) -> Option<Monitor> {
    let ponto = app.cursor_position().ok()?;
    app.monitor_from_point(ponto.x, ponto.y).ok().flatten()
}
