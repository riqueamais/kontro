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
//! - a carga que o Windows guarda so vale se a data dela for desta ligacao, e numero sem
//!   data nenhuma nao derruba leitura que tem hora;
//! - como o controle esta ligado e pergunta sobre o controle, e a resposta so vale se a
//!   varredura que a sustenta for de agora.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::device::discovery::{self, Controle};
use crate::device::gatt::{self, AvisoGatt, VinculoGatt};
use crate::device::vigia::{self, Vigia};
use crate::device::{gaming, hid, pnp, xinput};
use crate::history::History;
use crate::known::Conhecidos;
use crate::model::{BatteryState, Leitura, LinkMode, Precisao};
use crate::tempo;

/// Espacamento entre leituras que exigem perguntar ao dispositivo.
const INTERVALO_SEM_BLUETOOTH_MS: i64 = 20_000;

/// Quanto esperar antes de aceitar a primeira leitura de uma conexao.
const ESPERA_DE_CONFIRMACAO_MS: i64 = 12_000;

/// Rede de seguranca da varredura.
///
/// Quem manda a varredura acontecer e o vigia, no instante em que o Windows publica ou
/// retira a interface. Este relogio so existe para o caso de um aviso se perder: com ele
/// sozinho, como era antes, a lista de presentes chegava a ter meio minuto de atraso --
/// e desligar o controle dava "no cabo" durante todo esse tempo.
const INTERVALO_DE_DESCOBERTA_MS: i64 = 15_000;

/// Menor espaco entre duas varreduras.
///
/// O vigia manda um aviso por interface, e um controle publica varias ao chegar. Sem
/// este descanso, um controle entrando dispararia quatro varreduras seguidas.
const DESCANSO_DA_DESCOBERTA_MS: i64 = 600;

/// Por quanto tempo a lista de pareados serve.
///
/// Ela custa uma abertura de dispositivo Bluetooth por aparelho, e so da nome. Separar a
/// validade dela da validade da presenca e o que torna barato varrer com frequencia.
const VALIDADE_DOS_PAREADOS_MS: i64 = 60_000;

/// Espaco minimo entre duas gravacoes do historico.
const INTERVALO_DE_GRAVACAO_MS: i64 = 30_000;

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

/// O que a ultima varredura viu de um controle que esta ligado agora.
///
/// Antes daqui so sobrava a chave, e as duas coisas que ela nao carrega eram justamente
/// as que faltavam. A chave depende da via -- o mesmo controle e o endereco no Bluetooth
/// e `hid:...` no cabo -- entao compara-la com a chave do registro salvo dava
/// desconectado com o cabo na mao. E o endereco desta interface e a unica prova direta de
/// que o controle chegou por Bluetooth agora, que e o que separa "sem fio" de "no cabo"
/// em controle que o XInput nao enxerga.
#[derive(Debug, Clone)]
struct Presenca {
    chave: String,
    container: String,
    /// Endereco extraido da interface desta conexao. Nao-zero significa Bluetooth agora.
    endereco: u64,
}

impl Presenca {
    fn de(controle: &Controle) -> Self {
        Presenca {
            chave: controle.chave(),
            container: controle.container.clone(),
            endereco: controle.endereco,
        }
    }
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
    /// Numero cuja hora e desconhecida: veio da carga que o Windows guarda, sem carimbo.
    /// Serve para mostrar, nunca para afirmar que e de agora.
    incerto: bool,
    /// Como este controle estava ligado na leitura anterior, e desde quando. E por
    /// controle, e nao do app: com dois ligados, um pode trocar de via sem o outro sair
    /// do lugar, e um relogio so fazia a troca de um zerar o piso de leitura do outro.
    via: LinkMode,
    via_desde: i64,
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

    vinculo: Option<VinculoGatt>,
    envio: Sender<AvisoGatt>,
    recebimento: Receiver<AvisoGatt>,

    _vigia: Vigia,
    avisos_do_vigia: Receiver<()>,

    ultima_descoberta: i64,
    presentes: Vec<Presenca>,
    pareados: Vec<(u64, String)>,
    pareados_em: i64,

    em_observacao: Option<(String, i32, i64)>,
    conectado_desde: Option<i64>,

    ultima_gravacao: i64,

    ultimo: Option<BatteryState>,
}

impl Monitor {
    pub fn novo() -> Self {
        let conhecidos = Conhecidos::carregar();
        let historico = History::carregar();
        let (envio, recebimento) = mpsc::channel();
        let (aviso, avisos_do_vigia) = mpsc::channel();
        let agora = tempo::agora();

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
                        via_desde: agora,
                        ..Default::default()
                    },
                );
            }
        }

        Monitor {
            conhecidos,
            historico,
            leituras,
            vinculo: None,
            envio,
            recebimento,
            _vigia: vigia::observar(aviso),
            avisos_do_vigia,
            ultima_descoberta: 0,
            presentes: Vec::new(),
            pareados: Vec::new(),
            pareados_em: 0,
            em_observacao: None,
            conectado_desde: None,
            ultima_gravacao: agora,
            ultimo: None,
        }
    }

    /// Um giro do ciclo. Devolve o panorama quando algo mudou o bastante para redesenhar.
    pub fn ciclo(&mut self) -> Option<Panorama> {
        let agora = tempo::agora();

        self.talvez_descobrir(agora);

        // o Notify chega quando o controle quer, nao quando perguntamos
        while let Ok(aviso) = self.recebimento.try_recv() {
            let AvisoGatt::Carga { endereco, percent } = aviso;
            let chave = format!("{endereco:012x}");
            self.gravar(&chave, percent, agora, false);
        }

        let controles: Vec<Controle> =
            self.conhecidos.itens().iter().map(|c| c.como_controle()).collect();

        // O XInput responde pelo conjunto, entao a pergunta e feita uma vez por ciclo --
        // e so quando ha controle presente que nao chegou por Bluetooth, que e o unico
        // caso em que a resposta dele muda alguma coisa.
        let algum_sem_bluetooth = self.presentes.iter().any(|p| p.endereco == 0);
        let algum_no_cabo = algum_sem_bluetooth && no_cabo();

        let mut estados = Vec::with_capacity(controles.len().max(1));
        let mut vinculado = false;

        for controle in &controles {
            let presenca = presenca_de(&self.presentes, controle).cloned();
            let no_gatt = controle.endereco != 0 && gatt::conectado(controle.endereco);
            let modo = modo_de(presenca.as_ref(), no_gatt, algum_no_cabo);
            self.marcar_via(&controle.chave(), modo, agora);

            // So um vinculo GATT por vez: manter varios abertos multiplicaria as
            // assinaturas de Notify sem o app ter o que fazer com elas.
            if modo == LinkMode::Bluetooth && !vinculado {
                vinculado = true;
                self.garantir_vinculo(controle, agora);
                self.confirmar_em_observacao(agora);
            }

            if modo != LinkMode::Offline && modo != LinkMode::Bluetooth {
                self.ler_sem_bluetooth(controle, agora);
            }

            estados.push(self.montar_um(controle, modo, agora));
        }

        if !vinculado {
            self.soltar_vinculo();
            self.em_observacao = None;
        }

        // sem controle conhecido ainda ha o que dizer: que nao ha nada
        if estados.is_empty() {
            estados.push(self.montar_vazio());
        }

        self.talvez_salvar(agora);

        let principal =
            escolher_principal(&estados).cloned().unwrap_or_else(|| self.montar_vazio());
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

    // ---------------------------------------------------------------- descoberta

    /// Varre quando o vigia avisa, e de tempos em tempos caso o aviso se perca.
    fn talvez_descobrir(&mut self, agora: i64) {
        let mexeram = self.avisos_do_vigia.try_iter().count() > 0;
        let desde = agora - self.ultima_descoberta;

        if desde > INTERVALO_DE_DESCOBERTA_MS || (mexeram && desde > DESCANSO_DA_DESCOBERTA_MS)
        {
            self.descobrir(agora);
        }
    }

    fn descobrir(&mut self, agora: i64) {
        self.ultima_descoberta = agora;

        let renovou = agora - self.pareados_em > VALIDADE_DOS_PAREADOS_MS;
        if renovou {
            self.renovar_pareados(agora);
        }

        let mut achados = discovery::descobrir_com(&self.pareados);

        // Controle que a lista guardada nao conhece chegou depois dela. Sem esta segunda
        // volta ele passaria ate um minuto exibindo o rotulo generico do HID em vez do
        // nome pelo qual o usuario o reconhece.
        if !renovou && achados.iter().any(|c| c.endereco != 0 && !self.tem_nome(c.endereco)) {
            self.renovar_pareados(agora);
            achados = discovery::descobrir_com(&self.pareados);
        }

        self.presentes = achados.iter().map(Presenca::de).collect();
        if !achados.is_empty() {
            self.conhecidos.fundir(&achados);
        }
    }

    fn renovar_pareados(&mut self, agora: i64) {
        self.pareados_em = agora;
        self.pareados = gatt::pareados();
    }

    fn tem_nome(&self, endereco: u64) -> bool {
        self.pareados.iter().any(|(e, _)| *e == endereco)
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
            registro.incerto = false;
            return;
        }

        self.em_observacao = None;
        registro.percent = Some(percent);
        registro.nivel = None;
        registro.precisao = Precisao::Exata;
        registro.em = Some(agora);
        registro.provisorio = false;
        registro.incerto = false;
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

        let anterior = self.leituras.get(&chave).cloned().unwrap_or_default();

        // Uma leitura guardada pelo Windows vale pela data dela, nao pela hora em que
        // perguntamos. O piso e o mais recente entre o inicio desta ligacao e o que ja
        // sabemos: numero anterior ao que temos e noticia velha, nunca nova.
        let piso = anterior.em.map(|c| c.max(anterior.via_desde)).unwrap_or(anterior.via_desde);

        let mut momento = None;
        let mut incerto = false;
        let mut leitura = hid::ler(&controle.id_hid);

        if !leitura.tem() {
            if let Some(c) = pnp::por_instancia(&controle.id_instancia, Some(piso)) {
                if let Some(valor) = aceitar_guardada(&c, &anterior) {
                    leitura = Leitura::exata(valor);
                    momento = c.medido_em;
                    incerto = c.medido_em.is_none();
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
                if let Some(valor) = aceitar_guardada(&c, &anterior) {
                    leitura = Leitura::exata(valor);
                    momento = c.medido_em;
                    incerto = c.medido_em.is_none();
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
        // desconfiar do valor -- por isso o que nao tem data vai marcado como incerto.
        let em = momento.map(|m| m.min(agora)).unwrap_or(agora);
        registro.em = Some(em);
        registro.precisao = leitura.precisao;
        registro.provisorio = false;
        registro.incerto = incerto;

        if leitura.precisao == Precisao::Exata {
            registro.percent = Some(leitura.valor);
            registro.nivel = None;
            // Numero de hora desconhecida nao entra na serie: uma amostra com hora
            // inventada estraga tanto a media de consumo quanto a comparacao de saude.
            if !incerto {
                self.historico.adicionar(&chave, leitura.valor, em);
            }
        } else {
            // degrau nao vira historico: a serie ficaria com saltos artificiais
            registro.nivel = Some(leitura.valor);
            registro.percent = None;
        }
    }

    fn marcar_via(&mut self, chave: &str, modo: LinkMode, agora: i64) {
        let registro = self.leituras.entry(chave.to_string()).or_default();
        if registro.via_desde == 0 {
            registro.via_desde = agora;
        }
        if registro.via == modo {
            return;
        }
        registro.via = modo;
        registro.via_desde = agora;
    }

    // ---------------------------------------------------------------- gravacao

    /// Grava o historico de tempos em tempos, quando ha o que gravar.
    ///
    /// Antes ele so era gravado no encerramento, e o encerramento nunca chegava: o laco
    /// de eventos do Tauri nao devolve o controle, entao a thread do monitor nunca saia
    /// do `loop` e o `salvar` final era codigo morto. Na pratica o arquivo ficava parado
    /// no que estivesse la, e depois de reiniciar o computador o app abria mostrando uma
    /// leitura antiga como se fosse a ultima que ele tinha feito.
    fn talvez_salvar(&mut self, agora: i64) {
        if !self.historico.precisa_salvar() {
            return;
        }
        if agora - self.ultima_gravacao < INTERVALO_DE_GRAVACAO_MS {
            return;
        }
        self.ultima_gravacao = agora;
        self.historico.salvar();
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

    fn montar_um(&self, controle: &Controle, modo: LinkMode, agora: i64) -> BatteryState {
        let chave = controle.chave();
        let registro = self.leituras.get(&chave).cloned().unwrap_or_default();

        // No Bluetooth vale a leitura feita depois que este vinculo comecou; nas outras
        // vias, a que couber na janela de releitura. Em nenhuma delas vale uma leitura
        // ainda em observacao, nem uma cuja hora ninguem sabe.
        let do_vinculo = modo != LinkMode::Bluetooth
            || matches!(
                (self.conectado_desde, registro.em),
                (Some(inicio), Some(em)) if em >= inicio
            );

        let ao_vivo = registro.em.is_some()
            && !registro.provisorio
            && !registro.incerto
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

    // ---------------------------------------------------------------- pedidos

    /// Refaz varredura e leitura agora, sem esperar nenhum dos relogios.
    ///
    /// E o que o botao "Atualizar" sempre prometeu: ate agora o pedido chegava ao ciclo e
    /// era descartado, e os dois botoes e o item da bandeja nao faziam absolutamente nada.
    pub fn ler_agora(&mut self) {
        let agora = tempo::agora();

        self.pareados_em = 0;
        self.descobrir(agora);

        for registro in self.leituras.values_mut() {
            registro.tentativa = None;
        }

        // O Notify so fala quando o valor muda: sem perguntar, quem apertou o botao
        // esperaria a bateria cair para ver a tela responder.
        if let Some(vinculo) = &self.vinculo {
            let endereco = vinculo.endereco();
            if let Some(pct) = gatt::reler(vinculo) {
                self.gravar(&format!("{endereco:012x}"), pct, agora, false);
            }
        }

        self.ultimo = None;
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
        self.presentes.retain(|p| p.chave != chave);

        if self.vinculo.as_ref().map(|v| format!("{:012x}", v.endereco()))
            == Some(chave.to_string())
        {
            self.soltar_vinculo();
        }
        // sem isto a tela so mudaria no proximo movimento da carga
        self.ultimo = None;
    }

    pub fn historico(&self) -> &History {
        &self.historico
    }

    pub fn salvar(&mut self) {
        self.historico.salvar();
        self.conhecidos.salvar();
    }
}

/// Como este controle esta ligado agora.
///
/// O modo e de cada controle, e nao do computador: com dois ligados, um pode estar no
/// Bluetooth e o outro no cabo.
///
/// A ordem das provas vai da mais direta para a menos. O GATT respondendo e prova de
/// Bluetooth. A interface pela qual a varredura acabou de achar o controle e prova da via
/// de agora: endereco nela significa que ele chegou por Bluetooth, e nesse caso nao ha o
/// que perguntar ao XInput -- que, alias, nem enxerga um DualSense. So quando nao ha
/// nenhuma dessas provas a resposta vem do XInput, que fala pelo conjunto.
fn modo_de(presenca: Option<&Presenca>, no_gatt: bool, algum_no_cabo: bool) -> LinkMode {
    if no_gatt {
        return LinkMode::Bluetooth;
    }
    let Some(presenca) = presenca else { return LinkMode::Offline };
    if presenca.endereco != 0 {
        return LinkMode::Wireless;
    }
    if algum_no_cabo {
        LinkMode::Cable
    } else {
        LinkMode::Wireless
    }
}

/// A presenca deste controle, se ele esta ligado agora.
///
/// O casamento repete o criterio de `Conhecidos::fundir`: chave ou container. So a chave
/// nao serve, porque ela depende da via -- um controle pareado por Bluetooth e depois
/// plugado no cabo e descoberto sem endereco, com outra chave, enquanto o registro salvo
/// guarda a do endereco de proposito. Comparando so as chaves, ele aparecia desconectado
/// justamente quando estava plugado.
fn presenca_de<'a>(presentes: &'a [Presenca], controle: &Controle) -> Option<&'a Presenca> {
    let chave = controle.chave();
    presentes.iter().find(|p| {
        p.chave == chave
            || (!controle.container.is_empty()
                && p.container.eq_ignore_ascii_case(&controle.container))
    })
}

/// Decide se um numero guardado pelo Windows vale mais do que o que ja sabemos.
///
/// Com data, o piso da consulta ja garantiu que ele e mais novo do que tudo o que
/// tinhamos, e ele entra. Sem data nao da para saber de que sessao ele e -- e numero de
/// idade desconhecida nao derruba uma leitura que tem hora.
///
/// Era isso que fazia "84%" virar "64%" depois de reiniciar o computador: a guarda antiga
/// so recusava o valor sem data quando a nossa leitura tinha menos de cinco minutos, e
/// depois de um boot a leitura semeada do arquivo tem sempre muito mais que isso. O valor
/// velho do Windows entrava, era carimbado com a hora de agora e ainda ia para o
/// historico como se fosse medida nova.
///
/// Quem so tem esta fonte nao fica sem carga: sem leitura nenhuma o numero entra, e a
/// partir dai um incerto sempre pode substituir outro incerto -- e a fonte continua
/// acompanhada, sempre marcada como hora desconhecida.
fn aceitar_guardada(carga: &pnp::CargaGuardada, atual: &Registro) -> Option<i32> {
    if carga.medido_em.is_some() {
        return Some(carga.percent);
    }
    let temos = atual.percent.is_some() && atual.precisao == Precisao::Exata;
    (!temos || atual.incerto).then_some(carga.percent)
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

/// Ligacao por cabo, para os controles que nao chegaram por Bluetooth.
///
/// O XInput distingue cabo de bateria no proprio relatorio, e por isso decide primeiro.
/// Quando ele nao ve nada, sobra o cabo: um controle que aparece no HID sem endereco de
/// Bluetooth e sem slot de XInput chegou por fio ou por dongle, e dizer "cabo" erra menos
/// que anunciar um percentual que aquela via nao sabe dar.
///
/// Esta funcao recebia antes um "ha algum controle presente" calculado sobre uma lista de
/// ate trinta segundos atras. Desligar o controle dava cabo durante todo esse tempo: anel
/// girando, "no cabo" na tela e um aviso de "agora no cabo" antes do de desconectado.
/// Hoje a presenca e do instante, e quem chegou por Bluetooth nem chega a perguntar aqui.
fn no_cabo() -> bool {
    if xinput::alguem_no_cabo() {
        return true;
    }
    !xinput::alguem_na_bateria()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn presente(chave: &str, container: &str, endereco: u64) -> Presenca {
        Presenca { chave: chave.into(), container: container.into(), endereco }
    }

    fn salvo(endereco: u64, container: &str) -> Controle {
        Controle { endereco, container: container.into(), ..Default::default() }
    }

    fn com_hora(percent: i32) -> Registro {
        Registro {
            percent: Some(percent),
            precisao: Precisao::Exata,
            em: Some(1),
            ..Default::default()
        }
    }

    fn estado(chave: &str, preenchimento: Option<i32>, modo: LinkMode) -> BatteryState {
        BatteryState::montar(
            modo,
            preenchimento,
            if preenchimento.is_some() { Precisao::Exata } else { Precisao::Nenhuma },
            None,
            None,
            false,
            false,
            chave.into(),
            None,
            chave.into(),
            1,
            None,
        )
    }

    #[test]
    fn desligar_o_controle_nao_e_ligar_no_cabo() {
        // O controle sai, o XInput nao ve mais nada, e o cabo continua sendo a resposta
        // do conjunto -- mas sem presenca nao ha o que estar no cabo.
        assert_eq!(modo_de(None, false, true), LinkMode::Offline);
    }

    #[test]
    fn quem_chegou_por_bluetooth_nunca_esta_no_cabo() {
        // O XInput nao enxerga um DualSense, e o Xbox por Bluetooth se declara a ele
        // como tipo desconhecido: nos dois casos a pergunta ao XInput nao responde nada,
        // e a resposta antiga para "nao sei" era cabo.
        let p = presente("408e2c82242f", "c1", 0x408e2c82242f);
        assert_eq!(modo_de(Some(&p), false, true), LinkMode::Wireless);
    }

    #[test]
    fn o_gatt_respondendo_manda_em_tudo() {
        let p = presente("hid:x", "c1", 0);
        assert_eq!(modo_de(Some(&p), true, true), LinkMode::Bluetooth);
    }

    #[test]
    fn sem_endereco_a_palavra_e_do_xinput() {
        let p = presente("hid:x", "c1", 0);
        assert_eq!(modo_de(Some(&p), false, true), LinkMode::Cable);
        assert_eq!(modo_de(Some(&p), false, false), LinkMode::Wireless);
    }

    #[test]
    fn o_controle_plugado_continua_sendo_o_mesmo() {
        // Pareado por Bluetooth, a chave salva e o endereco; plugado no cabo, a
        // descoberta o acha sem endereco e com outra chave. So o container os une, e sem
        // ele o app mostrava desconectado com o cabo na mao.
        let presentes = vec![presente("hid:abc", "c1", 0)];
        assert!(presenca_de(&presentes, &salvo(0x408e2c82242f, "c1")).is_some());
    }

    #[test]
    fn container_diferente_nao_empresta_presenca() {
        let presentes = vec![presente("hid:abc", "c1", 0)];
        assert!(presenca_de(&presentes, &salvo(0x408e2c82242f, "c2")).is_none());
    }

    #[test]
    fn numero_sem_data_nao_derruba_leitura_com_hora() {
        // Era assim que 84% virava 64% depois de reiniciar o computador.
        let guardado = pnp::CargaGuardada { percent: 64, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &com_hora(84)), None);
    }

    #[test]
    fn numero_sem_data_preenche_o_vazio() {
        let guardado = pnp::CargaGuardada { percent: 64, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &Registro::default()), Some(64));
    }

    #[test]
    fn a_fonte_sem_data_continua_acompanhada() {
        // Quem so tem esta via nao pode congelar no primeiro numero que informou.
        let anterior = Registro { incerto: true, ..com_hora(64) };
        let guardado = pnp::CargaGuardada { percent: 60, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &anterior), Some(60));
    }

    #[test]
    fn numero_com_data_entra_sempre() {
        // A data ja passou pelo piso da consulta: se chegou aqui, e mais nova.
        let guardado = pnp::CargaGuardada { percent: 64, medido_em: Some(2) };
        assert_eq!(aceitar_guardada(&guardado, &com_hora(84)), Some(64));
    }

    #[test]
    fn o_icone_mostra_quem_esta_pior() {
        let lista = [
            estado("cheio", Some(90), LinkMode::Bluetooth),
            estado("vazio", Some(12), LinkMode::Wireless),
        ];
        assert_eq!(escolher_principal(&lista).unwrap().key, "vazio");
    }

    #[test]
    fn controle_desligado_nao_rouba_o_icone_de_quem_esta_ligado() {
        let lista = [
            estado("desligado", Some(3), LinkMode::Offline),
            estado("ligado", Some(80), LinkMode::Bluetooth),
        ];
        assert_eq!(escolher_principal(&lista).unwrap().key, "ligado");
    }
}
