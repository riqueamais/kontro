//! O icone da bandeja, desenhado a cada mudanca.
//!
//! O icone nao ilustra o dado: ele *e* o dado. O anel acompanha a carga e o tema do
//! sistema, entao nao da para usar arquivo pronto -- cada estado vira um desenho novo,
//! no tamanho exato que o Windows pediu. Escalar um bitmap depois borraria justamente as
//! bordas finas que fazem o anel se ler em 16 pixels.

use tauri::image::Image;

use crate::geometria as g;
use crate::model::{BatteryState, LinkMode};

const VERMELHO_ABAIXO: i32 = 30;
const AMBAR_ABAIXO: i32 = 60;

/// Desenha o icone para o estado atual.
///
/// `tema_claro` diz se a barra de tarefas esta clara: no claro o controle precisa ser
/// escuro, senao ele some no proprio fundo.
pub fn desenhar(estado: &BatteryState, tamanho: u32, tema_claro: bool) -> Option<Image<'static>> {
    let svg = montar_svg(estado, tema_claro);
    rasterizar(&svg, tamanho)
}

fn montar_svg(estado: &BatteryState, tema_claro: bool) -> String {
    let caixa = g::CAIXA;
    let centro = caixa / 2.0;

    let cor_glifo = if tema_claro { g::GLIFO_NO_CLARO } else { "#FFFFFF" };

    // Desconectado nao desenha anel nenhum: um anel cinza cheio diria "medi e deu zero",
    // que e diferente de "nao ha o que medir".
    let (cor_anel, preenchimento) = match (estado.mode, estado.preenchimento) {
        (LinkMode::Offline, _) => (g::CINZA, None),
        (LinkMode::Cable, None) => (g::TEAL, Some(100)),
        (_, Some(p)) => (g::cor_do_nivel(p, VERMELHO_ABAIXO, AMBAR_ABAIXO), Some(p)),
        (_, None) => (g::CINZA, None),
    };

    let trilho = format!(
        r#"<circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="{cor_anel}" stroke-opacity="0.22" stroke-width="{}"/>"#,
        g::ANEL_RAIO,
        g::ANEL_LARGURA
    );

    let anel = match preenchimento {
        Some(p) if p >= 100 => format!(
            r#"<circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="{cor_anel}" stroke-width="{}"/>"#,
            g::ANEL_RAIO,
            g::ANEL_LARGURA
        ),
        Some(p) if p > 0 => format!(
            r#"<path d="{}" fill="none" stroke="{cor_anel}" stroke-width="{}" stroke-linecap="round"/>"#,
            g::arco(360.0 * p as f32 / 100.0),
            g::ANEL_LARGURA
        ),
        _ => String::new(),
    };

    let glifo = format!(
        r#"<g transform="translate({centro},{}) scale({}) translate({},{})"><path d="{}" fill="{cor_glifo}" fill-rule="evenodd"/></g>"#,
        g::PAD_CENTRO_Y_BANDEJA,
        g::PAD_ESCALA_BANDEJA,
        -centro,
        -g::PAD_CENTRO_Y,
        g::pad_com_sticks_vazados()
    );

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {caixa} {caixa}" width="{caixa}" height="{caixa}">{trilho}{anel}{glifo}</svg>"#
    )
}

fn rasterizar(svg: &str, tamanho: u32) -> Option<Image<'static>> {
    let opcoes = usvg::Options::default();
    let arvore = usvg::Tree::from_str(svg, &opcoes).ok()?;

    let mut mapa = tiny_skia::Pixmap::new(tamanho, tamanho)?;
    let escala = tamanho as f32 / g::CAIXA;
    resvg::render(
        &arvore,
        tiny_skia::Transform::from_scale(escala, escala),
        &mut mapa.as_mut(),
    );

    Some(Image::new_owned(mapa.take(), tamanho, tamanho))
}

/// Tamanho que o Windows espera para o icone da bandeja, em pixels reais.
///
/// Desenhar maior e deixar o sistema reduzir borra justamente as bordas finas que fazem
/// o anel se ler nesse tamanho. A metrica ja acompanha a escala da tela, entao ela vale
/// tanto em 100% quanto em 150%.
pub fn tamanho_do_icone() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};
    let medido = unsafe { GetSystemMetrics(SM_CXSMICON) };
    if medido <= 0 {
        16
    } else {
        medido as u32
    }
}

/// A barra de tarefas esta no tema claro?
pub fn barra_clara() -> bool {
    // A chave e por usuario e muda ao vivo quando ele troca o tema; ler na hora e mais
    // barato e mais correto do que cachear e tentar descobrir quando invalidar.
    let saida = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "SystemUsesLightTheme",
        ])
        .output();

    match saida {
        Ok(s) => String::from_utf8_lossy(&s.stdout).contains("0x1"),
        Err(_) => false,
    }
}
