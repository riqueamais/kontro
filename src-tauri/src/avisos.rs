use std::path::{Path, PathBuf};

use tauri::AppHandle;
use tauri_winrt_notification::{IconCrop, Toast};

use crate::bandeja;
use crate::caminhos;
use crate::configuracoes::Limiares;
use crate::modelo::EstadoDoControle;
use crate::sistema;

const TAMANHO: u32 = 96;

pub fn mostrar(app: &AppHandle, titulo: &str, corpo: &str, imagem: Option<PathBuf>) {
    let id = id_do_app(app);
    let mut aviso = Toast::new(&id).title(titulo).text1(corpo);
    if let Some(caminho) = imagem.as_deref() {
        aviso = aviso.icon(caminho, IconCrop::Circular, "");
    }
    if let Err(erro) = aviso.show() {
        eprintln!("aviso do Windows: {erro}");
    }
}

pub fn imagem_do_nivel(estado: &EstadoDoControle, limiares: Limiares) -> Option<PathBuf> {
    let claro = sistema::barra_clara();
    let pct = estado.percentual?;
    let nome = format!(
        "nivel-{pct}-{}-{}-{}.png",
        limiares.critico,
        limiares.aviso,
        if claro { "claro" } else { "escuro" }
    );
    gravar(&nome, || bandeja::montar_svg(estado, limiares, claro))
}

pub fn imagem_do_app() -> Option<PathBuf> {
    gravar("app.png", || bandeja::svg_do_app(TAMANHO))
}

fn gravar(nome: &str, desenhar: impl FnOnce() -> String) -> Option<PathBuf> {
    let pasta = caminhos::arquivo("avisos");
    let destino = pasta.join(nome);
    if destino.exists() {
        return Some(destino);
    }
    std::fs::create_dir_all(&pasta).ok()?;
    let mapa = bandeja::pixmap(&desenhar(), TAMANHO)?;
    mapa.save_png(&destino).ok()?;
    Some(destino)
}

fn id_do_app(app: &AppHandle) -> String {
    let em_desenvolvimento = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .is_some_and(|pasta| {
            pasta.ends_with(r"target\debug") || pasta.ends_with(r"target\release")
        });
    if em_desenvolvimento {
        Toast::POWERSHELL_APP_ID.to_string()
    } else {
        app.config().identifier.clone()
    }
}
