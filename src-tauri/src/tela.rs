use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUERY_USER_NOTIFICATION_STATE, QUNS_BUSY,
    QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};

pub fn em_tela_cheia() -> bool {
    let Some(estado) = consultar() else { return false };
    estado == QUNS_RUNNING_D3D_FULL_SCREEN
        || estado == QUNS_BUSY
        || estado == QUNS_PRESENTATION_MODE
}

fn consultar() -> Option<QUERY_USER_NOTIFICATION_STATE> {
    unsafe { SHQueryUserNotificationState().ok() }
}

pub fn centro_da_janela_em_foco() -> Option<(f64, f64)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    unsafe {
        let janela = GetForegroundWindow();
        if janela.is_invalid() {
            return None;
        }
        let mut area = RECT::default();
        GetWindowRect(janela, &mut area).ok()?;
        if area.right <= area.left || area.bottom <= area.top {
            return None;
        }
        Some((
            (area.left + area.right) as f64 / 2.0,
            (area.top + area.bottom) as f64 / 2.0,
        ))
    }
}
