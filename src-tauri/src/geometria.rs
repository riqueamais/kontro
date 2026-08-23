//! A geometria e as cores do sistema "Anel", no espaco canonico de 512 x 512.
//!
//! Sao os mesmos numeros da versao anterior: mexer aqui muda o icone da bandeja, a
//! sobreposicao e o aviso de uma vez so. Os analogicos sao vazados, nao pintados --
//! sem os recortes a silhueta vira uma mancha que so lembra um controle.

pub const CAIXA: f32 = 512.0;

pub const PAD: &str = "M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 \
366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 372C146 400 126 418 \
98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z";

pub const STICK_RAIO: f32 = 48.3;
pub const STICK_ESQ: (f32, f32) = (129.3, 228.0);
pub const STICK_DIR: (f32, f32) = (382.7, 228.0);

/// Centro vertical do proprio desenho do controle, para posiciona-lo sem chute.
pub const PAD_CENTRO_Y: f32 = 288.0;

pub const ANEL_RAIO: f32 = 194.0;
pub const ANEL_LARGURA: f32 = 56.0;
pub const PAD_ESCALA_BANDEJA: f32 = 0.5;
pub const PAD_CENTRO_Y_BANDEJA: f32 = 268.0;

pub const VERDE: &str = "#5FE083";
pub const TEAL: &str = "#35D7A8";
pub const AMBAR: &str = "#F2C14E";
pub const VERMELHO: &str = "#F2564E";
pub const CINZA: &str = "#8D979F";
pub const GLIFO_NO_CLARO: &str = "#1B1F24";

/// Cor do anel para um percentual, pelos limiares que o usuario configurou.
pub fn cor_do_nivel(percent: i32, vermelho_abaixo: i32, ambar_abaixo: i32) -> &'static str {
    if percent < vermelho_abaixo {
        VERMELHO
    } else if percent < ambar_abaixo {
        AMBAR
    } else {
        VERDE
    }
}

/// A silhueta com os analogicos vazados, como um caminho unico de regra par-impar.
pub fn pad_com_sticks_vazados() -> String {
    format!("{PAD} {} {}", circulo(STICK_ESQ), circulo(STICK_DIR))
}

/// Circulo como subcaminho, para poder viver dentro do mesmo path do controle.
fn circulo((cx, cy): (f32, f32)) -> String {
    let r = STICK_RAIO;
    format!(
        "M{} {} a{r} {r} 0 1 0 {} 0 a{r} {r} 0 1 0 -{} 0 Z",
        cx - r,
        cy,
        r * 2.0,
        r * 2.0
    )
}

/// Arco do anel comecando no topo, em graus. Devolve o atributo `d` de um path.
pub fn arco(varredura: f32) -> String {
    let centro = CAIXA / 2.0;
    let r = ANEL_RAIO;
    let varredura = varredura.clamp(0.0, 359.9);

    let ponto = |graus: f32| {
        let rad = graus.to_radians();
        (centro + r * rad.cos(), centro + r * rad.sin())
    };

    let (x0, y0) = ponto(-90.0);
    let (x1, y1) = ponto(-90.0 + varredura);
    let maior = if varredura > 180.0 { 1 } else { 0 };
    format!("M{x0} {y0} A{r} {r} 0 {maior} 1 {x1} {y1}")
}
