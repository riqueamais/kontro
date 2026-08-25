use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use crate::janelas;
use crate::Compartilhado;

fn atalho() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK)
}

pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _atalho, evento| {
            if evento.state() != ShortcutState::Pressed {
                return;
            }
            alternar(app);
        })
        .build()
}

pub fn aplicar<R: tauri::Runtime>(app: &AppHandle<R>, ligado: bool) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gerenciador = app.global_shortcut();
    let combinacao = atalho();

    if ligado {
        if !gerenciador.is_registered(combinacao) {
            let _ = gerenciador.register(combinacao);
        }
    } else if gerenciador.is_registered(combinacao) {
        let _ = gerenciador.unregister(combinacao);
    }
}

fn alternar<R: tauri::Runtime>(app: &AppHandle<R>) {
    let Some(compartilhado) = app.try_state::<Arc<Compartilhado>>() else {
        return;
    };

    let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) else {
        return;
    };

    let visivel = janela.is_visible().unwrap_or(false);
    let alvo = !visivel;

    *compartilhado.sobreposicao_a_mao.lock().unwrap() = Some(alvo);

    if alvo {
        let _ = janela.show();
    } else {
        let _ = janela.hide();
    }
}
