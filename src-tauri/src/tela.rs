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

/// Tela cheia exclusiva: nenhuma janela comum e desenhada por cima.
///
/// Saber disso permite explicar ao usuario por que a sobreposicao nao aparece, em vez de
/// deixa-lo achando que quebrou.
pub fn em_tela_cheia_exclusiva() -> bool {
    consultar() == Some(QUNS_RUNNING_D3D_FULL_SCREEN)
}

fn consultar() -> Option<QUERY_USER_NOTIFICATION_STATE> {
    unsafe { SHQueryUserNotificationState().ok() }
}
