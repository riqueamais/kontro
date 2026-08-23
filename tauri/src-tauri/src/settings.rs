//! Preferencias do usuario.
//!
//! Os nomes dos campos sao os mesmos do arquivo da versao em .NET -- por isso o
//! PascalCase -- para que quem atualiza nao perca o que ja tinha configurado.

use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseAction {
    MinimizeToTray,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayMode {
    Desligada,
    EmJogo,
    Sempre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayCorner {
    SuperiorEsquerdo,
    SuperiorDireito,
    InferiorEsquerdo,
    InferiorDireito,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Settings {
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub close_action: CloseAction,
    pub notifications_enabled: bool,
    pub warn_threshold: i32,
    pub critical_threshold: i32,
    pub connect_toast_enabled: bool,
    pub overlay_mode: OverlayMode,
    pub overlay_tip_shown: bool,
    pub overlay_corner: OverlayCorner,
    /// -1 significa acompanhar a tela em foco.
    pub overlay_monitor: i32,
    pub overlay_scale: f64,
    pub overlay_opacity: f64,
    pub auto_check_updates: bool,
    pub first_run_done: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            start_with_windows: false,
            start_minimized: true,
            close_action: CloseAction::MinimizeToTray,
            notifications_enabled: true,
            warn_threshold: 20,
            critical_threshold: 10,
            connect_toast_enabled: true,
            overlay_mode: OverlayMode::EmJogo,
            overlay_tip_shown: false,
            overlay_corner: OverlayCorner::InferiorDireito,
            overlay_monitor: -1,
            overlay_scale: 1.0,
            overlay_opacity: 0.9,
            auto_check_updates: true,
            first_run_done: false,
        }
    }
}

impl Settings {
    pub fn carregar() -> Self {
        std::fs::read_to_string(paths::arquivo("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn salvar(&self) {
        paths::garantir_dir();
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(paths::arquivo("settings.json"), t);
        }
    }
}
