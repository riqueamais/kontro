use tauri::window::{Color, Effect, EffectsBuilder, Monitor};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::Foundation::HWND;

use crate::configuracoes::Settings;
use crate::sistema;
use crate::tela;

pub const LARGURA_DO_PAINEL: f64 = 328.0;

const MARGEM_LATERAL_DO_PAINEL: f64 = 12.0;
const MARGEM_INFERIOR_DO_PAINEL: f64 = 8.0;

const SANGRIA_SUPERIOR_DO_AVISO: f64 = 28.0;

pub const PRINCIPAL: &str = "principal";
pub const PAINEL: &str = "painel";
pub const SOBREPOSICAO: &str = "sobreposicao";
pub const AVISO: &str = "aviso";

const LARGURA_DA_SOBREPOSICAO: f64 = 200.0;
const ALTURA_DA_SOBREPOSICAO: f64 = 72.0;

const FOLGA_DO_ENCAIXE: f64 = 28.0;

const MARGEM_DO_AVISO: f64 = 24.0;

const TINTA_DA_NOITE: Color = Color(11, 14, 17, 255);
const TINTA_DO_DIA: Color = Color(238, 241, 245, 255);

pub struct Nascimento {
    tema: &'static str,
    material: bool,
}

impl Nascimento {
    pub fn de(cfg: &Settings) -> Self {
        Nascimento { tema: cfg.tema_efetivo(), material: sistema::material_disponivel() }
    }

    fn endereco(&self, janela: &str) -> WebviewUrl {
        let material = if self.material { "sim" } else { "nao" };
        WebviewUrl::App(
            format!("index.html?janela={janela}&tema={}&material={material}", self.tema).into(),
        )
    }

    fn tinta(&self) -> Color {
        if self.tema == "dia" {
            TINTA_DO_DIA
        } else {
            TINTA_DA_NOITE
        }
    }
}

pub fn criar_todas(app: &AppHandle, nascimento: &Nascimento) -> tauri::Result<()> {
    criar_principal(app, nascimento)?;
    criar_painel(app, nascimento)?;
    criar_sobreposicao(app, nascimento)?;
    criar_aviso(app, nascimento)?;
    Ok(())
}

fn criar_principal(app: &AppHandle, nascimento: &Nascimento) -> tauri::Result<WebviewWindow> {
    let mut construtor = WebviewWindowBuilder::new(app, PRINCIPAL, nascimento.endereco(PRINCIPAL))
        .title("Kontro")
        .inner_size(840.0, 600.0)
        .min_inner_size(720.0, 520.0)
        .decorations(false)
        .visible(false)
        .center();

    if nascimento.material {
        construtor = construtor
            .transparent(true)
            .effects(EffectsBuilder::new().effect(Effect::MicaDark).build());
    } else {
        construtor = construtor.background_color(nascimento.tinta());
    }

    let janela = construtor.build()?;

    arredondar_cantos(&janela);
    vestir_icone(&janela);
    Ok(janela)
}

pub fn vestir_material(app: &AppHandle, claro: bool) {
    if !sistema::material_disponivel() {
        return;
    }
    let Some(janela) = app.get_webview_window(PRINCIPAL) else { return };
    let efeito = if claro { Effect::MicaLight } else { Effect::MicaDark };
    let _ = janela.set_effects(EffectsBuilder::new().effect(efeito).build());
}

fn vestir_icone(janela: &WebviewWindow) {
    if let Some(icone) = crate::bandeja::icone_do_app(crate::bandeja::tamanho_do_icone_grande()) {
        let _ = janela.set_icon(icone);
    }
}

pub fn hwnd_de(janela: &WebviewWindow) -> Option<HWND> {
    let bruto = janela.hwnd().ok()?;
    Some(HWND(bruto.0 as *mut core::ffi::c_void))
}

pub fn hwnd_de_valor(valor: isize) -> HWND {
    HWND(valor as *mut core::ffi::c_void)
}

pub fn vestir_estilos(janela: &WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
        WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };

    let Some(alvo) = hwnd_de(janela) else { return };

    unsafe {
        let antes = GetWindowLongPtrW(alvo, GWL_EXSTYLE);
        let depois = (antes | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize);
        if depois != antes {
            SetWindowLongPtrW(alvo, GWL_EXSTYLE, depois);
        }
        let _ = SetWindowPos(
            alvo,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
        );
    }
}

pub fn mostrar_por_cima(janela: &WebviewWindow) {
    let _ = janela.show();
    vestir_estilos(janela);
}

pub fn mostrar_por_cima_da_thread_da_interface(app: &AppHandle, janela: WebviewWindow) {
    let _ = app.run_on_main_thread(move || mostrar_por_cima(&janela));
}

fn arredondar_cantos(janela: &WebviewWindow) {
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    let Some(alvo) = hwnd_de(janela) else { return };
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

fn criar_painel(app: &AppHandle, nascimento: &Nascimento) -> tauri::Result<WebviewWindow> {
    let mut construtor = WebviewWindowBuilder::new(app, PAINEL, nascimento.endereco(PAINEL))
        .title("Kontro")
        .inner_size(LARGURA_DO_PAINEL, 360.0)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false);

    if nascimento.material {
        construtor = construtor.effects(EffectsBuilder::new().effect(Effect::Acrylic).build());
    }

    let janela = construtor.build()?;

    arredondar_cantos(&janela);
    Ok(janela)
}

fn criar_sobreposicao(app: &AppHandle, nascimento: &Nascimento) -> tauri::Result<WebviewWindow> {
    let janela = WebviewWindowBuilder::new(app, SOBREPOSICAO, nascimento.endereco(SOBREPOSICAO))
        .title("Kontro")
        .inner_size(LARGURA_DA_SOBREPOSICAO, ALTURA_DA_SOBREPOSICAO)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .focusable(false)
        .visible(false)
        .build()?;

    let _ = janela.set_ignore_cursor_events(true);
    vestir_estilos(&janela);
    Ok(janela)
}

fn criar_aviso(app: &AppHandle, nascimento: &Nascimento) -> tauri::Result<WebviewWindow> {
    let janela = WebviewWindowBuilder::new(app, AVISO, nascimento.endereco(AVISO))
        .title("Kontro")
        .inner_size(384.0, 180.0)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .focusable(false)
        .visible(false)
        .build()?;

    let _ = janela.set_ignore_cursor_events(true);
    vestir_estilos(&janela);
    Ok(janela)
}

pub fn redimensionar_sobreposicao(
    app: &AppHandle,
    cfg: &Settings,
    solta: bool,
    largura: f64,
    altura: f64,
) {
    let Some(janela) = app.get_webview_window(SOBREPOSICAO) else { return };

    let escala = janela.scale_factor().unwrap_or(1.0);
    let Ok(antes) = janela.outer_size() else { return };

    let _ = janela.set_size(LogicalSize::new(largura, altura));

    if !solta {
        posicionar_sobreposicao(app, cfg);
        return;
    }

    let dx = if cfg.overlay_x > 0.5 { antes.width as f64 - largura * escala } else { 0.0 };
    let dy = if cfg.overlay_y > 0.5 { antes.height as f64 - altura * escala } else { 0.0 };
    if dx.abs() < 1.0 && dy.abs() < 1.0 {
        return;
    }

    let Ok(posicao) = janela.outer_position() else { return };
    let _ = janela.set_position(PhysicalPosition::new(
        posicao.x + dx.round() as i32,
        posicao.y + dy.round() as i32,
    ));
}

pub fn posicionar_sobreposicao(app: &AppHandle, cfg: &Settings) {
    let Some(janela) = app.get_webview_window(SOBREPOSICAO) else { return };
    let Some(monitor) = monitor_da_sobreposicao(app, cfg) else { return };
    let Some(palco) = palco(&monitor, &janela) else { return };

    let x = palco.esquerda + palco.livre_x * cfg.overlay_x;
    let y = palco.topo + palco.livre_y * cfg.overlay_y;
    let _ = janela.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

pub struct Pouso {
    pub x: f64,
    pub y: f64,
    pub monitor: i32,
}

pub fn onde_a_sobreposicao_parou(app: &AppHandle) -> Option<Pouso> {
    let janela = app.get_webview_window(SOBREPOSICAO)?;
    let monitor = janela.current_monitor().ok().flatten()?;
    let palco = palco(&monitor, &janela)?;

    let posicao = janela.outer_position().ok()?;

    Some(Pouso {
        x: encaixar(posicao.x as f64 - palco.esquerda, palco.livre_x, palco.folga),
        y: encaixar(posicao.y as f64 - palco.topo, palco.livre_y, palco.folga),
        monitor: indice_do_monitor(app, &monitor).unwrap_or(-1),
    })
}

struct Palco {
    esquerda: f64,
    topo: f64,
    livre_x: f64,
    livre_y: f64,
    folga: f64,
}

fn palco(monitor: &Monitor, janela: &WebviewWindow) -> Option<Palco> {
    let escala_do_monitor = monitor.scale_factor();
    let escala_da_janela = janela.scale_factor().unwrap_or(escala_do_monitor);
    let conversao = escala_do_monitor / escala_da_janela;

    let posicao = monitor.position();
    let tamanho = monitor.size();
    let janela = janela.outer_size().ok()?;

    Some(Palco {
        esquerda: posicao.x as f64,
        topo: posicao.y as f64,
        livre_x: (tamanho.width as f64 - janela.width as f64 * conversao).max(0.0),
        livre_y: (tamanho.height as f64 - janela.height as f64 * conversao).max(0.0),
        folga: FOLGA_DO_ENCAIXE * escala_do_monitor,
    })
}

fn encaixar(desvio: f64, livre: f64, folga: f64) -> f64 {
    if livre <= 0.0 {
        return 0.0;
    }

    let fracao = (desvio / livre).clamp(0.0, 1.0);
    [0.0, 0.5, 1.0]
        .into_iter()
        .find(|encaixe| (fracao - encaixe).abs() * livre <= folga)
        .unwrap_or(fracao)
}

fn monitor_da_sobreposicao(app: &AppHandle, cfg: &Settings) -> Option<Monitor> {
    let monitores = app.available_monitors().ok()?;
    if monitores.is_empty() {
        return None;
    }

    usize::try_from(cfg.overlay_monitor)
        .ok()
        .and_then(|i| monitores.get(i).cloned())
        .or_else(|| monitor_em_foco(app))
        .or_else(|| monitores.first().cloned())
}

fn indice_do_monitor(app: &AppHandle, alvo: &Monitor) -> Option<i32> {
    let monitores = app.available_monitors().ok()?;
    let onde = monitores.iter().position(|m| m.position() == alvo.position())?;
    i32::try_from(onde).ok()
}

pub fn posicionar_aviso(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(AVISO) else { return };
    let monitor =
        janela.current_monitor().ok().flatten().or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };

    let escala = monitor.scale_factor();
    let posicao = monitor.position().to_logical::<f64>(escala);
    let tamanho = monitor.size().to_logical::<f64>(escala);
    let Ok(tam_janela) = janela.outer_size() else { return };
    let tam_janela: LogicalSize<f64> = tam_janela.to_logical(escala);

    let x = posicao.x + (tamanho.width - tam_janela.width) / 2.0;
    let y = posicao.y + MARGEM_DO_AVISO - SANGRIA_SUPERIOR_DO_AVISO;
    let _ = janela.set_position(LogicalPosition::new(x, y));
}

pub fn posicionar_painel(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(PAINEL) else { return };

    let monitor = monitor_do_cursor(app)
        .or_else(|| janela.current_monitor().ok().flatten())
        .or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let Ok(tamanho) = janela.outer_size() else { return };

    let canto = canto_do_painel(&monitor, tamanho.width as i32, tamanho.height as i32);
    let _ = janela.set_position(canto);
}

pub fn redimensionar_painel(janela: &WebviewWindow, altura: f64) {
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

    let monitor =
        janela.current_monitor().ok().flatten().or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let Some(alvo) = hwnd_de(janela) else { return };
    let (Ok(fora), Ok(dentro)) = (janela.outer_size(), janela.inner_size()) else { return };

    let escala = janela.scale_factor().unwrap_or_else(|_| monitor.scale_factor());
    let moldura_x = fora.width as i32 - dentro.width as i32;
    let moldura_y = fora.height as i32 - dentro.height as i32;
    let largura = (LARGURA_DO_PAINEL * escala).round() as i32 + moldura_x;
    let altura = (altura * escala).round() as i32 + moldura_y;
    let canto = canto_do_painel(&monitor, largura, altura);

    unsafe {
        let _ = SetWindowPos(
            alvo,
            None,
            canto.x,
            canto.y,
            largura,
            altura,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

fn canto_do_painel(monitor: &Monitor, largura: i32, altura: i32) -> PhysicalPosition<i32> {
    let escala = monitor.scale_factor();
    let area = monitor.work_area();
    let margem_lateral = (MARGEM_LATERAL_DO_PAINEL * escala).round() as i32;
    let margem_inferior = (MARGEM_INFERIOR_DO_PAINEL * escala).round() as i32;

    PhysicalPosition::new(
        area.position.x + area.size.width as i32 - largura - margem_lateral,
        area.position.y + area.size.height as i32 - altura - margem_inferior,
    )
}

fn monitor_em_foco(app: &AppHandle) -> Option<Monitor> {
    let (x, y) = tela::centro_da_janela_em_foco()?;
    app.monitor_from_point(x, y).ok().flatten()
}

fn monitor_do_cursor(app: &AppHandle) -> Option<Monitor> {
    let ponto = app.cursor_position().ok()?;
    app.monitor_from_point(ponto.x, ponto.y).ok().flatten()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn largar_quase_no_canto_vale_como_canto() {
        assert_eq!(encaixar(9.0, 1000.0, FOLGA_DO_ENCAIXE), 0.0);
        assert_eq!(encaixar(985.0, 1000.0, FOLGA_DO_ENCAIXE), 1.0);
    }

    #[test]
    fn a_folga_do_encaixe_cresce_com_a_escala_do_monitor() {
        let em_150 = FOLGA_DO_ENCAIXE * 1.5;
        assert_eq!(encaixar(40.0, 1000.0, em_150), 0.0);
        assert_eq!(encaixar(40.0, 1000.0, FOLGA_DO_ENCAIXE), 0.04);
    }

    #[test]
    fn largar_quase_no_meio_vale_como_meio() {
        assert_eq!(encaixar(510.0, 1000.0, FOLGA_DO_ENCAIXE), 0.5);
    }

    #[test]
    fn no_meio_do_caminho_a_escolha_e_respeitada() {
        assert_eq!(encaixar(250.0, 1000.0, FOLGA_DO_ENCAIXE), 0.25);
    }

    #[test]
    fn tela_sem_folga_nao_divide_por_zero() {
        assert_eq!(encaixar(120.0, 0.0, FOLGA_DO_ENCAIXE), 0.0);
    }

    #[test]
    fn largar_fora_da_tela_volta_para_dentro() {
        assert_eq!(encaixar(-400.0, 1000.0, FOLGA_DO_ENCAIXE), 0.0);
        assert_eq!(encaixar(4000.0, 1000.0, FOLGA_DO_ENCAIXE), 1.0);
    }
}
