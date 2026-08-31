use std::str::FromStr;
use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use crate::configuracoes::Settings;
use crate::janelas;
use crate::Compartilhado;

enum Acao {
    MostrarOuEsconder,
    SoltarOuPrender,
}

pub fn combinacao(texto: &str) -> Option<Shortcut> {
    let combinacao = Shortcut::from_str(texto.trim()).ok()?;
    if combinacao.mods.is_empty() {
        return None;
    }
    Some(combinacao)
}

pub fn sanear(texto: &str, padrao: &str) -> String {
    match combinacao(texto) {
        Some(_) => texto.trim().to_string(),
        None => padrao.to_string(),
    }
}

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, atalho, evento| {
            if evento.state() != ShortcutState::Pressed {
                return;
            }
            match acao(app, atalho) {
                Some(Acao::SoltarOuPrender) => alternar_ajuste(app),
                Some(Acao::MostrarOuEsconder) => alternar(app),
                None => {}
            }
        })
        .build()
}

pub fn aplicar(app: &AppHandle, cfg: &Settings) -> Vec<String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gerenciador = app.global_shortcut();
    let _ = gerenciador.unregister_all();

    if !cfg.overlay_shortcut_enabled {
        return Vec::new();
    }

    let mut recusados = Vec::new();
    for texto in [&cfg.overlay_shortcut, &cfg.overlay_move_shortcut] {
        let registrou = combinacao(texto).is_some_and(|c| gerenciador.register(c).is_ok());
        if !registrou {
            recusados.push(texto.clone());
        }
    }
    recusados
}

fn acao(app: &AppHandle, atalho: &Shortcut) -> Option<Acao> {
    let compartilhado = app.try_state::<Arc<Compartilhado>>()?;
    let cfg = compartilhado.config.lock().unwrap().clone();

    if combinacao(&cfg.overlay_move_shortcut).as_ref() == Some(atalho) {
        return Some(Acao::SoltarOuPrender);
    }
    if combinacao(&cfg.overlay_shortcut).as_ref() == Some(atalho) {
        return Some(Acao::MostrarOuEsconder);
    }
    None
}

fn alternar(app: &AppHandle) {
    let Some(compartilhado) = app.try_state::<Arc<Compartilhado>>() else {
        return;
    };

    let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) else {
        return;
    };

    let solta = *compartilhado.sobreposicao_solta.lock().unwrap();
    if solta {
        crate::soltar_sobreposicao(app, false);
    }

    let visivel = janela.is_visible().unwrap_or(false);
    let alvo = !visivel;

    *compartilhado.sobreposicao_a_mao.lock().unwrap() = Some(alvo);

    if alvo {
        let _ = janela.show();
    } else {
        let _ = janela.hide();
    }
}

fn alternar_ajuste(app: &AppHandle) {
    let Some(compartilhado) = app.try_state::<Arc<Compartilhado>>() else {
        return;
    };

    let solta = *compartilhado.sobreposicao_solta.lock().unwrap();
    crate::soltar_sobreposicao(app, !solta);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn combinacao_sem_modificador_nao_serve_como_atalho_global() {
        assert!(combinacao("KeyK").is_none());
        assert!(combinacao("F5").is_none());
    }

    #[test]
    fn o_que_o_teclado_do_navegador_manda_e_entendido() {
        assert!(combinacao("Ctrl+Shift+KeyK").is_some());
        assert!(combinacao("Ctrl+Alt+Digit4").is_some());
        assert!(combinacao("Shift+Alt+ArrowUp").is_some());
    }

    #[test]
    fn texto_sem_sentido_nao_vira_atalho() {
        assert!(combinacao("Ctrl+Banana").is_none());
        assert!(combinacao("").is_none());
    }
}
