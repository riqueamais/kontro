use tauri::image::Image;

use crate::geometria as g;
use crate::modelo::{EstadoDoControle, Via};
use crate::configuracoes::Limiares;

pub fn desenhar(estado: &EstadoDoControle, tamanho: u32, limiares: Limiares) -> Option<Image<'static>> {
    rasterizar(&montar_svg(estado, limiares), tamanho)
}

pub(crate) fn montar_svg(estado: &EstadoDoControle, limiares: Limiares) -> String {
    let caixa = g::CAIXA;
    let centro = caixa / 2.0;
    let raio = g::ANEL_RAIO;
    let grossura = g::ANEL_LARGURA;

    let cor_glifo = g::BRANCO;

    let fundo = format!(
        r##"<circle cx="{centro}" cy="{centro}" r="{}" fill="{}"/><circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="{}" stroke-opacity="{}" stroke-width="{}"/>"##,
        g::FUNDO_RAIO,
        g::FUNDO,
        g::FUNDO_RAIO - g::BORDA_LARGURA / 2.0,
        g::BRANCO,
        g::BORDA_OPACIDADE,
        g::BORDA_LARGURA
    );

    let trilho = format!(
        r##"<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{cor_glifo}" stroke-opacity="{}" stroke-width="{grossura}"/>"##,
        g::TRILHO_OPACIDADE
    );

    let desenhar_glifo = |opacidade: f32| {
        format!(
            r##"<g opacity="{opacidade}" transform="translate({centro},{}) scale({}) translate({},{})"><path d="{}" fill="{cor_glifo}" fill-rule="evenodd"/></g>"##,
            g::PAD_CENTRO_Y_BANDEJA,
            g::PAD_ESCALA_BANDEJA,
            -centro,
            -g::PAD_CENTRO_Y,
            g::pad_com_sticks_vazados()
        )
    };

    let miolo = match (estado.via, estado.preenchimento) {

        (Via::Desligado, _) => format!(
            r##"{trilho}{}<path d="{}" stroke="{cor_glifo}" stroke-width="{}" stroke-linecap="round"/>"##,
            desenhar_glifo(g::GLIFO_APAGADO),
            g::RISCO,
            g::RISCO_LARGURA
        ),

        (Via::Cabo, None) => format!(
            r##"{trilho}<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{}" stroke-width="{grossura}"/>{}"##,
            g::CINZA,
            desenhar_glifo(1.0)
        ),

        (_, Some(p)) => {
            let cor = g::cor_do_nivel(p, limiares.critico, limiares.aviso);
            let arco = if p >= 100 {
                format!(
                    r##"<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{cor}" stroke-width="{grossura}"/>"##
                )
            } else if p > 0 {
                format!(
                    r##"<path d="{}" fill="none" stroke="{cor}" stroke-width="{grossura}" stroke-linecap="round"/>"##,
                    g::arco(360.0 * p as f32 / 100.0)
                )
            } else {
                String::new()
            };
            format!("{trilho}{arco}{}", desenhar_glifo(1.0))
        }

        (_, None) => format!("{trilho}{}", desenhar_glifo(1.0)),
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {caixa} {caixa}" width="{caixa}" height="{caixa}">{fundo}{miolo}</svg>"##
    )
}

pub(crate) fn estado_demo(preenchimento: Option<i32>, modo: Via) -> EstadoDoControle {
    EstadoDoControle::montar(crate::modelo::Bruto {
        via: modo,
        percentual: preenchimento,
        precisao: if preenchimento.is_some() {
            crate::modelo::Precisao::Exata
        } else {
            crate::modelo::Precisao::Nenhuma
        },
        nome: "demo".into(),
        chave: "demo".into(),
        quantos_conhecidos: 1,
        ..Default::default()
    })
}

pub fn salvar_previa(caminho: &str, tamanho: u32, fundo_claro: bool) -> Option<()> {
    let exemplos: [(Option<i32>, Via); 5] = [
        (None, Via::Desligado),
        (Some(100), Via::Bluetooth),
        (Some(55), Via::Bluetooth),
        (Some(12), Via::Bluetooth),
        (None, Via::Cabo),
    ];

    let escala = 8u32;
    let largura = tamanho * escala * exemplos.len() as u32;
    let mut tira = tiny_skia::Pixmap::new(largura, tamanho * escala)?;

    tira.fill(if fundo_claro {
        tiny_skia::Color::from_rgba8(0xF3, 0xF3, 0xF3, 255)
    } else {
        tiny_skia::Color::from_rgba8(0x20, 0x20, 0x20, 255)
    });

    let pintura = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Nearest,
        ..Default::default()
    };

    for (i, (preenchimento, modo)) in exemplos.iter().enumerate() {
        let svg = montar_svg(&estado_demo(*preenchimento, *modo), Limiares::PADRAO);
        let arvore = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;

        let mut um = tiny_skia::Pixmap::new(tamanho, tamanho)?;
        let e = tamanho as f32 / g::CAIXA;
        resvg::render(&arvore, tiny_skia::Transform::from_scale(e, e), &mut um.as_mut());

        tira.draw_pixmap(
            (i as u32 * tamanho) as i32,
            0,
            um.as_ref(),
            &pintura,
            tiny_skia::Transform::from_scale(escala as f32, escala as f32),
            None,
        );
    }

    tira.save_png(caminho).ok()
}

pub fn svg_do_app(tamanho: u32) -> String {
    let caixa = g::CAIXA;
    let centro = caixa / 2.0;
    let raio = g::APP_ANEL_RAIO;
    let grossura = g::APP_ANEL_LARGURA;

    let borda = if tamanho >= 32 {
        format!(
            r##"<circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="{}" stroke-opacity="{}" stroke-width="{}"/>"##,
            centro - g::BORDA_LARGURA / 2.0,
            g::BRANCO,
            g::BORDA_OPACIDADE,
            g::BORDA_LARGURA
        )
    } else {
        String::new()
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {caixa} {caixa}" width="{caixa}" height="{caixa}">
<defs><linearGradient id="marca" x1="{g1}" y1="{g2}" x2="{g3}" y2="{g4}" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="{verde}"/><stop offset="1" stop-color="{teal}"/></linearGradient></defs>
<circle cx="{centro}" cy="{centro}" r="{centro}" fill="{fundo}"/>
{borda}
<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{branco}" stroke-opacity="{trilho}" stroke-width="{grossura}"/>
<path d="{arco}" fill="none" stroke="url(#marca)" stroke-width="{grossura}" stroke-linecap="round"/>
<g transform="translate({centro},{pad_y}) scale({escala}) translate({desloca_x},{desloca_y})"><path d="{pad}" fill="{claro}"/><circle cx="{ex}" cy="{ey}" r="{sr}" fill="{fundo}"/><circle cx="{dx}" cy="{dy}" r="{sr}" fill="{fundo}"/></g>
</svg>"##,
        verde = g::VERDE,
        teal = g::TEAL,
        branco = g::BRANCO,
        claro = g::GLIFO_CLARO,
        trilho = g::APP_TRILHO_OPACIDADE,
        g1 = g::GRADIENTE.0,
        g2 = g::GRADIENTE.1,
        g3 = g::GRADIENTE.2,
        g4 = g::GRADIENTE.3,
        fundo = g::FUNDO,
        arco = g::arco_em(g::APP_ANEL_VARREDURA, raio),
        pad_y = g::APP_PAD_CENTRO_Y,
        escala = g::APP_PAD_ESCALA,
        desloca_x = -centro,
        desloca_y = -g::PAD_CENTRO_Y,
        pad = g::PAD,
        ex = g::APP_STICK_ESQ.0,
        ey = g::APP_STICK_ESQ.1,
        dx = g::APP_STICK_DIR.0,
        dy = g::APP_STICK_DIR.1,
        sr = g::APP_STICK_RAIO,
    )
}

pub fn icone_do_app(tamanho: u32) -> Option<Image<'static>> {
    rasterizar(&svg_do_app(tamanho), tamanho)
}

pub fn tamanho_do_icone_grande() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXICON};
    let medido = unsafe { GetSystemMetrics(SM_CXICON) };
    if medido <= 0 {
        32
    } else {
        medido as u32
    }
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

pub fn tamanho_do_icone() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};
    let medido = unsafe { GetSystemMetrics(SM_CXSMICON) };
    if medido <= 0 {
        16
    } else {
        medido as u32
    }
}
