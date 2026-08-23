//! Quando cada janela aparece.
//!
//! A sobreposicao e o aviso nao sao decisao da interface: ela desenha o que recebe, mas
//! quem decide se e hora de mostrar precisa saber o que ocupa a tela e o que mudou na
//! ligacao desde o ciclo anterior. Essa memoria vive aqui.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::janelas;
use crate::model::{BatteryState, LinkMode};
use crate::settings::{OverlayMode, Settings};
use crate::tela;
use crate::tempo;

/// Do que o aviso trata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AvisoDeLigacao {
    Conectou,
    Desconectou,
    /// Estava sem fio e passou para o cabo, ou o contrario.
    TrocouDeVia,
}

/// Quanto esperar pela carga antes de avisar assim mesmo.
///
/// A leitura real chega alguns segundos depois da conexao, e o aviso fica muito melhor
/// com o numero pronto. Mas ha controle que nunca informa carga: passado este tempo,
/// avisar sem numero e melhor que nao avisar.
const ESPERA_DO_AVISO_MS: i64 = 10_000;

/// Os primeiros segundos apos abrir sao ignorados de proposito: o app parte de
/// desconectado e logo encontra o controle, o que viraria um aviso em toda inicializacao
/// sem nada ter acontecido.
const CARENCIA_DA_ABERTURA_MS: i64 = 6_000;

pub struct Orquestrador {
    aberto_em: i64,
    modo_anterior: LinkMode,
    /// Conexao que ainda espera uma leitura para virar aviso.
    conexao_a_avisar: Option<i64>,
    /// Limiares ja avisados nesta carga, para nao repetir a cada leitura.
    avisados: Vec<i32>,
    ultimo_percentual: Option<i32>,
}

impl Orquestrador {
    pub fn novo() -> Self {
        Orquestrador {
            aberto_em: tempo::agora(),
            modo_anterior: LinkMode::Offline,
            conexao_a_avisar: None,
            avisados: Vec::new(),
            ultimo_percentual: None,
        }
    }

    /// Reavalia as janelas para o estado atual.
    ///
    /// Roda a cada ciclo, e nao so quando a carga muda: a visibilidade da sobreposicao
    /// depende do que ocupa a tela. Reavaliar so na mudanca de carga faria entrar e sair
    /// de um jogo nao ter efeito ate a proxima variacao de percentual, o que pode
    /// demorar muitos minutos.
    pub fn reavaliar(&mut self, app: &AppHandle, estado: &BatteryState, cfg: &Settings) {
        self.sobreposicao(app, estado, cfg);
        self.transicao(app, estado, cfg);
        self.talvez_avisar(app, estado);
        self.limiares(app, estado, cfg);
    }

    // ------------------------------------------------------------ sobreposicao

    fn sobreposicao(&self, app: &AppHandle, estado: &BatteryState, cfg: &Settings) {
        let Some(janela) = app.get_webview_window(janelas::SOBREPOSICAO) else { return };

        let ligada = cfg.overlay_mode != OverlayMode::Desligada;
        let tem_leitura = estado.mode != LinkMode::Offline;
        let momento_de_jogo =
            cfg.overlay_mode == OverlayMode::Sempre || tela::em_tela_cheia();

        // Enquanto o usuario esta na janela de configuracoes a sobreposicao aparece de
        // qualquer jeito, funcionando como previa: escolher o canto seria as cegas se
        // ela sumisse ao clicar no botao.
        //
        // O criterio e o foco, e nao a visibilidade. Uma janela aberta atras de tudo, ou
        // minimizada, continua "visivel" para o sistema -- e com esse criterio bastava
        // abrir as configuracoes uma vez e nao fechar para a pilula ficar na tela para
        // sempre, mesmo no modo que so devia aparecer em jogo.
        let ajustando = app
            .get_webview_window(janelas::PRINCIPAL)
            .and_then(|j| j.is_focused().ok())
            .unwrap_or(false);

        let mostrar = ligada && (ajustando || (tem_leitura && momento_de_jogo));

        if mostrar {
            janelas::posicionar_sobreposicao(app, cfg);
            if !janela.is_visible().unwrap_or(false) {
                let _ = janela.show();
            }
        } else if janela.is_visible().unwrap_or(false) {
            let _ = janela.hide();
        }
    }

    // ------------------------------------------------------------ aviso

    /// Detecta o que mudou na ligacao e decide se ha algo a avisar.
    fn transicao(&mut self, app: &AppHandle, estado: &BatteryState, cfg: &Settings) {
        let anterior = self.modo_anterior;
        self.modo_anterior = estado.mode;
        if anterior == estado.mode {
            return;
        }

        if !cfg.connect_toast_enabled {
            return;
        }
        if tempo::agora() - self.aberto_em < CARENCIA_DA_ABERTURA_MS {
            self.conexao_a_avisar = None;
            return;
        }

        if estado.mode == LinkMode::Offline {
            // nao ha leitura a esperar: o controle acabou de sair
            self.conexao_a_avisar = None;
            mostrar_aviso(app, estado, AvisoDeLigacao::Desconectou);
            return;
        }

        if anterior == LinkMode::Offline {
            // Conectar e um evento; ter a carga e outro, alguns segundos depois. O aviso
            // fica marcado aqui e sai quando o numero existe -- assim ele nasce pronto,
            // em vez de aparecer dizendo que esta lendo.
            self.conexao_a_avisar = Some(tempo::agora());
            return;
        }

        // seguia ligado e trocou de via: cabo para sem fio, ou o contrario
        mostrar_aviso(app, estado, AvisoDeLigacao::TrocouDeVia);
    }

    /// Solta o aviso marcado quando a carga chega, ou quando a espera acaba.
    fn talvez_avisar(&mut self, app: &AppHandle, estado: &BatteryState) {
        let Some(desde) = self.conexao_a_avisar else { return };
        if estado.mode == LinkMode::Offline {
            self.conexao_a_avisar = None;
            return;
        }

        let tem_carga = !estado.stale && estado.preenchimento.is_some();
        let cansou = tempo::agora() - desde > ESPERA_DO_AVISO_MS;
        if !tem_carga && !cansou {
            return;
        }

        self.conexao_a_avisar = None;
        mostrar_aviso(app, estado, AvisoDeLigacao::Conectou);
    }
}

impl Orquestrador {
    /// Avisa quando a carga cruza os limiares que o usuario escolheu.
    ///
    /// Limiar em porcentagem so faz sentido com leitura exata: com os quatro degraus do
    /// XInput nao da para dizer se passou de vinte por cento. Melhor calar do que
    /// inventar o momento do aviso.
    fn limiares(&mut self, app: &AppHandle, estado: &BatteryState, cfg: &Settings) {
        if !cfg.notifications_enabled || estado.mode == LinkMode::Offline {
            return;
        }
        let Some(pct) = estado.percent.filter(|_| estado.tem_numero) else { return };

        // subiu de forma relevante: carga nova, pode avisar de novo mais tarde
        if let Some(anterior) = self.ultimo_percentual {
            if pct - anterior > 5 {
                self.avisados.clear();
            }
        }
        self.ultimo_percentual = Some(pct);

        let nome = if estado.device_name.trim().is_empty() {
            "O controle"
        } else {
            estado.device_name.as_str()
        };

        for limite in [cfg.critical_threshold, cfg.warn_threshold] {
            if pct > limite || self.avisados.contains(&limite) {
                continue;
            }
            self.avisados.push(limite);

            let corpo = format!("{nome} está com {pct}% de carga.");
            let titulo = if limite == cfg.critical_threshold {
                "Carga crítica"
            } else {
                "Carga baixa"
            };

            // A notificacao do sistema e a certa aqui, e nao a caixa do proprio app: ela
            // fica na Central de Acoes para ser lida depois. O aviso do app e para o que
            // acontece na hora e nao precisa sobreviver.
            let _ = app.notification().builder().title(titulo).body(corpo).show();
            break;
        }
    }
}

#[derive(Serialize, Clone)]
struct Pacote<'a> {
    assunto: AvisoDeLigacao,
    estado: &'a BatteryState,
}

/// Posiciona, mostra e conta para a interface do que se trata.
///
/// Quem apaga o aviso e a propria interface: ela conhece a duracao da animacao de saida,
/// e escondendo daqui a janela sumiria no meio dela.
fn mostrar_aviso(app: &AppHandle, estado: &BatteryState, assunto: AvisoDeLigacao) {
    let Some(janela) = app.get_webview_window(janelas::AVISO) else { return };

    janelas::posicionar_aviso(app);
    let _ = janela.show();
    let _ = app.emit("kontro://aviso", Pacote { assunto, estado });
}
