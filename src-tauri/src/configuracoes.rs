use serde::{Deserialize, Serialize};

use crate::caminhos;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limiares {
    pub critico: i32,
    pub aviso: i32,
}

impl Limiares {
    pub const PADRAO: Limiares = Limiares { critico: 10, aviso: 20 };
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
    pub overlay_corner: OverlayCorner,
    pub overlay_monitor: i32,
    pub overlay_scale: f64,
    pub overlay_opacity: f64,
    pub auto_check_updates: bool,
    pub overlay_shortcut_enabled: bool,
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
            overlay_corner: OverlayCorner::InferiorDireito,
            overlay_monitor: -1,
            overlay_scale: 1.0,
            overlay_opacity: 0.9,
            auto_check_updates: true,
            overlay_shortcut_enabled: true,
            first_run_done: false,
        }
    }
}

impl Settings {
    pub fn carregar() -> Self {
        let Some(bruto) = caminhos::ler("settings.json") else {
            return Settings::default();
        };

        match serde_json::from_str::<Settings>(&bruto) {
            Ok(mut cfg) => {
                cfg.ajustar();
                cfg
            }
            Err(_) => {
                let caminho = caminhos::arquivo("settings.json");
                let _ = std::fs::rename(&caminho, caminho.with_extension("json.invalido"));
                Settings::default()
            }
        }
    }

    pub fn limiares(&self) -> Limiares {
        Limiares { critico: self.critical_threshold, aviso: self.warn_threshold }
    }

    pub fn ajustar(&mut self) {
        self.warn_threshold = self.warn_threshold.clamp(5, 90);
        self.critical_threshold = self.critical_threshold.clamp(1, self.warn_threshold - 1);
        self.overlay_scale = self.overlay_scale.clamp(0.75, 2.0);
        self.overlay_opacity = self.overlay_opacity.clamp(0.3, 1.0);
    }

    pub fn salvar(&self) {
        caminhos::garantir_dir();
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(caminhos::arquivo("settings.json"), t);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_critico_cabe_abaixo_do_aviso() {
        let mut cfg = Settings { warn_threshold: 10, critical_threshold: 40, ..Default::default() };
        cfg.ajustar();
        assert!(cfg.critical_threshold < cfg.warn_threshold);
    }

    #[test]
    fn a_pilula_nao_aceita_tamanho_absurdo() {
        let mut cfg = Settings { overlay_scale: 12.0, overlay_opacity: 0.0, ..Default::default() };
        cfg.ajustar();
        assert_eq!(cfg.overlay_scale, 2.0);
        assert_eq!(cfg.overlay_opacity, 0.3);
    }

    #[test]
    fn o_padrao_dos_limiares_acompanha_o_padrao_das_configuracoes() {
        assert_eq!(Settings::default().limiares(), Limiares::PADRAO);
    }

    #[test]
    fn o_padrao_ja_esta_ajustado() {
        let mut cfg = Settings::default();
        let antes = cfg.clone();
        cfg.ajustar();
        assert_eq!(cfg.warn_threshold, antes.warn_threshold);
        assert_eq!(cfg.critical_threshold, antes.critical_threshold);
        assert_eq!(cfg.overlay_scale, antes.overlay_scale);
    }
}
