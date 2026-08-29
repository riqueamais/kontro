use std::io;
use std::path::Path;

use tiny_skia::Pixmap;

use crate::geometria as g;
use crate::model::LinkMode;
use crate::settings::Limiares;
use crate::tray;

const TAMANHOS_ICO: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 96, 128, 256];
const TAMANHOS_FAVICON: &[u32] = &[16, 24, 32, 48, 64, 128, 256];
const MAIOR_TAMANHO_EM_BITMAP: u32 = 96;

pub fn gerar(raiz: &str) -> io::Result<()> {
    let raiz = Path::new(raiz);

    icones_do_app(&raiz.join("src-tauri/icons"))?;
    icones_do_site(&raiz.join("docs"))?;
    icone_do_instalador(&raiz.join("Assets/branding/setup.ico"))?;
    favicon_do_front(&raiz.join("public/favicon.svg"))?;
    vetores_de_referencia(&raiz.join("Assets/svg"))?;

    Ok(())
}

fn icones_do_app(pasta: &Path) -> io::Result<()> {
    std::fs::create_dir_all(pasta)?;

    let arquivos: &[(&str, u32)] = &[
        ("16x16.png", 16),
        ("24x24.png", 24),
        ("32x32.png", 32),
        ("48x48.png", 48),
        ("64x64.png", 64),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
        ("256x256.png", 256),
        ("icon.png", 512),
    ];

    for (nome, tamanho) in arquivos {
        gravar_png(&pasta.join(nome), *tamanho)?;
    }

    gravar_ico(&pasta.join("icon.ico"), TAMANHOS_ICO)
}

fn icones_do_site(pasta: &Path) -> io::Result<()> {
    if !pasta.is_dir() {
        return Ok(());
    }

    for tamanho in [32u32, 128, 256, 512] {
        gravar_png(&pasta.join(format!("icon-{tamanho}.png")), tamanho)?;
    }

    gravar_ico(&pasta.join("favicon.ico"), TAMANHOS_FAVICON)
}

fn icone_do_instalador(caminho: &Path) -> io::Result<()> {
    criar_pasta_do_arquivo(caminho)?;
    gravar_ico(caminho, TAMANHOS_ICO)
}

fn favicon_do_front(caminho: &Path) -> io::Result<()> {
    criar_pasta_do_arquivo(caminho)?;
    std::fs::write(caminho, tray::svg_do_app(256))
}

fn vetores_de_referencia(pasta: &Path) -> io::Result<()> {
    std::fs::create_dir_all(pasta)?;

    std::fs::write(pasta.join("app-icon.svg"), tray::svg_do_app(512))?;

    let estados: &[(&str, Option<i32>, LinkMode)] = &[
        ("tray-off.svg", None, LinkMode::Offline),
        ("tray-cable.svg", None, LinkMode::Cable),
        ("tray-level-72.svg", Some(72), LinkMode::Bluetooth),
    ];

    for (nome, preenchimento, modo) in estados {
        let estado = tray::estado_demo(*preenchimento, *modo);
        std::fs::write(pasta.join(nome), tray::montar_svg(&estado, Limiares::PADRAO))?;
    }

    Ok(())
}

fn criar_pasta_do_arquivo(caminho: &Path) -> io::Result<()> {
    match caminho.parent() {
        Some(pai) => std::fs::create_dir_all(pai),
        None => Ok(()),
    }
}

fn gravar_png(caminho: &Path, tamanho: u32) -> io::Result<()> {
    let mapa = desenhar(tamanho)?;
    let bytes = mapa.encode_png().map_err(io::Error::other)?;
    std::fs::write(caminho, bytes)
}

fn gravar_ico(caminho: &Path, tamanhos: &[u32]) -> io::Result<()> {
    let mut quadros = Vec::with_capacity(tamanhos.len());

    for &tamanho in tamanhos {
        let mapa = desenhar(tamanho)?;
        let bytes = if tamanho > MAIOR_TAMANHO_EM_BITMAP {
            mapa.encode_png().map_err(io::Error::other)?
        } else {
            quadro_em_bitmap(&mapa)
        };
        quadros.push((tamanho, bytes));
    }

    std::fs::write(caminho, montar_ico(&quadros))
}

fn desenhar(tamanho: u32) -> io::Result<Pixmap> {
    let svg = tray::svg_do_app(tamanho);
    let arvore = usvg::Tree::from_str(&svg, &usvg::Options::default())
        .map_err(|e| io::Error::other(format!("svg invalido em {tamanho}px: {e}")))?;

    let mut mapa = Pixmap::new(tamanho, tamanho)
        .ok_or_else(|| io::Error::other(format!("nao consegui alocar {tamanho}x{tamanho}")))?;

    let escala = tamanho as f32 / g::CAIXA;
    resvg::render(
        &arvore,
        tiny_skia::Transform::from_scale(escala, escala),
        &mut mapa.as_mut(),
    );

    Ok(mapa)
}

fn bytes_por_linha_da_mascara(largura: u32) -> usize {
    (largura as usize).div_ceil(32) * 4
}

fn quadro_em_bitmap(mapa: &Pixmap) -> Vec<u8> {
    let (largura, altura) = (mapa.width(), mapa.height());
    let linha_da_mascara = bytes_por_linha_da_mascara(largura);

    let mut saida = Vec::with_capacity(
        40 + (largura * altura * 4) as usize + linha_da_mascara * altura as usize,
    );

    saida.extend_from_slice(&40u32.to_le_bytes());
    saida.extend_from_slice(&(largura as i32).to_le_bytes());
    saida.extend_from_slice(&((altura * 2) as i32).to_le_bytes());
    saida.extend_from_slice(&1u16.to_le_bytes());
    saida.extend_from_slice(&32u16.to_le_bytes());
    saida.extend_from_slice(&0u32.to_le_bytes());
    saida.extend_from_slice(&0u32.to_le_bytes());
    saida.extend_from_slice(&0i32.to_le_bytes());
    saida.extend_from_slice(&0i32.to_le_bytes());
    saida.extend_from_slice(&0u32.to_le_bytes());
    saida.extend_from_slice(&0u32.to_le_bytes());

    let pixels = mapa.pixels();

    for y in (0..altura).rev() {
        for x in 0..largura {
            let cor = pixels[(y * largura + x) as usize].demultiply();
            saida.push(cor.blue());
            saida.push(cor.green());
            saida.push(cor.red());
            saida.push(cor.alpha());
        }
    }

    for y in (0..altura).rev() {
        let mut linha = vec![0u8; linha_da_mascara];
        for x in 0..largura {
            if pixels[(y * largura + x) as usize].alpha() == 0 {
                linha[(x / 8) as usize] |= 0x80 >> (x % 8);
            }
        }
        saida.extend_from_slice(&linha);
    }

    saida
}

fn montar_ico(quadros: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let tamanho_do_cabecalho = 6 + 16 * quadros.len();
    let mut cabecalho = Vec::with_capacity(tamanho_do_cabecalho);
    let mut corpo = Vec::new();

    cabecalho.extend_from_slice(&0u16.to_le_bytes());
    cabecalho.extend_from_slice(&1u16.to_le_bytes());
    cabecalho.extend_from_slice(&(quadros.len() as u16).to_le_bytes());

    for (tamanho, bytes) in quadros {
        let medida = if *tamanho >= 256 { 0u8 } else { *tamanho as u8 };

        cabecalho.push(medida);
        cabecalho.push(medida);
        cabecalho.push(0);
        cabecalho.push(0);
        cabecalho.extend_from_slice(&1u16.to_le_bytes());
        cabecalho.extend_from_slice(&32u16.to_le_bytes());
        cabecalho.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        cabecalho.extend_from_slice(&((tamanho_do_cabecalho + corpo.len()) as u32).to_le_bytes());

        corpo.extend_from_slice(bytes);
    }

    cabecalho.extend_from_slice(&corpo);
    cabecalho
}
