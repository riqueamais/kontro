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
/// Nao recebe mais o tema da barra: desde que o icone ganhou disco de fundo, o controle
/// e sempre branco e o desenho vale igual na barra clara e na escura.
pub fn desenhar(estado: &BatteryState, tamanho: u32) -> Option<Image<'static>> {
    rasterizar(&montar_svg(estado), tamanho)
}

fn montar_svg(estado: &BatteryState) -> String {
    let caixa = g::CAIXA;
    let centro = caixa / 2.0;
    let raio = g::ANEL_RAIO;
    let grossura = g::ANEL_LARGURA;

    // O icone tem disco de fundo, como o icone do app.
    //
    // O sistema de design pede a marca sem fundo na bandeja, com o argumento de que a
    // barra e o fundo. Na barra do Windows 11 isso nao se sustenta: todos os vizinhos
    // sao icones solidos, e um anel fino e transparente no meio deles some -- ainda mais
    // com a barra translucida deixando o papel de parede atravessar.
    //
    // O fundo tambem resolve o tema de um jeito que a transparencia nao resolvia: sobre
    // disco escuro o controle e sempre branco, seja a barra clara ou escura, e para de
    // depender de acertar qual tema o usuario esta usando.
    let cor_glifo = "#FFFFFF";

    let fundo = format!(
        r##"<circle cx="{centro}" cy="{centro}" r="{}" fill="{}"/><circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="#FFFFFF" stroke-opacity="0.10" stroke-width="8"/>"##,
        g::FUNDO_RAIO,
        g::FUNDO,
        g::FUNDO_RAIO - 4.0
    );

    let trilho = format!(
        r##"<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{cor_glifo}" stroke-opacity="0.22" stroke-width="{grossura}"/>"##
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

    let miolo = match (estado.mode, estado.preenchimento) {
        // Desconectado: o controle apagado com o risco por cima. O trilho fica, porque e
        // ele que mantem a silhueta circular da marca.
        (LinkMode::Offline, _) => format!(
            r##"{trilho}{}<path d="M120 392 L392 120" stroke="{cor_glifo}" stroke-width="46" stroke-linecap="round"/>"##,
            desenhar_glifo(0.7)
        ),

        // No cabo o anel fica inteiro e neutro: "estou plugado e nao tenho numero".
        (LinkMode::Cable, None) => format!(
            r##"{trilho}<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="{}" stroke-width="{grossura}"/>{}"##,
            g::CINZA,
            desenhar_glifo(1.0)
        ),

        (_, Some(p)) => {
            let cor = g::cor_do_nivel(p, VERMELHO_ABAIXO, AMBAR_ABAIXO);
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

/// Salva em disco exatamente o que a bandeja receberia.
///
/// Um icone de dezesseis pixels e pequeno demais para julgar na tela. Ampliado sobre o
/// fundo real da barra de tarefas da para ver o que importa: se o anel sumiu, se o
/// controle esta torto, se o contraste morreu.
pub fn salvar_previa(caminho: &str, tamanho: u32, fundo_claro: bool) -> Option<()> {
    let exemplos: [(Option<i32>, LinkMode); 5] = [
        (None, LinkMode::Offline),
        (Some(100), LinkMode::Bluetooth),
        (Some(55), LinkMode::Bluetooth),
        (Some(12), LinkMode::Bluetooth),
        (None, LinkMode::Cable),
    ];

    let escala = 8u32;
    let largura = tamanho * escala * exemplos.len() as u32;
    let mut tira = tiny_skia::Pixmap::new(largura, tamanho * escala)?;

    // O icone e transparente e vive sobre a barra de tarefas. Julgar num fundo branco
    // esconde justamente o que o usuario ve: branco sobre branco some, e translucido
    // sobre escuro tambem.
    tira.fill(if fundo_claro {
        tiny_skia::Color::from_rgba8(0xF3, 0xF3, 0xF3, 255)
    } else {
        tiny_skia::Color::from_rgba8(0x20, 0x20, 0x20, 255)
    });

    // Nearest de proposito: interpolar suavizaria os pixels e esconderia exatamente o
    // defeito que se quer enxergar.
    let pintura = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Nearest,
        ..Default::default()
    };

    for (i, (preenchimento, modo)) in exemplos.iter().enumerate() {
        let estado = crate::model::BatteryState::montar(
            *modo,
            *preenchimento,
            if preenchimento.is_some() {
                crate::model::Precisao::Exata
            } else {
                crate::model::Precisao::Nenhuma
            },
            None,
            None,
            false,
            false,
            "demo".into(),
            None,
            "demo".into(),
            1,
            None,
        );

        let svg = montar_svg(&estado);
        let arvore = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;

        let mut um = tiny_skia::Pixmap::new(tamanho, tamanho)?;
        let e = tamanho as f32 / g::CAIXA;
        resvg::render(&arvore, tiny_skia::Transform::from_scale(e, e), &mut um.as_mut());

        // o deslocamento passa pela mesma escala do desenho, entao ele vai em pixels
        // do icone e nao da tira
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

/// O icone do app, desenhado no tamanho pedido.
///
/// Nao e o icone da bandeja: aqui o anel nao mede carga nenhuma, ele e a marca. O que os
/// dois compartilham e a silhueta do controle e a paleta, para que o app na barra de
/// tarefas e o icone ao lado do relogio sejam reconhecidamente a mesma coisa.
///
/// Cada tamanho e desenhado do zero em vez de reduzido do maior: reduzir um bitmap de
/// 256 para 16 borra o anel ate ele virar um halo, que e o aspecto de "baixa resolucao"
/// que aparece na barra de tarefas.
pub fn svg_do_app(tamanho: u32) -> String {
    let caixa = g::CAIXA;
    let centro = caixa / 2.0;
    let raio = g::APP_ANEL_RAIO;
    let grossura = g::APP_ANEL_LARGURA;

    // Abaixo de 32 pixels a borda de um pixel nao chega a ser borda: vira um cinza que
    // engorda a silhueta e come o contraste do anel.
    let borda = if tamanho >= 32 {
        format!(
            r##"<circle cx="{centro}" cy="{centro}" r="{}" fill="none" stroke="#FFFFFF" stroke-opacity="0.10" stroke-width="8"/>"##,
            centro - 4.0
        )
    } else {
        String::new()
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {caixa} {caixa}" width="{caixa}" height="{caixa}">
<defs><linearGradient id="marca" x1="120" y1="80" x2="400" y2="440" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="{verde}"/><stop offset="1" stop-color="#35D7A8"/></linearGradient></defs>
<circle cx="{centro}" cy="{centro}" r="{centro}" fill="{fundo}"/>
{borda}
<circle cx="{centro}" cy="{centro}" r="{raio}" fill="none" stroke="#FFFFFF" stroke-opacity="0.13" stroke-width="{grossura}"/>
<path d="{arco}" fill="none" stroke="url(#marca)" stroke-width="{grossura}" stroke-linecap="round"/>
<g transform="translate({centro},{pad_y}) scale({escala}) translate({desloca_x},{desloca_y})"><path d="{pad}" fill="#F4F7F9"/><circle cx="{ex}" cy="{ey}" r="{sr}" fill="{fundo}"/><circle cx="{dx}" cy="{dy}" r="{sr}" fill="{fundo}"/></g>
</svg>"##,
        verde = g::VERDE,
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

/// O icone do app pronto para o Windows, no tamanho pedido.
pub fn icone_do_app(tamanho: u32) -> Option<Image<'static>> {
    rasterizar(&svg_do_app(tamanho), tamanho)
}

/// Tamanho do icone grande de janela, ja com a escala da tela.
///
/// E o que a barra de tarefas e o Alt+Tab pedem. O icone embutido no pacote tem 32
/// pixels; em telas a 125% ou 150% o sistema precisa de 40 ou 48 e amplia aqueles 32 --
/// e ampliar e o que borra.
pub fn tamanho_do_icone_grande() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXICON};
    let medido = unsafe { GetSystemMetrics(SM_CXICON) };
    if medido <= 0 {
        32
    } else {
        medido as u32
    }
}

/// Grava o icone do app em PNG, um arquivo por tamanho.
pub fn salvar_icones(pasta: &str, tamanhos: &[u32]) -> std::io::Result<()> {
    std::fs::create_dir_all(pasta)?;
    for &t in tamanhos {
        let svg = svg_do_app(t);
        let Some(arvore) = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok() else {
            continue;
        };
        let Some(mut mapa) = tiny_skia::Pixmap::new(t, t) else { continue };
        let e = t as f32 / g::CAIXA;
        resvg::render(&arvore, tiny_skia::Transform::from_scale(e, e), &mut mapa.as_mut());
        mapa.save_png(format!("{pasta}/{t}.png"))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    }
    Ok(())
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
