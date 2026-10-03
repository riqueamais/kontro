use tauri::window::{Color, Effect, EffectsBuilder, Monitor};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::Foundation::HWND;

use crate::configuracoes::{Settings, Theme};
use crate::sistema;
use crate::tela;

pub const LARGURA_DO_PAINEL: f64 = 328.0;

const MARGEM_DO_PAINEL: f64 = 12.0;

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
    tem_material: bool,
    material: bool,
}

impl Nascimento {
    pub fn de(cfg: &Settings) -> Self {
        Nascimento {
            tema: cfg.tema_efetivo(),
            tem_material: sistema::tem_material(),
            material: sistema::material_disponivel(),
        }
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

    if nascimento.tem_material {
        construtor = construtor.transparent(true);
        if nascimento.material {
            construtor = construtor.effects(EffectsBuilder::new().effect(Effect::MicaDark).build());
        }
    } else {
        construtor = construtor.background_color(nascimento.tinta());
    }

    let janela = construtor.build()?;

    arredondar_cantos(&janela);
    vestir_icone(&janela);
    Ok(janela)
}

pub fn vestir_material(app: &AppHandle, cfg: &Settings) {
    let claro = cfg.tema_claro();
    let tema = match cfg.theme {
        Theme::Sistema => None,
        _ if claro => Some(tauri::Theme::Light),
        _ => Some(tauri::Theme::Dark),
    };
    for rotulo in [PRINCIPAL, PAINEL, SOBREPOSICAO, AVISO] {
        if let Some(janela) = app.get_webview_window(rotulo) {
            let _ = janela.set_theme(tema);
        }
    }

    if !sistema::tem_material() {
        return;
    }
    let ligado = sistema::transparencia_ligada();

    if let Some(janela) = app.get_webview_window(PRINCIPAL) {
        let efeito = match (ligado, cfg.theme) {
            (false, _) | (true, Theme::Preto) => None,
            (true, _) if claro => Some(Effect::MicaLight),
            (true, _) => Some(Effect::MicaDark),
        };
        let _ = janela.set_effects(efeito.map(|e| EffectsBuilder::new().effect(e).build()));
    }

    if let Some(janela) = app.get_webview_window(PAINEL) {
        let efeito = ligado.then(|| EffectsBuilder::new().effect(Effect::Acrylic).build());
        let _ = janela.set_effects(efeito);
    }
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
    if let Ok(atual) = janela.outer_position() {
        let tolerancia = 0.5 * janela.scale_factor().unwrap_or(1.0);
        if (atual.x as f64 - x).abs() <= tolerancia && (atual.y as f64 - y).abs() <= tolerancia {
            return;
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Retangulo {
    x: f64,
    y: f64,
    largura: f64,
    altura: f64,
}

impl Retangulo {
    fn direita(&self) -> f64 {
        self.x + self.largura
    }

    fn baixo(&self) -> f64 {
        self.y + self.altura
    }
}

fn lugar_do_painel(
    tela: Retangulo,
    area: Retangulo,
    escala: f64,
    icone: Option<(f64, f64)>,
    altura: f64,
) -> Retangulo {
    let largura = LARGURA_DO_PAINEL * escala;
    let altura = altura * escala;
    let margem = MARGEM_DO_PAINEL * escala;

    let limitar_x = |x: f64| x.min(area.direita() - largura - margem).max(area.x + margem);
    let limitar_y = |y: f64| y.min(area.baixo() - altura - margem).max(area.y + margem);
    let junto_em_x = || limitar_x(icone.map_or(f64::MAX, |(x, _)| x - largura / 2.0));
    let junto_em_y = || limitar_y(icone.map_or(f64::MAX, |(_, y)| y - altura / 2.0));

    let (x, y) = if area.y > tela.y {
        (junto_em_x(), area.y + margem)
    } else if area.x > tela.x {
        (area.x + margem, junto_em_y())
    } else if area.largura < tela.largura {
        (area.direita() - largura - margem, junto_em_y())
    } else {
        (junto_em_x(), area.baixo() - altura - margem)
    };

    Retangulo { x, y, largura, altura }
}

pub fn assentar_painel(app: &AppHandle, icone: Option<(f64, f64)>, altura: f64) {
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

    let Some(janela) = app.get_webview_window(PAINEL) else { return };
    let monitor = icone
        .and_then(|(x, y)| app.monitor_from_point(x, y).ok().flatten())
        .or_else(|| monitor_do_cursor(app))
        .or_else(|| janela.current_monitor().ok().flatten())
        .or_else(|| janela.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let Some(alvo) = hwnd_de(&janela) else { return };
    let (Ok(fora), Ok(dentro)) = (janela.outer_size(), janela.inner_size()) else { return };

    let tela = Retangulo {
        x: monitor.position().x as f64,
        y: monitor.position().y as f64,
        largura: monitor.size().width as f64,
        altura: monitor.size().height as f64,
    };
    let util = monitor.work_area();
    let area = Retangulo {
        x: util.position.x as f64,
        y: util.position.y as f64,
        largura: util.size.width as f64,
        altura: util.size.height as f64,
    };
    let lugar = lugar_do_painel(tela, area, monitor.scale_factor(), icone, altura);

    let moldura_x = fora.width as i32 - dentro.width as i32;
    let moldura_y = fora.height as i32 - dentro.height as i32;
    let (recuo_x, recuo_y) = recuo_do_cliente(alvo);

    unsafe {
        let _ = SetWindowPos(
            alvo,
            None,
            lugar.x.round() as i32 - recuo_x,
            lugar.y.round() as i32 - recuo_y,
            lugar.largura.round() as i32 + moldura_x,
            lugar.altura.round() as i32 + moldura_y,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

fn recuo_do_cliente(alvo: HWND) -> (i32, i32) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::Graphics::Gdi::ClientToScreen;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

    let mut janela = RECT::default();
    let mut origem = POINT::default();
    unsafe {
        if GetWindowRect(alvo, &mut janela).is_err() || !ClientToScreen(alvo, &mut origem).as_bool()
        {
            return (0, 0);
        }
    }
    (origem.x - janela.left, origem.y - janela.top)
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

    const TELA: Retangulo = Retangulo { x: 0.0, y: 0.0, largura: 1920.0, altura: 1080.0 };

    #[test]
    fn com_a_barra_embaixo_o_painel_centra_sobre_o_icone() {
        let area = Retangulo { altura: 1032.0, ..TELA };
        let lugar = lugar_do_painel(TELA, area, 1.0, Some((1500.0, 1056.0)), 360.0);
        assert_eq!(lugar.x, 1500.0 - LARGURA_DO_PAINEL / 2.0);
        assert_eq!(lugar.baixo(), 1032.0 - MARGEM_DO_PAINEL);
    }

    #[test]
    fn o_icone_no_canto_nao_empurra_o_painel_para_fora() {
        let area = Retangulo { altura: 1032.0, ..TELA };
        let lugar = lugar_do_painel(TELA, area, 1.0, Some((1910.0, 1056.0)), 360.0);
        assert_eq!(lugar.direita(), 1920.0 - MARGEM_DO_PAINEL);
    }

    #[test]
    fn com_a_barra_no_topo_o_painel_encosta_nela() {
        let area = Retangulo { y: 48.0, altura: 1032.0, ..TELA };
        let lugar = lugar_do_painel(TELA, area, 1.0, Some((1500.0, 24.0)), 360.0);
        assert_eq!(lugar.y, 48.0 + MARGEM_DO_PAINEL);
        assert_eq!(lugar.x, 1500.0 - LARGURA_DO_PAINEL / 2.0);
    }

    #[test]
    fn com_a_barra_nas_laterais_o_painel_fica_do_lado_do_icone() {
        let esquerda = Retangulo { x: 64.0, largura: 1856.0, ..TELA };
        let lugar = lugar_do_painel(TELA, esquerda, 1.0, Some((32.0, 900.0)), 360.0);
        assert_eq!(lugar.x, 64.0 + MARGEM_DO_PAINEL);
        assert_eq!(lugar.baixo(), 1080.0 - MARGEM_DO_PAINEL);

        let direita = Retangulo { largura: 1856.0, ..TELA };
        let lugar = lugar_do_painel(TELA, direita, 1.0, Some((1888.0, 400.0)), 360.0);
        assert_eq!(lugar.direita(), 1856.0 - MARGEM_DO_PAINEL);
        assert_eq!(lugar.y, 400.0 - 180.0);
    }

    #[test]
    fn a_escala_vem_do_monitor_de_destino() {
        let tela = Retangulo { x: 1920.0, y: 0.0, largura: 3840.0, altura: 2160.0 };
        let area = Retangulo { altura: 2088.0, ..tela };
        let lugar = lugar_do_painel(tela, area, 1.5, None, 360.0);
        assert_eq!(lugar.largura, LARGURA_DO_PAINEL * 1.5);
        assert_eq!(lugar.direita(), 1920.0 + 3840.0 - MARGEM_DO_PAINEL * 1.5);
        assert_eq!(lugar.baixo(), 2088.0 - MARGEM_DO_PAINEL * 1.5);
    }

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
