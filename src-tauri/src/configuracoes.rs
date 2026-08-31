use serde::{Deserialize, Serialize};

use crate::atalho;
use crate::caminhos;

pub const ATALHO_DA_PILULA: &str = "Ctrl+Shift+KeyK";
pub const ATALHO_DE_MOVER: &str = "Ctrl+Shift+KeyM";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limiares {
    pub critico: i32,
    pub aviso: i32,
}

impl Limiares {
    pub const PADRAO: Limiares = Limiares { critico: 10, aviso: 20 };
}

pub const ENUMS_DA_CONFIG: &[(&str, &[&str])] = &[
    ("CloseAction", &["MinimizeToTray", "Exit"]),
    ("OverlayMode", &["Desligada", "EmJogo", "Sempre"]),
];

pub const CAMPOS_DA_CONFIG: &[(&str, &str)] = &[
    ("StartWithWindows", "boolean"),
    ("StartMinimized", "boolean"),
    ("CloseAction", "CloseAction"),
    ("NotificationsEnabled", "boolean"),
    ("WarnThreshold", "number"),
    ("CriticalThreshold", "number"),
    ("ConnectToastEnabled", "boolean"),
    ("OverlayMode", "OverlayMode"),
    ("OverlayX", "number"),
    ("OverlayY", "number"),
    ("OverlayMonitor", "number"),
    ("OverlayScale", "number"),
    ("OverlayOpacity", "number"),
    ("AutoCheckUpdates", "boolean"),
    ("OverlayShortcutEnabled", "boolean"),
    ("OverlayShortcut", "string"),
    ("OverlayMoveShortcut", "string"),
    ("FirstRunDone", "boolean"),
];

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
    pub overlay_x: f64,
    pub overlay_y: f64,
    pub overlay_monitor: i32,
    pub overlay_scale: f64,
    pub overlay_opacity: f64,
    pub auto_check_updates: bool,
    pub overlay_shortcut_enabled: bool,
    pub overlay_shortcut: String,
    pub overlay_move_shortcut: String,
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
            overlay_x: 1.0,
            overlay_y: 1.0,
            overlay_monitor: -1,
            overlay_scale: 1.0,
            overlay_opacity: 0.9,
            auto_check_updates: true,
            overlay_shortcut_enabled: true,
            overlay_shortcut: ATALHO_DA_PILULA.to_string(),
            overlay_move_shortcut: ATALHO_DE_MOVER.to_string(),
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
                if let Some((x, y)) = posicao_do_canto_antigo(&bruto) {
                    cfg.overlay_x = x;
                    cfg.overlay_y = y;
                }
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
        self.overlay_x = ponto_valido(self.overlay_x);
        self.overlay_y = ponto_valido(self.overlay_y);
        self.overlay_shortcut = atalho::sanear(&self.overlay_shortcut, ATALHO_DA_PILULA);
        self.overlay_move_shortcut = atalho::sanear(&self.overlay_move_shortcut, ATALHO_DE_MOVER);

        if self.overlay_move_shortcut == self.overlay_shortcut {
            self.overlay_move_shortcut = ATALHO_DE_MOVER.to_string();
            if self.overlay_move_shortcut == self.overlay_shortcut {
                self.overlay_shortcut = ATALHO_DA_PILULA.to_string();
            }
        }
    }

    pub fn salvar(&self) {
        caminhos::garantir_dir();
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(caminhos::arquivo("settings.json"), t);
        }
    }
}

fn ponto_valido(valor: f64) -> f64 {
    if valor.is_finite() {
        valor.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn posicao_do_canto_antigo(bruto: &str) -> Option<(f64, f64)> {
    let json: serde_json::Value = serde_json::from_str(bruto).ok()?;
    let campos = json.as_object()?;
    if campos.contains_key("OverlayX") || campos.contains_key("OverlayY") {
        return None;
    }

    match campos.get("OverlayCorner")?.as_str()? {
        "SuperiorEsquerdo" => Some((0.0, 0.0)),
        "SuperiorDireito" => Some((1.0, 0.0)),
        "InferiorEsquerdo" => Some((0.0, 1.0)),
        "InferiorDireito" => Some((1.0, 1.0)),
        _ => None,
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
    fn a_pilula_nao_e_largada_fora_da_tela() {
        let mut cfg = Settings { overlay_x: 4.0, overlay_y: -2.0, ..Default::default() };
        cfg.ajustar();
        assert_eq!(cfg.overlay_x, 1.0);
        assert_eq!(cfg.overlay_y, 0.0);
    }

    #[test]
    fn o_canto_escolhido_na_versao_antiga_vira_ponto_na_tela() {
        let antigo = r#"{"OverlayCorner":"SuperiorEsquerdo","OverlayScale":1.0}"#;
        assert_eq!(posicao_do_canto_antigo(antigo), Some((0.0, 0.0)));
    }

    #[test]
    fn quem_ja_arrastou_a_pilula_nao_volta_para_o_canto() {
        let novo = r#"{"OverlayCorner":"SuperiorEsquerdo","OverlayX":0.5,"OverlayY":0.0}"#;
        assert_eq!(posicao_do_canto_antigo(novo), None);
    }

    #[test]
    fn atalho_que_o_sistema_nao_entende_volta_para_o_padrao() {
        let mut cfg = Settings {
            overlay_shortcut: "Banana+Uva".to_string(),
            overlay_move_shortcut: "  Ctrl+Alt+KeyP  ".to_string(),
            ..Default::default()
        };
        cfg.ajustar();
        assert_eq!(cfg.overlay_shortcut, ATALHO_DA_PILULA);
        assert_eq!(cfg.overlay_move_shortcut, "Ctrl+Alt+KeyP");
    }

    #[test]
    fn atalho_sem_modificador_volta_para_o_padrao() {
        let mut cfg = Settings { overlay_shortcut: "KeyK".to_string(), ..Default::default() };
        cfg.ajustar();
        assert_eq!(cfg.overlay_shortcut, ATALHO_DA_PILULA);
    }

    #[test]
    fn os_dois_atalhos_nunca_ficam_iguais() {
        let mut cfg = Settings {
            overlay_shortcut: "Ctrl+Alt+KeyJ".to_string(),
            overlay_move_shortcut: "Ctrl+Alt+KeyJ".to_string(),
            ..Default::default()
        };
        cfg.ajustar();
        assert_ne!(cfg.overlay_shortcut, cfg.overlay_move_shortcut);
    }

    #[test]
    fn a_descricao_da_config_cobre_exatamente_os_campos_gravados() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        let mut reais: Vec<String> = json.as_object().unwrap().keys().cloned().collect();
        let mut descritos: Vec<String> =
            CAMPOS_DA_CONFIG.iter().map(|(nome, _)| nome.to_string()).collect();
        reais.sort();
        descritos.sort();

        assert_eq!(
            descritos, reais,
            "campo novo em Settings sem entrada em CAMPOS_DA_CONFIG: o front nao veria"
        );
    }

    #[test]
    fn os_valores_descritos_dos_enums_ainda_sao_aceitos() {
        for (tipo, valores) in ENUMS_DA_CONFIG {
            for valor in *valores {
                let mut base = serde_json::to_value(Settings::default()).unwrap();
                base[*tipo] = serde_json::Value::String(valor.to_string());

                assert!(
                    serde_json::from_value::<Settings>(base).is_ok(),
                    "{tipo}::{valor} nao e mais aceito, mas continua descrito"
                );
            }
        }
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
