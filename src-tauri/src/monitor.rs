//! O coracao: decide o que o app sabe sobre a carga, e com quanta confianca.
//!
//! A fonte principal e o GATT do Bluetooth LE, que da percentual exato e avisa sozinho.
//! As outras vias existem porque nem todo controle e Bluetooth -- e todas sao piores,
//! cada uma de um jeito. As regras que separam leitura boa de leitura ruim estao aqui, e
//! todas nasceram de um caso real:
//!
//! - a primeira leitura depois de conectar pode ser um valor de espera; fica em
//!   observacao ate ser confirmada ou desmentida;
//! - estar conectado nao e o mesmo que ter leitura de agora; carga anterior ao vinculo
//!   atual nunca conta como ao vivo;
//! - degrau do XInput nunca substitui percentual exato de um controle de Bluetooth;
//! - a carga que o Windows guarda so vale se a data dela for desta ligacao.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::device::discovery::{self, Controle};
use crate::device::gatt::{self, AvisoGatt, VinculoGatt};
use crate::device::{gaming, hid, pnp, xinput};
use crate::history::History;
use crate::known::Conhecidos;
use crate::model::{BatteryState, Leitura, LinkMode, Precisao};
use crate::tempo;

/// Espacamento entre leituras que exigem perguntar ao dispositivo.
const INTERVALO_SEM_BLUETOOTH_MS: i64 = 20_000;

/// Quanto esperar antes de aceitar a primeira leitura de uma conexao.
const ESPERA_DE_CONFIRMACAO_MS: i64 = 12_000;

const INTERVALO_DE_DESCOBERTA_MS: i64 = 30_000;

#[derive(Debug, Default, Clone)]
struct Registro {
    percent: Option<i32>,
    nivel: Option<i32>,
    precisao: Precisao,
    /// Quando a leitura foi feita.
    em: Option<i64>,
    /// Quando se tentou pela ultima vez, tendo dado certo ou nao. Separado de `em`
    /// porque tentativa frustrada nao pode se passar por leitura nova nem apagar a
    /// ultima carga conhecida.
    tentativa: Option<i64>,
    /// Valor ainda nao confirmado, vindo da primeira leitura apos conectar.
    provisorio: bool,
}

impl Default for Precisao {
    fn default() -> Self {
        Precisao::Nenhuma
    }
}

pub struct Monitor {
    conhecidos: Conhecidos,
    historico: History,
    leituras: HashMap<String, Registro>,

    ativo: Option<Controle>,
    vinculo: Option<VinculoGatt>,
    envio: Sender<AvisoGatt>,
    recebimento: Receiver<AvisoGatt>,

    ultima_descoberta: i64,
    presentes: HashSet<String>,

    em_observacao: Option<(String, i32, i64)>,
    conectado_desde: Option<i64>,
    via_desde: i64,
    via_anterior: LinkMode,

    ultimo: Option<BatteryState>,
}

impl Monitor {
    pub fn novo() -> Self {
        let conhecidos = Conhecidos::carregar();
        let historico = History::carregar();
        let (envio, recebimento) = mpsc::channel();

        // o que ja se sabia da sessao passada entra como leitura conhecida, porem velha
        let mut leituras = HashMap::new();
        for salvo in conhecidos.itens() {
            let chave = salvo.como_controle().chave();
            if let Some(ultimo) = historico.ultimo(&chave) {
                leituras.insert(
                    chave,
                    Registro {
                        percent: Some(ultimo.p),
                        precisao: Precisao::Exata,
                        em: Some(ultimo.t),
                        ..Default::default()
                    },
                );
            }
        }

        let ativo = conhecidos
            .itens()
            .iter()
            .max_by_key(|c| c.visto_em())
            .map(|c| c.como_controle());

        Monitor {
            conhecidos,
            historico,
            leituras,
            ativo,
            vinculo: None,
            envio,
            recebimento,
            ultima_descoberta: 0,
            presentes: HashSet::new(),
            em_observacao: None,
            conectado_desde: None,
            via_desde: tempo::agora(),
            via_anterior: LinkMode::Offline,
            ultimo: None,
        }
    }

    /// Um giro do ciclo. Devolve o estado quando ele mudou o bastante para redesenhar.
    pub fn ciclo(&mut self) -> Option<BatteryState> {
        let agora = tempo::agora();

        let descobertos = if agora - self.ultima_descoberta > INTERVALO_DE_DESCOBERTA_MS {
            self.ultima_descoberta = agora;
            let achados = discovery::descobrir();
            self.presentes = achados.iter().map(|c| c.chave()).collect();
            if !achados.is_empty() {
                self.conhecidos.fundir(&achados);
            }
            achados
        } else {
            Vec::new()
        };

        // o Notify chega quando o controle quer, nao quando perguntamos
        while let Ok(aviso) = self.recebimento.try_recv() {
            let AvisoGatt::Carga { endereco, percent } = aviso;
            let chave = format!("{endereco:012x}");
            self.gravar(&chave, percent, agora, false);
        }

        let conectado = self
            .conhecidos
            .itens()
            .iter()
            .map(|c| c.como_controle())
            .find(|c| c.endereco != 0 && gatt::conectado(c.endereco));

        let modo = match conectado {
            Some(controle) => {
                self.ativo = Some(controle.clone());
                self.garantir_vinculo(&controle, agora);
                self.confirmar_em_observacao(agora);
                self.marcar_via(LinkMode::Bluetooth, agora);
                LinkMode::Bluetooth
            }
            None => {
                self.soltar_vinculo();
                self.em_observacao = None;

                let no_cabo = no_cabo(&descobertos);
                let modo = if no_cabo {
                    LinkMode::Cable
                } else if xinput::alguem_conectado() {
                    // Sem cabo e sem Bluetooth, o XInput ainda enxergar um controle so
                    // pode significar ligacao sem fio propria -- dongle ou adaptador.
                    LinkMode::Wireless
                } else {
                    LinkMode::Offline
                };
                self.marcar_via(modo, agora);

                // quem esta ligado agora vem antes de quem so foi visto um dia
                let melhor = self
                    .conhecidos
                    .itens()
                    .iter()
                    .max_by_key(|c| {
                        let presente = self.presentes.contains(&c.como_controle().chave());
                        (presente, c.visto_em())
                    })
                    .map(|c| c.como_controle());

                if melhor.is_some() {
                    self.ativo = melhor;
                }

                if modo != LinkMode::Offline {
                    if let Some(ativo) = self.ativo.clone() {
                        self.ler_sem_bluetooth(&ativo, agora);
                    }
                }
                modo
            }
        };

        let estado = self.montar(modo);
        let mudou = self.ultimo.as_ref().map(|u| !u.igual_a(&estado)).unwrap_or(true);
        self.ultimo = Some(estado.clone());
        mudou.then_some(estado)
    }

    // ---------------------------------------------------------------- vinculo

    fn garantir_vinculo(&mut self, controle: &Controle, agora: i64) {
        if self.vinculo.as_ref().map(|v| v.endereco()) == Some(controle.endereco) {
            return;
        }
        self.soltar_vinculo();

        let Ok((vinculo, inicial)) = gatt::abrir(controle.endereco, self.envio.clone()) else {
            return;
        };
        self.vinculo = Some(vinculo);
        self.conectado_desde = Some(agora);

        if let Some(pct) = inicial {
            self.gravar(&controle.chave(), pct, agora, true);
        }
    }

    fn soltar_vinculo(&mut self) {
        self.vinculo = None;
        self.conectado_desde = None;
    }

    // ---------------------------------------------------------------- leituras

    /// Guarda uma leitura exata. Provisoria significa "ainda sob suspeita".
    fn gravar(&mut self, chave: &str, percent: i32, agora: i64, provisorio: bool) {
        if !(0..=100).contains(&percent) {
            return;
        }
        let registro = self.leituras.entry(chave.to_string()).or_default();

        if provisorio {
            self.em_observacao = Some((chave.to_string(), percent, agora));

            // Havendo leitura anterior confiavel, ela continua no ar: trocar por um
            // valor que pode ser desmentido em segundos e o que fazia o numero piscar.
            if registro.precisao == Precisao::Exata && registro.percent.is_some() {
                return;
            }
            registro.percent = Some(percent);
            registro.nivel = None;
            registro.precisao = Precisao::Exata;
            registro.em = Some(agora);
            registro.provisorio = true;
            return;
        }

        self.em_observacao = None;
        registro.percent = Some(percent);
        registro.nivel = None;
        registro.precisao = Precisao::Exata;
        registro.em = Some(agora);
        registro.provisorio = false;
        self.historico.adicionar(chave, percent, agora);
    }

    /// Aceita a leitura de conexao que ninguem desmentiu dentro da janela de espera.
    /// Sem isto, controle que nao usa Notify nunca teria carga registrada.
    fn confirmar_em_observacao(&mut self, agora: i64) {
        let Some((chave, percent, quando)) = self.em_observacao.clone() else { return };
        if agora - quando < ESPERA_DE_CONFIRMACAO_MS {
            return;
        }
        self.gravar(&chave, percent, agora, false);
    }

    /// Vias que nao passam pelo Bluetooth, da mais precisa para a menos.
    fn ler_sem_bluetooth(&mut self, controle: &Controle, agora: i64) {
        let chave = controle.chave();
        {
            let registro = self.leituras.entry(chave.clone()).or_default();
            if let Some(t) = registro.tentativa {
                if agora - t < INTERVALO_SEM_BLUETOOTH_MS {
                    return;
                }
            }
            registro.tentativa = Some(agora);
        }

        let mut leitura = hid::ler(&controle.id_hid);
        if !leitura.tem() {
            if let Some(pct) = pnp::por_instancia(&controle.id_instancia, Some(self.via_desde)) {
                leitura = Leitura::exata(pct);
            }
        }
        if !leitura.tem() && controle.slot_xinput >= 0 {
            if let Some(n) = xinput::carga_do_slot(controle.slot_xinput as u32) {
                leitura = Leitura::degrau(n);
            }
        }
        if !leitura.tem() {
            if let Some(n) = xinput::carga_de_qualquer_slot() {
                leitura = Leitura::degrau(n);
            }
        }
        if !leitura.tem() {
            if let Some(pct) = pnp::por_container(&controle.container, Some(self.via_desde)) {
                leitura = Leitura::exata(pct);
            }
        }
        if !leitura.tem() {
            return;
        }

        // Controle de Bluetooth tem fonte melhor que qualquer uma daqui. Num intervalo
        // em que o GATT nao respondeu, aceitar o degrau no lugar trocaria "69%" por
        // "carga cheia" -- perder precisao que ja se tinha.
        let registro = self.leituras.entry(chave.clone()).or_default();
        if leitura.precisao == Precisao::Aproximada
            && registro.precisao == Precisao::Exata
            && controle.endereco != 0
        {
            return;
        }

        registro.em = Some(agora);
        registro.precisao = leitura.precisao;
        registro.provisorio = false;

        if leitura.precisao == Precisao::Exata {
            registro.percent = Some(leitura.valor);
            registro.nivel = None;
            self.historico.adicionar(&chave, leitura.valor, agora);
        } else {
            // degrau nao vira historico: a serie ficaria com saltos artificiais
            registro.nivel = Some(leitura.valor);
            registro.percent = None;
        }
    }

    fn marcar_via(&mut self, modo: LinkMode, agora: i64) {
        if self.via_anterior == modo {
            return;
        }
        self.via_anterior = modo;
        self.via_desde = agora;
    }

    // ---------------------------------------------------------------- saida

    fn montar(&self, modo: LinkMode) -> BatteryState {
        let chave = self
            .ativo
            .as_ref()
            .map(|c| c.chave())
            .unwrap_or_else(|| "wired".to_string());
        let registro = self.leituras.get(&chave).cloned().unwrap_or_default();
        let agora = tempo::agora();

        // No Bluetooth vale a leitura feita depois que este vinculo comecou; nas outras
        // vias, a que couber na janela de releitura. Em nenhuma delas vale uma leitura
        // ainda em observacao.
        let do_vinculo = modo != LinkMode::Bluetooth
            || matches!(
                (self.conectado_desde, registro.em),
                (Some(inicio), Some(em)) if em >= inicio
            );

        let ao_vivo = registro.em.is_some()
            && !registro.provisorio
            && do_vinculo
            && (modo == LinkMode::Bluetooth
                || agora - registro.em.unwrap_or(0) < INTERVALO_SEM_BLUETOOTH_MS * 2);

        BatteryState::montar(
            modo,
            registro.percent,
            registro.precisao,
            registro.nivel,
            registro.em,
            modo == LinkMode::Cable && gaming::carregando(),
            !ao_vivo,
            self.ativo
                .as_ref()
                .map(|c| c.nome.clone())
                .unwrap_or_else(|| "Nenhum controle pareado".to_string()),
            self.ativo.as_ref().and_then(|c| c.endereco_bonito()),
            chave.clone(),
            self.conhecidos.quantidade(),
            self.autonomia(&chave, &registro, modo),
        )
    }

    /// O que dizer sobre quanto tempo ainda da para jogar.
    ///
    /// Sem percentual exato nao ha o que estimar: os quatro degraus do XInput nao dizem
    /// quanto caiu, e inventar uma duracao a partir deles seria chutar com cara de conta.
    fn autonomia(&self, chave: &str, registro: &Registro, modo: LinkMode) -> Option<String> {
        if modo == LinkMode::Offline || modo == LinkMode::Cable {
            return None;
        }
        let percent = registro.percent.filter(|_| registro.precisao == Precisao::Exata)?;

        if let Some(minutos) = self.historico.autonomia_em_minutos(chave, percent) {
            return Some(crate::model::descrever_autonomia(minutos));
        }
        // Ha consumo medido, mas nao o bastante para uma duracao honesta. Dizer a taxa
        // ja e mais util que calar.
        if let Some(taxa) = self.historico.consumo_por_hora(chave) {
            return Some(format!("consumo de {taxa:.1} %/h").replace('.', ","));
        }
        Some("medindo o consumo".to_string())
    }

    pub fn historico(&self) -> &History {
        &self.historico
    }

    pub fn salvar(&self) {
        self.historico.salvar();
        self.conhecidos.salvar();
    }
}

/// Ligacao por cabo.
///
/// O XInput decide primeiro, porque ele distingue cabo de bateria no proprio relatorio.
/// Quando ele nao ve nada, um controle descoberto por outra via so pode estar no cabo --
/// sem fio ele apareceria para o XInput.
fn no_cabo(descobertos: &[Controle]) -> bool {
    if xinput::alguem_no_cabo() {
        return true;
    }
    if xinput::alguem_na_bateria() {
        return false;
    }
    !descobertos.is_empty() || gaming::quantidade() > 0
}
