use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::configuracoes::{OverlayMode, Settings};
use crate::janelas;
use crate::modelo::{EstadoDoControle, Via};
use crate::tela;
use crate::tempo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AvisoDeLigacao {
    Conectou,
    Desconectou,
    TrocouDeVia,
}

const ESPERA_DO_AVISO_MS: i64 = 10_000;

const CARENCIA_DA_ABERTURA_MS: i64 = 6_000;

pub struct Orquestrador {
    aberto_em: i64,
    anterior: Option<(String, Via)>,
    conexao_a_avisar: Option<i64>,
    avisados: HashMap<String, Vec<i32>>,
    ultimo_percentual: HashMap<String, i32>,
}

impl Orquestrador {
    pub fn novo() -> Self {
        Orquestrador {
            aberto_em: tempo::agora(),
            anterior: None,
            conexao_a_avisar: None,
            avisados: HashMap::new(),
            ultimo_percentual: HashMap::new(),
        }
    }

    pub fn reavaliar(
        &mut self,
        app: &AppHandle,
        estado: &EstadoDoControle,
        cfg: &Settings,
        mao: Option<bool>,
        ligados: usize,
    ) {
        self.sobreposicao(app, estado, cfg, mao, ligados);
        self.transicao(app, estado, cfg);
        self.talvez_avisar(app, estado);
        self.limiares(app, estado, cfg);
    }

    fn sobreposicao(
        &self,
        app: &AppHandle,
        estado: &EstadoDoControle,
        cfg: &Settings,
        mao: Option<bool>,
        ligados: usize,
    ) {
        let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) else { return };

        let ligada = cfg.overlay_mode != OverlayMode::Desligada;
        let tem_leitura = estado.via != Via::Desligado;
        let momento_de_jogo = cfg.overlay_mode == OverlayMode::Sempre || tela::em_tela_cheia();

        let ajustando = app
            .get_webview_window(janelas::PRINCIPAL)
            .and_then(|j| j.is_focused().ok())
            .unwrap_or(false);

        let critico = !estado.carregando
            && !estado.leitura_antiga
            && estado.preenchimento.map(|p| p <= cfg.critical_threshold).unwrap_or(false);

        let mostrar = match mao {
            Some(escolha) => ligada && escolha,
            None => ligada && (ajustando || (tem_leitura && (momento_de_jogo || critico))),
        };

        if mostrar {
            janelas::posicionar_sobreposicao(app, cfg, ligados);
            if !janela.is_visible().unwrap_or(false) {
                let _ = janela.show();
            }
        } else if janela.is_visible().unwrap_or(false) {
            let _ = janela.hide();
        }
    }

    fn transicao(&mut self, app: &AppHandle, estado: &EstadoDoControle, cfg: &Settings) {
        let anterior = self.anterior.replace((estado.chave.clone(), estado.via));

        let Some((chave, modo)) = anterior else { return };
        if chave != estado.chave || modo == estado.via {
            return;
        }
        let anterior = modo;

        if !cfg.connect_toast_enabled {
            return;
        }
        if tempo::agora() - self.aberto_em < CARENCIA_DA_ABERTURA_MS {
            self.conexao_a_avisar = None;
            return;
        }

        if estado.via == Via::Desligado {
            self.conexao_a_avisar = None;
            mostrar_aviso(app, estado, AvisoDeLigacao::Desconectou);
            return;
        }

        if anterior == Via::Desligado {
            self.conexao_a_avisar = Some(tempo::agora());
            return;
        }

        mostrar_aviso(app, estado, AvisoDeLigacao::TrocouDeVia);
    }

    fn talvez_avisar(&mut self, app: &AppHandle, estado: &EstadoDoControle) {
        let Some(desde) = self.conexao_a_avisar else { return };
        if estado.via == Via::Desligado {
            self.conexao_a_avisar = None;
            return;
        }

        let tem_carga = !estado.leitura_antiga && estado.preenchimento.is_some();
        let cansou = tempo::agora() - desde > ESPERA_DO_AVISO_MS;
        if !tem_carga && !cansou {
            return;
        }

        self.conexao_a_avisar = None;
        mostrar_aviso(app, estado, AvisoDeLigacao::Conectou);
    }

    fn limiares(&mut self, app: &AppHandle, estado: &EstadoDoControle, cfg: &Settings) {
        if !cfg.notifications_enabled || estado.via == Via::Desligado {
            return;
        }
        if estado.leitura_antiga {
            return;
        }
        let Some(pct) = estado.percentual.filter(|_| estado.tem_numero) else { return };

        let avisados = self.avisados.entry(estado.chave.clone()).or_default();

        if let Some(anterior) = self.ultimo_percentual.get(&estado.chave) {
            if pct - anterior > 5 {
                avisados.clear();
            }
        }
        self.ultimo_percentual.insert(estado.chave.clone(), pct);

        let nome = if estado.nome.trim().is_empty() { "O controle" } else { estado.nome.as_str() };

        for limite in [cfg.critical_threshold, cfg.warn_threshold] {
            if pct > limite || avisados.contains(&limite) {
                continue;
            }
            avisados.push(limite);

            let corpo = format!("{nome} está com {pct}% de carga.");
            let titulo =
                if limite == cfg.critical_threshold { "Carga crítica" } else { "Carga baixa" };

            let _ = app.notification().builder().title(titulo).body(corpo).show();
            break;
        }
    }
}

#[derive(Serialize, Clone)]
struct Pacote<'a> {
    assunto: AvisoDeLigacao,
    estado: &'a EstadoDoControle,
}

fn mostrar_aviso(app: &AppHandle, estado: &EstadoDoControle, assunto: AvisoDeLigacao) {
    let Some(janela) = app.get_webview_window(janelas::AVISO) else { return };

    janelas::posicionar_aviso(app);
    let _ = janela.show();
    let _ = app.emit("kontro://aviso", Pacote { assunto, estado });
}
