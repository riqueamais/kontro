use tauri::window::Monitor;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
            WebviewWindowBuilder};

use crate::settings::{OverlayCorner, Settings};
use crate::tela;

pub const LARGURA_DO_PAINEL: f64 = 392.0;

const FOLGA_LATERAL_DO_PAINEL: f64 = 32.0;
const FOLGA_INFERIOR_DO_PAINEL: f64 = 56.0;

const FOLGA_SUPERIOR_DO_AVISO: f64 = 28.0;

pub const PRINCIPAL: &str = "principal";
pub const PAINEL: &str = "painel";
pub const SOBREPOSICAO: &str = "sobreposicao";
pub const AVISO: &str = "aviso";

const MARGEM_SOBREPOSICAO: f64 = 24.0;

const LARGURA_DA_SOBREPOSICAO: f64 = 200.0;
const ALTURA_DA_SOBREPOSICAO: f64 = 72.0;

const LARGURA_POR_ACOMPANHANTE: f64 = 84.0;
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
            .decorations(false)
            .visible(false)
            .center()
            .build()?;

    arredondar_cantos(&janela);
    vestir_icone(&janela);
    Ok(janela)
}

fn vestir_icone(janela: &WebviewWindow) {
    if let Some(icone) = crate::tray::icone_do_app(crate::tray::tamanho_do_icone_grande()) {
        let _ = janela.set_icon(icone);
    }
}

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

pub fn posicionar_sobreposicao(app: &AppHandle, cfg: &Settings, ligados: usize) {
    let Some(janela) = app.get_webview_window(SOBREPOSICAO) else { return };
    let Ok(monitores) = app.available_monitors() else { return };
    if monitores.is_empty() {
        return;
    }

    let escolhido: Option<Monitor> = usize::try_from(cfg.overlay_monitor)
        .ok()
        .and_then(|i| monitores.get(i).cloned())
        .or_else(|| monitor_em_foco(app))
        .or_else(|| monitores.first().cloned());

    let Some(monitor) = escolhido else { return };
    let monitor = &monitor;

    let extra = LARGURA_POR_ACOMPANHANTE * ligados.saturating_sub(1) as f64;
    let _ = janela.set_size(LogicalSize::new(
        (LARGURA_DA_SOBREPOSICAO + extra) * cfg.overlay_scale,
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

    let x = posicao.x + (tamanho.width - tam_janela.width) / 2.0;
    let y = posicao.y + MARGEM_AVISO_TOPO - FOLGA_SUPERIOR_DO_AVISO;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

pub fn posicionar_painel(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(PAINEL) else { return };

    let monitor = monitor_do_cursor(app)
        .or_else(|| janela.current_monitor().ok().flatten())
        .or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };

    let escala = monitor.scale_factor();
    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    let area = monitor.work_area();
    let canto = area.position.to_logical::<f64>(escala);
    let util = area.size.to_logical::<f64>(escala);

    let x = canto.x + util.width - tam_janela.width + FOLGA_LATERAL_DO_PAINEL - 12.0;
    let y = canto.y + util.height - tam_janela.height + FOLGA_INFERIOR_DO_PAINEL - 8.0;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

fn monitor_em_foco(app: &AppHandle) -> Option<Monitor> {
    let (x, y) = tela::centro_da_janela_em_foco()?;
    app.monitor_from_point(x, y).ok().flatten()
}

fn monitor_do_cursor(app: &AppHandle) -> Option<Monitor> {
    let ponto = app.cursor_position().ok()?;
    app.monitor_from_point(ponto.x, ponto.y).ok().flatten()
}
