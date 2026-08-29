use std::io;
use std::path::Path;

use crate::configuracoes::{CAMPOS_DA_CONFIG, ENUMS_DA_CONFIG};
use crate::geometria as g;
use crate::icones;

pub fn tudo(raiz: &str) -> io::Result<()> {
    let raiz = Path::new(raiz);

    icones::gerar(raiz)?;
    escrever(&raiz.join("src/estilo/geometria.gerada.ts"), &modulo_de_geometria())?;
    escrever(&raiz.join("src/config.gerada.ts"), &modulo_de_config())?;

    Ok(())
}

fn escrever(caminho: &Path, conteudo: &str) -> io::Result<()> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai)?;
    }
    std::fs::write(caminho, conteudo)
}

fn modulo_de_config() -> String {
    let mut t = String::new();

    for (tipo, valores) in ENUMS_DA_CONFIG {
        let uniao = valores.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(" | ");
        t.push_str(&format!("export type {tipo} = {uniao};\n"));
    }

    t.push_str("\nexport interface Config {\n");
    for (nome, tipo) in CAMPOS_DA_CONFIG {
        t.push_str(&format!("  {nome}: {tipo};\n"));
    }
    t.push_str("}\n");

    t
}

fn modulo_de_geometria() -> String {
    format!(
        r#"export const CAIXA = {caixa};
export const PAD_CENTRO_Y = {pad_y};
export const GLIFO_CAIXA = "{glifo}";

export const PAD = "{pad}";

export const PAD_COM_STICKS_VAZADOS = "{vazado}";

export const CORES = {{
  fundo: "{fundo}",
  verde: "{verde}",
  teal: "{teal}",
  branco: "{branco}",
  glifoClaro: "{claro}",
}} as const;

export const APP = {{
  anelRaio: {app_raio},
  anelLargura: {app_largura},
  anelVarredura: {app_varredura},
  padEscala: {app_escala},
  padCentroY: {app_pad_y},
  stickRaio: {stick_r},
  stickEsq: [{stick_ex}, {stick_ey}],
  stickDir: [{stick_dx}, {stick_dy}],
  trilhoOpacidade: {app_trilho},
}} as const;

export const BANDEJA = {{
  anelRaio: {raio},
  anelLargura: {largura},
  padEscala: {escala},
  padCentroY: {pad_bandeja_y},
  trilhoOpacidade: {trilho},
}} as const;

export const GRADIENTE = {{ x1: {g1}, y1: {g2}, x2: {g3}, y2: {g4} }} as const;
"#,
        caixa = g::CAIXA,
        pad_y = g::PAD_CENTRO_Y,
        glifo = g::GLIFO_CAIXA,
        pad = g::PAD,
        vazado = g::pad_com_sticks_vazados(),
        fundo = g::FUNDO,
        verde = g::VERDE,
        teal = g::TEAL,
        branco = g::BRANCO,
        claro = g::GLIFO_CLARO,
        app_raio = g::APP_ANEL_RAIO,
        app_largura = g::APP_ANEL_LARGURA,
        app_varredura = g::APP_ANEL_VARREDURA,
        app_escala = g::APP_PAD_ESCALA,
        app_pad_y = g::APP_PAD_CENTRO_Y,
        stick_r = g::APP_STICK_RAIO,
        stick_ex = g::APP_STICK_ESQ.0,
        stick_ey = g::APP_STICK_ESQ.1,
        stick_dx = g::APP_STICK_DIR.0,
        stick_dy = g::APP_STICK_DIR.1,
        app_trilho = g::APP_TRILHO_OPACIDADE,
        raio = g::ANEL_RAIO,
        largura = g::ANEL_LARGURA,
        escala = g::PAD_ESCALA_BANDEJA,
        pad_bandeja_y = g::PAD_CENTRO_Y_BANDEJA,
        trilho = g::TRILHO_OPACIDADE,
        g1 = g::GRADIENTE.0,
        g2 = g::GRADIENTE.1,
        g3 = g::GRADIENTE.2,
        g4 = g::GRADIENTE.3,
    )
}
