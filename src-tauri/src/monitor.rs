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

const INTERVALO_SEM_BLUETOOTH_MS: i64 = 20_000;

const ESPERA_DE_CONFIRMACAO_MS: i64 = 12_000;

const INTERVALO_DE_DESCOBERTA_MS: i64 = 15_000;

const DESCANSO_DA_DESCOBERTA_MS: i64 = 600;

const VALIDADE_DOS_PAREADOS_MS: i64 = 60_000;

const INTERVALO_DE_GRAVACAO_MS: i64 = 30_000;

#[derive(Debug, Clone)]
pub struct Panorama {
    pub principal: BatteryState,
    pub todos: Vec<BatteryState>,
}

#[derive(Debug, Clone)]
struct Presenca {
    chave: String,
    container: String,
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
    em: Option<i64>,
    tentativa: Option<i64>,
    provisorio: bool,
    incerto: bool,
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

    pub fn ciclo(&mut self) -> Option<Panorama> {
        let agora = tempo::agora();

        self.talvez_descobrir(agora);

        while let Ok(aviso) = self.recebimento.try_recv() {
            let AvisoGatt::Carga { endereco, percent } = aviso;
            let chave = format!("{endereco:012x}");
            self.gravar(&chave, percent, agora, false);
        }

        let controles: Vec<Controle> =
            self.conhecidos.itens().iter().map(|c| c.como_controle()).collect();

        let algum_sem_bluetooth = self.presentes.iter().any(|p| p.endereco == 0);
        let algum_no_cabo = algum_sem_bluetooth && no_cabo();

        let mut estados = Vec::with_capacity(controles.len().max(1));
        let mut vinculado = false;

        for controle in &controles {
            let presenca = presenca_de(&self.presentes, controle).cloned();
            let no_gatt = controle.endereco != 0 && gatt::conectado(controle.endereco);
            let modo = modo_de(presenca.as_ref(), no_gatt, algum_no_cabo);
            self.marcar_via(&controle.chave(), modo, agora);

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

    fn gravar(&mut self, chave: &str, percent: i32, agora: i64, provisorio: bool) {
        if !(0..=100).contains(&percent) {
            return;
        }
        let registro = self.leituras.entry(chave.to_string()).or_default();

        if provisorio {
            self.em_observacao = Some((chave.to_string(), percent, agora));

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

    fn confirmar_em_observacao(&mut self, agora: i64) {
        let Some((chave, percent, quando)) = self.em_observacao.clone() else { return };
        if agora - quando < ESPERA_DE_CONFIRMACAO_MS {
            return;
        }
        self.gravar(&chave, percent, agora, false);
    }

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

        let registro = self.leituras.entry(chave.clone()).or_default();
        if leitura.precisao == Precisao::Aproximada
            && registro.precisao == Precisao::Exata
            && controle.endereco != 0
        {
            return;
        }

        let em = momento.map(|m| m.min(agora)).unwrap_or(agora);
        registro.em = Some(em);
        registro.precisao = leitura.precisao;
        registro.provisorio = false;
        registro.incerto = incerto;

        if leitura.precisao == Precisao::Exata {
            registro.percent = Some(leitura.valor);
            registro.nivel = None;
            if !incerto {
                self.historico.adicionar(&chave, leitura.valor, em);
            }
        } else {
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

    fn autonomia(&self, chave: &str, registro: &Registro, modo: LinkMode) -> Option<String> {
        if modo == LinkMode::Offline || modo == LinkMode::Cable {
            return None;
        }
        let percent = registro.percent.filter(|_| registro.precisao == Precisao::Exata)?;

        if let Some(minutos) = self.historico.autonomia_em_minutos(chave, percent) {
            return Some(crate::model::descrever_autonomia(minutos));
        }
        if let Some(taxa) = self.historico.consumo_por_hora(chave) {
            return Some(format!("consumo de {taxa:.1} %/h").replace('.', ","));
        }
        Some("medindo o consumo".to_string())
    }

    pub fn ler_agora(&mut self) {
        let agora = tempo::agora();

        self.pareados_em = 0;
        self.descobrir(agora);

        for registro in self.leituras.values_mut() {
            registro.tentativa = None;
        }

        if let Some(vinculo) = &self.vinculo {
            let endereco = vinculo.endereco();
            if let Some(pct) = gatt::reler(vinculo) {
                self.gravar(&format!("{endereco:012x}"), pct, agora, false);
            }
        }

        self.ultimo = None;
    }

    pub fn renomear(&mut self, chave: &str, nome: &str) {
        if self.conhecidos.renomear(chave, nome) {
            self.ultimo = None;
        }
    }

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

fn presenca_de<'a>(presentes: &'a [Presenca], controle: &Controle) -> Option<&'a Presenca> {
    let chave = controle.chave();
    presentes.iter().find(|p| {
        p.chave == chave
            || (!controle.container.is_empty()
                && p.container.eq_ignore_ascii_case(&controle.container))
    })
}

fn aceitar_guardada(carga: &pnp::CargaGuardada, atual: &Registro) -> Option<i32> {
    if carga.medido_em.is_some() {
        return Some(carga.percent);
    }
    let temos = atual.percent.is_some() && atual.precisao == Precisao::Exata;
    (!temos || atual.incerto).then_some(carga.percent)
}

fn escolher_principal(estados: &[BatteryState]) -> Option<&BatteryState> {
    let ligados: Vec<&BatteryState> =
        estados.iter().filter(|e| e.mode != LinkMode::Offline).collect();

    let candidatos = if ligados.is_empty() { estados.iter().collect() } else { ligados };

    candidatos
        .into_iter()
        .min_by_key(|e| (e.preenchimento.is_none(), e.preenchimento.unwrap_or(0)))
}

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
        assert_eq!(modo_de(None, false, true), LinkMode::Offline);
    }

    #[test]
    fn quem_chegou_por_bluetooth_nunca_esta_no_cabo() {
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
        let anterior = Registro { incerto: true, ..com_hora(64) };
        let guardado = pnp::CargaGuardada { percent: 60, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &anterior), Some(60));
    }

    #[test]
    fn numero_com_data_entra_sempre() {
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
