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
    pub overlay_corner: OverlayCorner,
    /// -1 significa acompanhar a tela em foco.
    pub overlay_monitor: i32,
    /// Tamanho da pilula, como multiplicador.
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
        let Some(bruto) = paths::ler("settings.json") else {
            return Settings::default();
        };

        match serde_json::from_str::<Settings>(&bruto) {
            Ok(mut cfg) => {
                cfg.ajustar();
                cfg
            }
            Err(_) => {
                // O arquivo existe e nao foi entendido. Deixa-lo no lugar seria pior: a
                // abertura ve `first_run_done` falso, regrava com os padroes e apaga em
                // silencio tudo o que o usuario tinha escolhido. Guardado de lado, ao
                // menos da para recuperar o que havia dentro.
                let caminho = paths::arquivo("settings.json");
                let _ = std::fs::rename(&caminho, caminho.with_extension("json.invalido"));
                Settings::default()
            }
        }
    }

    /// Poe os valores dentro do que o app sabe desenhar.
    ///
    /// A configuracao vem da interface, mas tambem de um arquivo que o usuario pode ter
    /// editado a mao ou que sobrou de uma versao anterior. Um limiar critico acima do de
    /// aviso, por exemplo, faria os dois avisos sairem juntos e no momento errado.
    pub fn ajustar(&mut self) {
        self.warn_threshold = self.warn_threshold.clamp(5, 90);
        self.critical_threshold = self.critical_threshold.clamp(1, self.warn_threshold - 1);
        self.overlay_scale = self.overlay_scale.clamp(0.75, 2.0);
        self.overlay_opacity = self.overlay_opacity.clamp(0.3, 1.0);
    }

    pub fn salvar(&self) {
        paths::garantir_dir();
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(paths::arquivo("settings.json"), t);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_critico_cabe_abaixo_do_aviso() {
        // Um arquivo editado a mao, ou vindo de uma versao anterior, pode trazer os dois
        // trocados -- e ai os dois avisos sairiam juntos, no momento errado.
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
    fn o_padrao_ja_esta_ajustado() {
        let mut cfg = Settings::default();
        let antes = cfg.clone();
        cfg.ajustar();
        assert_eq!(cfg.warn_threshold, antes.warn_threshold);
        assert_eq!(cfg.critical_threshold, antes.critical_threshold);
        assert_eq!(cfg.overlay_scale, antes.overlay_scale);
    }
}
