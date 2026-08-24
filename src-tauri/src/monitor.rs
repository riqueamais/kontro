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

/// Por quanto tempo uma leitura nossa segura um valor guardado sem data.
///
/// O Windows guarda a ultima carga que o controle informou e continua devolvendo esse
/// numero muito depois -- as vezes de outra sessao, as vezes sem carimbo nenhum. Quando
/// ja temos leitura propria e ela e recente, o valor guardado nao acrescenta nada e so
/// pode piorar: e assim que "84%" virava "64%" depois de reiniciar o computador.
const VALIDADE_DE_LEITURA_MS: i64 = 5 * 60 * 1000;

const INTERVALO_DE_DESCOBERTA_MS: i64 = 30_000;

/// O que o app sabe agora sobre todos os controles.
///
/// O principal e quem manda no icone da bandeja, na sobreposicao e no aviso: essas tres
/// coisas so cabem uma de cada vez, e escolher entre elas merece criterio -- nao "o
/// primeiro que aparecer".
#[derive(Debug, Clone)]
pub struct Panorama {
    pub principal: BatteryState,
    pub todos: Vec<BatteryState>,
}

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

    /// Um giro do ciclo. Devolve o panorama quando algo mudou o bastante para redesenhar.
    pub fn ciclo(&mut self) -> Option<Panorama> {
        let agora = tempo::agora();

        if agora - self.ultima_descoberta > INTERVALO_DE_DESCOBERTA_MS {
            self.ultima_descoberta = agora;
            let achados = discovery::descobrir();
            self.presentes = achados.iter().map(|c| c.chave()).collect();
            if !achados.is_empty() {
                self.conhecidos.fundir(&achados);
            }
        }

        // o Notify chega quando o controle quer, nao quando perguntamos
        while let Ok(aviso) = self.recebimento.try_recv() {
            let AvisoGatt::Carga { endereco, percent } = aviso;
            let chave = format!("{endereco:012x}");
            self.gravar(&chave, percent, agora, false);
        }

        let controles: Vec<Controle> =
            self.conhecidos.itens().iter().map(|c| c.como_controle()).collect();

        // O cabo e uma pergunta ao conjunto, nao a um dispositivo: uma vez por ciclo.
        let algum_no_cabo = no_cabo(!self.presentes.is_empty());

        let mut estados = Vec::with_capacity(controles.len().max(1));
        let mut vinculado = false;

        for controle in &controles {
            let presente = self.presentes.contains(&controle.chave());
            let modo = self.modo_de(controle, presente, algum_no_cabo);

            // So um vinculo GATT por vez: manter varios abertos multiplicaria as
            // assinaturas de Notify sem o app ter o que fazer com elas.
            if modo == LinkMode::Bluetooth && !vinculado {
                vinculado = true;
                self.garantir_vinculo(controle, agora);
                self.confirmar_em_observacao(agora);
                self.marcar_via(LinkMode::Bluetooth, agora);
            }

            if modo != LinkMode::Offline && modo != LinkMode::Bluetooth {
                self.marcar_via(modo, agora);
                self.ler_sem_bluetooth(controle, agora);
            }

            estados.push(self.montar_um(controle, modo));
        }

        if !vinculado {
            self.soltar_vinculo();
            self.em_observacao = None;
        }

        // sem controle conhecido ainda ha o que dizer: que nao ha nada
        if estados.is_empty() {
            estados.push(self.montar_vazio());
        }

        // quem esta ligado agora vem antes de quem so foi visto um dia
        self.ativo = escolher_principal(&estados)
            .and_then(|e| controles.iter().find(|c| c.chave() == e.key).cloned());

        let principal = escolher_principal(&estados)
            .cloned()
            .unwrap_or_else(|| self.montar_vazio());
        let panorama = Panorama { principal, todos: estados };

        let mudou = self
            .ultimo
            .as_ref()
            .map(|u| {
                !u.igual_a(&panorama.principal) || u.known_count != panorama.todos.len()
            })
            .unwrap_or(true);

        self.ultimo = Some(panorama.principal.clone());
        mudou.then_some(panorama)
    }

    /// Como este controle esta ligado agora.
    ///
    /// O modo e de cada controle, e nao do computador: com dois ligados, um pode estar no
    /// Bluetooth e o outro no cabo. O XInput so responde pelo conjunto, entao a distincao
    /// entre cabo e sem fio vale para todos os que nao sao Bluetooth -- e com um controle
    /// so, que e o caso comum, isso da exatamente a resposta certa.
    fn modo_de(&self, controle: &Controle, presente: bool, algum_no_cabo: bool) -> LinkMode {
        if controle.endereco != 0 && gatt::conectado(controle.endereco) {
            return LinkMode::Bluetooth;
        }
        if !presente {
            return LinkMode::Offline;
        }
        if algum_no_cabo {
            LinkMode::Cable
        } else {
            LinkMode::Wireless
        }
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

        // Uma leitura guardada pelo Windows vale pela data dela, nao pela hora em que
        // perguntamos. O piso e o mais recente entre o inicio desta ligacao e o que ja
        // sabemos: numero anterior ao que temos e noticia velha, nunca nova.
        let conhecida = self.leituras.get(&chave).and_then(|r| r.em);
        let piso = conhecida.map(|c| c.max(self.via_desde)).unwrap_or(self.via_desde);

        let mut momento = None;
        let mut leitura = hid::ler(&controle.id_hid);
        if !leitura.tem() {
            if let Some(c) = pnp::por_instancia(&controle.id_instancia, Some(piso)) {
                if let Some(valor) = Self::aceitar_guardada(&c, conhecida, agora) {
                    leitura = Leitura::exata(valor);
                    momento = c.medido_em;
                }
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
            if let Some(c) = pnp::por_container(&controle.container, Some(piso)) {
                if let Some(valor) = Self::aceitar_guardada(&c, conhecida, agora) {
                    leitura = Leitura::exata(valor);
                    momento = c.medido_em;
                }
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

        // A data da leitura e a da medicao, quando existe. Carimbar de "agora" um numero
        // de ontem faria a tela mentir a hora, que e o unico sinal que o usuario tem para
        // desconfiar do valor.
        let em = momento.map(|m| m.min(agora)).unwrap_or(agora);
        registro.em = Some(em);
        registro.precisao = leitura.precisao;
        registro.provisorio = false;

        if leitura.precisao == Precisao::Exata {
            registro.percent = Some(leitura.valor);
            registro.nivel = None;
            self.historico.adicionar(&chave, leitura.valor, em);
        } else {
            // degrau nao vira historico: a serie ficaria com saltos artificiais
            registro.nivel = Some(leitura.valor);
            registro.percent = None;
        }
    }

    /// Decide se um numero guardado pelo Windows vale mais do que o que ja sabemos.
    ///
    /// Sem data nenhuma nao da para comparar, entao vale a idade da nossa leitura: se ela
    /// e recente, o valor guardado nao entra. Com data, o piso da consulta ja garantiu que
    /// ele e mais novo do que tudo o que tinhamos.
    fn aceitar_guardada(
        carga: &pnp::CargaGuardada,
        conhecida: Option<i64>,
        agora: i64,
    ) -> Option<i32> {
        if carga.medido_em.is_none() {
            if let Some(quando) = conhecida {
                if agora - quando < VALIDADE_DE_LEITURA_MS {
                    return None;
                }
            }
        }
        Some(carga.percent)
    }

    fn marcar_via(&mut self, modo: LinkMode, agora: i64) {
        if self.via_anterior == modo {
            return;
        }
        self.via_anterior = modo;
        self.via_desde = agora;
    }

    // ---------------------------------------------------------------- saida

    /// O estado quando nao ha controle nenhum conhecido.
    fn montar_vazio(&self) -> BatteryState {
        BatteryState::montar(
            LinkMode::Offline,
            None,
            Precisao::Nenhuma,
            None,
            None,
            false,
            true,
            "Nenhum controle pareado".to_string(),
            None,
            "wired".to_string(),
            0,
            None,
        )
    }

    fn montar_um(&self, controle: &Controle, modo: LinkMode) -> BatteryState {
        let chave = controle.chave();
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
            controle.nome.clone(),
            controle.endereco_bonito(),
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

    /// Guarda o apelido e faz o proximo ciclo ja sair com ele.
    pub fn renomear(&mut self, chave: &str, nome: &str) {
        if self.conhecidos.renomear(chave, nome) {
            // forca o proximo ciclo a contar como mudanca, senao a tela so atualizaria
            // quando a carga variasse -- o que pode demorar muitos minutos
            self.ultimo = None;
        }
    }

    /// Tira o controle da lista, do historico e do que se sabia sobre ele.
    pub fn esquecer(&mut self, chave: &str) {
        if !self.conhecidos.esquecer(chave) {
            return;
        }
        self.historico.esquecer(chave);
        self.leituras.remove(chave);
        self.presentes.remove(chave);
        if self.ativo.as_ref().map(|c| c.chave()) == Some(chave.to_string()) {
            self.ativo = None;
            self.soltar_vinculo();
        }
        // sem isto a tela so mudaria no proximo movimento da carga
        self.ultimo = None;
    }

    pub fn historico(&self) -> &History {
        &self.historico
    }

    pub fn salvar(&self) {
        self.historico.salvar();
        self.conhecidos.salvar();
    }
}

/// Qual controle manda no icone da bandeja, na sobreposicao e no aviso.
///
/// O criterio e a menor carga entre os ligados. O icone existe para avisar antes do
/// controle morrer, entao ele tem de mostrar o que esta mais perto disso -- mostrar o
/// mais cheio seria esconder justamente a informacao que importa. Sem ninguem ligado,
/// vale a ultima leitura conhecida, que ao menos diz de quem era.
fn escolher_principal(estados: &[BatteryState]) -> Option<&BatteryState> {
    let ligados: Vec<&BatteryState> =
        estados.iter().filter(|e| e.mode != LinkMode::Offline).collect();

    let candidatos = if ligados.is_empty() { estados.iter().collect() } else { ligados };

    candidatos
        .into_iter()
        .min_by_key(|e| (e.preenchimento.is_none(), e.preenchimento.unwrap_or(0)))
}

/// Ligacao por cabo.
///
/// O XInput decide primeiro, porque ele distingue cabo de bateria no proprio relatorio.
/// Quando ele nao ve nada, um controle descoberto por outra via so pode estar no cabo:
/// sem fio ele apareceria para o XInput. E esse terceiro ramo que cobre os controles que
/// nao falam XInput -- sem ele, um controle plugado por cabo apareceria como sem fio.
fn no_cabo(ha_controle_presente: bool) -> bool {
    if xinput::alguem_no_cabo() {
        return true;
    }
    if xinput::alguem_na_bateria() {
        return false;
    }
    ha_controle_presente || gaming::quantidade() > 0
}
