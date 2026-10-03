use std::str::FromStr;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use crate::configuracoes::Settings;
use crate::janelas;
use crate::Compartilhado;

enum Acao {
    MostrarOuEsconder,
    SoltarOuPrender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Atalho {
    Mostrar,
    Mover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Motivo {
    Invalida,
    EmUso,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recusa {
    pub atalho: Atalho,
    pub combinacao: String,
    pub motivo: Motivo,
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

pub fn pausar(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app.global_shortcut().unregister_all();
}

pub fn aplicar(app: &AppHandle, cfg: &Settings) -> Vec<Recusa> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gerenciador = app.global_shortcut();
    let _ = gerenciador.unregister_all();

    if !cfg.overlay_shortcut_enabled {
        return Vec::new();
    }

    let mut recusas = Vec::new();
    for (atalho, texto) in
        [(Atalho::Mostrar, &cfg.overlay_shortcut), (Atalho::Mover, &cfg.overlay_move_shortcut)]
    {
        let motivo = match combinacao(texto) {
            None => Some(Motivo::Invalida),
            Some(c) if gerenciador.register(c).is_err() => Some(Motivo::EmUso),
            Some(_) => None,
        };
        if let Some(motivo) = motivo {
            recusas.push(Recusa { atalho, combinacao: texto.clone(), motivo });
        }
    }
    recusas
}

pub fn aplicar_sem_perder_o_anterior(
    app: &AppHandle,
    novas: &mut Settings,
    anterior: &Settings,
) -> Vec<Recusa> {
    let mut recusas = aplicar(app, novas);

    let mut voltou = false;
    for recusa in &recusas {
        let (campo, antes) = match recusa.atalho {
            Atalho::Mostrar => (&mut novas.overlay_shortcut, &anterior.overlay_shortcut),
            Atalho::Mover => (&mut novas.overlay_move_shortcut, &anterior.overlay_move_shortcut),
        };
        if *campo != *antes {
            campo.clone_from(antes);
            voltou = true;
        }
    }

    if voltou {
        for recusa in aplicar(app, novas) {
            if !recusas.iter().any(|r| r.atalho == recusa.atalho) {
                recusas.push(recusa);
            }
        }
    }
    recusas
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

pub(crate) fn alternar(app: &AppHandle) {
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
        janelas::mostrar_por_cima(&janela);
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
