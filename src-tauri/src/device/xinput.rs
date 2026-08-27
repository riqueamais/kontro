use windows::Win32::UI::Input::XboxController::{
    XInputGetBatteryInformation, XInputGetCapabilities, BATTERY_DEVTYPE_GAMEPAD,
    XINPUT_BATTERY_INFORMATION, XINPUT_CAPABILITIES, XINPUT_FLAG,
};

pub const TIPO_DESCONECTADO: u8 = 0;
pub const TIPO_COM_FIO: u8 = 1;
pub const TIPO_ALCALINA: u8 = 2;
pub const TIPO_RECARREGAVEL: u8 = 3;

const SUCESSO: u32 = 0;

fn conectado(slot: u32) -> bool {
    let mut caps = XINPUT_CAPABILITIES::default();
    unsafe { XInputGetCapabilities(slot, XINPUT_FLAG(0), &mut caps) == SUCESSO }
}

fn bateria(slot: u32) -> Option<(u8, u8)> {
    if !conectado(slot) {
        return None;
    }
    let mut info = XINPUT_BATTERY_INFORMATION::default();
    let ok = unsafe { XInputGetBatteryInformation(slot, BATTERY_DEVTYPE_GAMEPAD, &mut info) };
    if ok != SUCESSO {
        return None;
    }
    Some((info.BatteryType.0, info.BatteryLevel.0))
}

pub fn slots_conectados() -> Vec<u32> {
    (0u32..4).filter(|s| conectado(*s)).collect()
}

pub fn alguem_no_cabo() -> bool {
    (0u32..4).any(|s| matches!(bateria(s), Some((TIPO_COM_FIO, _))))
}

pub fn alguem_na_bateria() -> bool {
    (0u32..4).any(|s| matches!(bateria(s), Some((TIPO_ALCALINA | TIPO_RECARREGAVEL, _))))
}

pub fn carga_do_slot(slot: u32) -> Option<i32> {
    match bateria(slot) {
        Some((TIPO_DESCONECTADO | TIPO_COM_FIO, _)) | None => None,
        Some((_, nivel)) => Some(nivel as i32),
    }
}

pub fn carga_de_qualquer_slot() -> Option<i32> {
    (0u32..4).find_map(carga_do_slot)
}

pub fn descrever() -> Vec<String> {
    let tipos = ["desconectado", "com fio", "alcalina", "recarregavel"];
    let niveis = ["vazia", "baixa", "media", "cheia"];

    (0u32..4)
        .filter_map(|slot| {
            let (tipo, nivel) = bateria(slot)?;
            let t = tipos.get(tipo as usize).copied().unwrap_or("desconhecido");
            let n = niveis.get(nivel as usize).copied().unwrap_or("desconhecido");
            Some(format!("slot {slot}: {t} / {n}   (tipo={tipo} nivel={nivel} de 3)"))
        })
        .collect()
}
