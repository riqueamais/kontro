//! As quatro janelas do app e onde cada uma se ancora.
//!
//! Elas nao sao variacoes de uma so: cada uma tem um contrato diferente com o sistema.
//! A sobreposicao nao pode receber clique nem roubar foco; o aviso nasce e morre
//! sozinho; o painel segue a bandeja; a janela de configuracoes e a unica comum.

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
            WebviewWindowBuilder};

use crate::settings::{OverlayCorner, Settings};

/// Largura do painel da bandeja. A altura quem decide e o proprio conteudo.
///
/// Inclui as folgas laterais que a sombra ocupa: o painel visivel tem 328, e o resto e
/// espaco transparente para o desfoque se dissipar sem bater no limite da janela.
pub const LARGURA_DO_PAINEL: f64 = 392.0;

/// Folga transparente em volta do painel, igual as margens do CSS.
const FOLGA_LATERAL_DO_PAINEL: f64 = 32.0;
const FOLGA_INFERIOR_DO_PAINEL: f64 = 56.0;

/// O mesmo para o aviso.
const FOLGA_LATERAL_DO_AVISO: f64 = 32.0;
const FOLGA_SUPERIOR_DO_AVISO: f64 = 28.0;

pub const PRINCIPAL: &str = "principal";
pub const PAINEL: &str = "painel";
pub const SOBREPOSICAO: &str = "sobreposicao";
pub const AVISO: &str = "aviso";

/// Respiro entre a sobreposicao e a borda da tela.
const MARGEM_SOBREPOSICAO: f64 = 24.0;
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
            .inner_size(480.0, 720.0)
            .resizable(false)
            .visible(false)
            .center()
            .build()?;

    escurecer_barra_de_titulo(&janela);
    Ok(janela)
}

/// Pinta a barra de titulo de escuro.
///
/// E a unica janela do app com moldura do sistema. Sem isto ela ganha uma faixa clara
/// em cima do conteudo escuro -- o tipo de detalhe que denuncia na hora que o app foi
/// feito as pressas.
fn escurecer_barra_de_titulo(janela: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};

    let Ok(bruto) = janela.hwnd() else { return };
    let alvo = HWND(bruto.0 as *mut core::ffi::c_void);
    let ligado: i32 = 1;

    unsafe {
        let _ = DwmSetWindowAttribute(
            alvo,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &ligado as *const i32 as *const core::ffi::c_void,
            core::mem::size_of::<i32>() as u32,
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
    .inner_size(200.0, 72.0)
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

    let escolhida = cfg
        .overlay_monitor
        .try_into()
        .ok()
        .and_then(|i: usize| monitores.get(i))
        .or_else(|| janela.current_monitor().ok().flatten().as_ref().and_then(|_| monitores.first()))
        .or_else(|| monitores.first());

    let Some(monitor) = escolhida else { return };
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
    let _ = FOLGA_LATERAL_DO_AVISO;
    let x = posicao.x + (tamanho.width - tam_janela.width) / 2.0;
    let y = posicao.y + MARGEM_AVISO_TOPO - FOLGA_SUPERIOR_DO_AVISO;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

/// Encosta o painel no canto da bandeja.
pub fn posicionar_painel(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(PAINEL) else { return };
    let monitor = janela
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| janela.current_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };

    let escala = monitor.scale_factor();
    let posicao = monitor.position().to_logical::<f64>(escala);
    let tamanho = monitor.size().to_logical::<f64>(escala);
    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    // Ancorar pela janela deixaria o painel longe do canto: entre a borda da janela e a
    // borda visivel ha a folga da sombra. O respiro de baixo tambem e maior de proposito,
    // porque a barra de tarefas nao entra no que o sistema chama de area disponivel.
    let x = posicao.x + tamanho.width - tam_janela.width + FOLGA_LATERAL_DO_PAINEL - 12.0;
    let y = posicao.y + tamanho.height - tam_janela.height + FOLGA_INFERIOR_DO_PAINEL - 56.0;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}
