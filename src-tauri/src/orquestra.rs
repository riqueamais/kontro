use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

const INTERVALO_ENTRE_REAFIRMACOES: Duration = Duration::from_secs(1);

const REAFIRMACOES_ANTES_DE_DESISTIR: u8 = 3;

#[derive(Default)]
struct Topo {
    tentativas: u8,
    ultima: Option<Instant>,
    desistiu: bool,
}

pub struct Orquestrador {
    aberto_em: i64,
    anterior: Option<(String, Via)>,
    conexao_a_avisar: Option<i64>,
    avisados: HashMap<String, Vec<i32>>,
    ultimo_percentual: HashMap<String, i32>,
    topo: Topo,
}

impl Orquestrador {
    pub fn novo() -> Self {
        Orquestrador {
            aberto_em: tempo::agora(),
            anterior: None,
            conexao_a_avisar: None,
            avisados: HashMap::new(),
            ultimo_percentual: HashMap::new(),
            topo: Topo::default(),
        }
    }

    pub fn primeiro_plano_mudou(&mut self, app: &AppHandle) {
        self.topo = Topo::default();
        crate::marcar_coberta(app, None);
    }

    pub fn reafirmar_topo(&mut self, app: &AppHandle) {
        use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

        let Some(compartilhado) = app.try_state::<Arc<crate::Compartilhado>>() else { return };
        let Some(valor) = compartilhado.pilula.get().copied() else { return };
        let pilula = janelas::hwnd_de_valor(valor);

        if !unsafe { IsWindowVisible(pilula).as_bool() } {
            if self.topo.tentativas > 0 || self.topo.desistiu {
                self.topo = Topo::default();
            }
            crate::marcar_coberta(app, None);
            return;
        }

        let Some(ocluidor) = tela::alguem_por_cima(pilula) else {
            self.topo.tentativas = 0;
            self.topo.desistiu = false;
            crate::marcar_coberta(app, None);
            return;
        };

        if self.topo.desistiu {
            return;
        }
        if self.topo.ultima.is_some_and(|u| u.elapsed() < INTERVALO_ENTRE_REAFIRMACOES) {
            return;
        }

        if self.topo.tentativas >= REAFIRMACOES_ANTES_DE_DESISTIR {
            self.topo.desistiu = true;
            let nome = crate::jogo::executavel_de(ocluidor)
                .map(|c| crate::jogo::batizar(&c))
                .unwrap_or_else(|| "Outro programa".into());
            crate::marcar_coberta(app, Some(nome));
            return;
        }

        self.topo.tentativas += 1;
        self.topo.ultima = Some(Instant::now());
        if let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) {
            let _ = app.run_on_main_thread(move || janelas::vestir_estilos(&janela));
        }
    }

    pub fn reavaliar(
        &mut self,
        app: &AppHandle,
        estado: &EstadoDoControle,
        cfg: &Settings,
        mao: Option<bool>,
        solta: bool,
        tela_cheia: bool,
        previa: bool,
    ) {
        self.sobreposicao(app, estado, cfg, mao, solta, tela_cheia, previa);
        self.reafirmar_topo(app);
        self.transicao(app, estado, cfg);
        self.talvez_avisar(app, estado, cfg);
        self.limiares(app, estado, cfg);
    }

    fn sobreposicao(
        &self,
        app: &AppHandle,
        estado: &EstadoDoControle,
        cfg: &Settings,
        mao: Option<bool>,
        solta: bool,
        tela_cheia: bool,
        previa: bool,
    ) {
        let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) else { return };

        if solta {
            if !janela.is_visible().unwrap_or(false) {
                janelas::mostrar_por_cima_da_thread_da_interface(app, janela);
                let _ = app.emit("kontro://pilula-apareceu", ());
            }
            return;
        }

        let ligada = cfg.overlay_mode != OverlayMode::Desligada;
        let tem_leitura = estado.via != Via::Desligado;
        let momento_de_jogo = cfg.overlay_mode == OverlayMode::Sempre || tela_cheia;

        let ajustando = previa
            && app
                .get_webview_window(janelas::PRINCIPAL)
                .and_then(|j| j.is_focused().ok())
                .unwrap_or(false);

        let critico = !estado.carregando
            && !estado.leitura_antiga
            && estado.preenchimento.map(|p| p <= cfg.critical_threshold).unwrap_or(false);

        let mostrar = match mao {
            Some(escolha) => escolha,
            None => ligada && (ajustando || (tem_leitura && (momento_de_jogo || critico))),
        };

        if mostrar {
            janelas::posicionar_sobreposicao(app, cfg, tela_cheia);
            if !janela.is_visible().unwrap_or(false) {
                janelas::mostrar_por_cima_da_thread_da_interface(app, janela);
                let _ = app.emit("kontro://pilula-apareceu", ());
            }
        } else if janela.is_visible().unwrap_or(false) {
            let _ = app.emit("kontro://pilula-vai-sumir", ());
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
            mostrar_aviso(app, estado, cfg, AvisoDeLigacao::Desconectou);
            return;
        }

        if anterior == Via::Desligado {
            self.conexao_a_avisar = Some(tempo::agora());
            return;
        }

        mostrar_aviso(app, estado, cfg, AvisoDeLigacao::TrocouDeVia);
    }

    fn talvez_avisar(&mut self, app: &AppHandle, estado: &EstadoDoControle, cfg: &Settings) {
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
        mostrar_aviso(app, estado, cfg, AvisoDeLigacao::Conectou);
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

fn mostrar_aviso(
    app: &AppHandle,
    estado: &EstadoDoControle,
    cfg: &Settings,
    assunto: AvisoDeLigacao,
) {
    let Some(janela) = app.get_webview_window(janelas::AVISO) else { return };

    janelas::posicionar_aviso(app, cfg);
    janelas::mostrar_por_cima_da_thread_da_interface(app, janela);
    let _ = app.emit("kontro://aviso", Pacote { assunto, estado });
}
