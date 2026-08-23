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

fn montar_svg(estado: &BatteryState, _tema_claro: bool) -> String {
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
pub fn salvar_previa(caminho: &str, tamanho: u32, tema_claro: bool) -> Option<()> {
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
    tira.fill(if tema_claro {
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

        let svg = montar_svg(&estado, tema_claro);
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
