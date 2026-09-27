use std::sync::{mpsc, OnceLock};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};

use crate::Pedido;

static ENVIO: OnceLock<mpsc::Sender<Pedido>> = OnceLock::new();

pub fn vigiar(envio: mpsc::Sender<Pedido>) {
    if ENVIO.set(envio).is_err() {
        return;
    }

    std::thread::spawn(|| unsafe {
        let gancho = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(ao_mudar),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if gancho.is_invalid() {
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    });
}

unsafe extern "system" fn ao_mudar(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if let Some(envio) = ENVIO.get() {
        let _ = envio.send(Pedido::PrimeiroPlanoMudou);
    }
}
