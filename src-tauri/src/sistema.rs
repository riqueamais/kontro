use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

use crate::registro;

const PERSONALIZACAO: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const VERSAO: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

const PRIMEIRO_BUILD_COM_MATERIAL: u32 = 22000;

pub fn material_disponivel() -> bool {
    build() >= PRIMEIRO_BUILD_COM_MATERIAL && transparencia_ligada()
}

pub fn windows_no_claro() -> bool {
    registro::numero(HKEY_CURRENT_USER, PERSONALIZACAO, "AppsUseLightTheme")
        .map(|v| v != 0)
        .unwrap_or(false)
}

pub fn transparencia_ligada() -> bool {
    registro::numero(HKEY_CURRENT_USER, PERSONALIZACAO, "EnableTransparency")
        .map(|v| v != 0)
        .unwrap_or(true)
}

fn build() -> u32 {
    registro::texto(HKEY_LOCAL_MACHINE, VERSAO, "CurrentBuildNumber")
        .and_then(|t| t.trim().parse().ok())
        .unwrap_or(0)
}
