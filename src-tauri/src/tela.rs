//! O que esta ocupando a tela agora.
//!
//! Nao existe API para "o usuario esta jogando", e vasculhar nomes de processo seria uma
//! lista infinita e sempre desatualizada. O que da para saber com precisao e se algo
//! tomou a tela inteira -- que, na pratica, e quando o usuario quer o dado sobreposto.

use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUERY_USER_NOTIFICATION_STATE, QUNS_BUSY,
    QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};

/// Ha algo ocupando a tela inteira, seja jogo ou apresentacao.
pub fn em_tela_cheia() -> bool {
    let Some(estado) = consultar() else { return false };
    // QUNS_BUSY e o caso comum hoje: jogo em tela cheia sem moldura.
    estado == QUNS_RUNNING_D3D_FULL_SCREEN
        || estado == QUNS_BUSY
        || estado == QUNS_PRESENTATION_MODE
}

// Nao ha funcao para "tela cheia exclusiva" aqui de proposito. O estado que o Windows
// relata nao distingue jogo em tela cheia exclusiva de janela sem moldura ocupando a
// tela: ambos chegam como o mesmo valor. Avisar o usuario com base nisso erraria na
// metade dos casos, e um aviso que erra e pior que a duvida.

fn consultar() -> Option<QUERY_USER_NOTIFICATION_STATE> {
    unsafe { SHQueryUserNotificationState().ok() }
}
