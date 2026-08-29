use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::dispositivo::descoberta::{self, Controle};
use crate::dispositivo::gatt::{self, AvisoGatt, VinculoGatt};
use crate::dispositivo::vigia::{self, Vigia};
use crate::dispositivo::{gaming, hid, pnp, xinput};
use crate::historico::History;
use crate::conhecidos::Conhecidos;
use crate::modelo::{Bruto, EstadoDoControle, Leitura, Precisao, Via};
use crate::tempo;

const INTERVALO_SEM_BLUETOOTH_MS: i64 = 20_000;

const ESPERA_DE_CONFIRMACAO_MS: i64 = 12_000;

const INTERVALO_DE_DESCOBERTA_MS: i64 = 15_000;

const DESCANSO_DA_DESCOBERTA_MS: i64 = 600;

const VALIDADE_DOS_PAREADOS_MS: i64 = 60_000;

const INTERVALO_DE_GRAVACAO_MS: i64 = 30_000;
const ESPERA_ENTRE_VINCULOS_MS: i64 = 5_000;

#[derive(Debug, Clone)]
pub struct Panorama {
    pub principal: EstadoDoControle,
    pub todos: Vec<EstadoDoControle>,
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
    percentual: Option<i32>,
    nivel: Option<i32>,
    precisao: Precisao,
    em: Option<i64>,
    tentativa: Option<i64>,
    provisorio: bool,
    incerto: bool,
    via: Via,
    via_desde: i64,
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

    em_observacao: Option<(String, i32, i64, Via)>,
    conectado_desde: Option<i64>,
    tentativa_de_vinculo: i64,

    ultima_gravacao: i64,

    ultimo: Option<EstadoDoControle>,
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
                        percentual: Some(ultimo.p),
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
            tentativa_de_vinculo: 0,
            ultima_gravacao: agora,
            ultimo: None,
        }
    }

    pub fn ciclo(&mut self) -> Option<Panorama> {
        let agora = tempo::agora();

        self.talvez_descobrir(agora);

        while let Ok(aviso) = self.recebimento.try_recv() {
            let AvisoGatt::Carga { endereco, percentual } = aviso;
            let chave = format!("{endereco:012x}");
            self.gravar(&chave, percentual, agora, false, Via::Bluetooth);
        }

        let controles: Vec<Controle> =
            self.conhecidos.itens().iter().map(|c| c.como_controle()).collect();

        let algum_sem_bluetooth = self.presentes.iter().any(|p| p.endereco == 0);
        let algum_no_cabo = algum_sem_bluetooth && no_cabo();

        let mut estados = Vec::with_capacity(controles.len().max(1));
        let mut vinculado = false;

        for controle in &controles {
            let presenca = presenca_de(&self.presentes, controle).cloned();
            let endereco = controle.endereco;
            let modo = modo_de(presenca.as_ref(), algum_no_cabo, || {
                endereco != 0 && gatt::conectado(endereco)
            });
            self.marcar_via(&controle.chave(), modo, agora);

            if modo == Via::Bluetooth && !vinculado {
                vinculado = true;
                self.garantir_vinculo(controle, agora);
                self.confirmar_em_observacao(agora);
            }

            let dele = self.vinculo.as_ref().map(|v| v.endereco()) == Some(endereco);
            if modo != Via::Desligado && !dele {
                self.ler_sem_bluetooth(controle, modo, agora);
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
                !u.igual_a(&panorama.principal) || u.quantos_conhecidos != panorama.todos.len()
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

        let mut achados = descoberta::descobrir_com(&self.pareados);

        if !renovou && achados.iter().any(|c| c.endereco != 0 && !self.tem_nome(c.endereco)) {
            self.renovar_pareados(agora);
            achados = descoberta::descobrir_com(&self.pareados);
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
        if agora - self.tentativa_de_vinculo < ESPERA_ENTRE_VINCULOS_MS {
            return;
        }
        self.tentativa_de_vinculo = agora;
        self.soltar_vinculo();

        let Ok((vinculo, inicial)) = gatt::abrir(controle.endereco, self.envio.clone()) else {
            return;
        };
        self.vinculo = Some(vinculo);
        self.conectado_desde = Some(agora);

        if let Some(pct) = inicial {
            self.gravar(&controle.chave(), pct, agora, true, Via::Bluetooth);
        }
    }

    fn soltar_vinculo(&mut self) {
        self.vinculo = None;
        self.conectado_desde = None;
    }

    fn gravar(&mut self, chave: &str, percentual: i32, agora: i64, provisorio: bool, via: Via) {
        if !(0..=100).contains(&percentual) {
            return;
        }
        let registro = self.leituras.entry(chave.to_string()).or_default();

        if provisorio {
            self.em_observacao = Some((chave.to_string(), percentual, agora, via));

            if registro.precisao == Precisao::Exata && registro.percentual.is_some() {
                return;
            }
            registro.percentual = Some(percentual);
            registro.nivel = None;
            registro.precisao = Precisao::Exata;
            registro.em = Some(agora);
            registro.provisorio = true;
            registro.incerto = false;
            return;
        }

        self.em_observacao = None;
        registro.percentual = Some(percentual);
        registro.nivel = None;
        registro.precisao = Precisao::Exata;
        registro.em = Some(agora);
        registro.provisorio = false;
        registro.incerto = false;
        self.historico.adicionar(chave, percentual, agora, via);
    }

    fn confirmar_em_observacao(&mut self, agora: i64) {
        let Some((chave, percentual, quando, via)) = self.em_observacao.clone() else { return };
        if agora - quando < ESPERA_DE_CONFIRMACAO_MS {
            return;
        }
        self.gravar(&chave, percentual, agora, false, via);
    }

    fn ler_sem_bluetooth(&mut self, controle: &Controle, modo: Via, agora: i64) {
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
        let mut nos: Option<Vec<pnp::NoDeBateria>> = None;

        if !leitura.tem() {
            let lista = nos.get_or_insert_with(pnp::nos_com_bateria);
            if let Some(c) = pnp::por_instancia(lista, &controle.id_instancia, Some(piso)) {
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
            let lista = nos.get_or_insert_with(pnp::nos_com_bateria);
            if let Some(c) = pnp::por_container(lista, &controle.container, Some(piso)) {
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
            registro.percentual = Some(leitura.valor);
            registro.nivel = None;
            if !incerto {
                self.historico.adicionar(&chave, leitura.valor, em, modo);
            }
        } else {
            registro.nivel = Some(leitura.valor);
            registro.percentual = None;
        }
    }

    fn marcar_via(&mut self, chave: &str, modo: Via, agora: i64) {
        let registro = self.leituras.entry(chave.to_string()).or_default();
        if registro.via_desde == 0 {
            registro.via_desde = agora;
        }
        if registro.via == modo {
            return;
        }
        let saiu_do_ar = modo == Via::Desligado && registro.via != Via::Desligado;
        let ultimo = registro.percentual.filter(|_| registro.precisao == Precisao::Exata);
        registro.via = modo;
        registro.via_desde = agora;

        if let (true, Some(percentual)) = (saiu_do_ar, ultimo) {
            self.historico.marcar_desligado(chave, percentual, agora);
        }
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

    fn montar_vazio(&self) -> EstadoDoControle {
        EstadoDoControle::montar(Bruto {
            leitura_antiga: true,
            nome: "Nenhum controle pareado".to_string(),
            chave: "wired".to_string(),
            ..Default::default()
        })
    }

    fn montar_um(&self, controle: &Controle, modo: Via, agora: i64) -> EstadoDoControle {
        let chave = controle.chave();
        let registro = self.leituras.get(&chave).cloned().unwrap_or_default();

        let do_vinculo = modo != Via::Bluetooth
            || matches!(
                (self.conectado_desde, registro.em),
                (Some(inicio), Some(em)) if em >= inicio
            );

        let ao_vivo = registro.em.is_some()
            && !registro.provisorio
            && !registro.incerto
            && do_vinculo
            && (modo == Via::Bluetooth
                || agora - registro.em.unwrap_or(0) < INTERVALO_SEM_BLUETOOTH_MS * 2);

        EstadoDoControle::montar(Bruto {
            via: modo,
            percentual: registro.percentual,
            precisao: registro.precisao,
            nivel: registro.nivel,
            lido_em: registro.em,
            carregando: modo == Via::Cabo && gaming::carregando(),
            leitura_antiga: !ao_vivo,
            nome: controle.nome.clone(),
            endereco: controle.endereco_bonito(),
            chave: chave.clone(),
            quantos_conhecidos: self.conhecidos.quantidade(),
            autonomia: self.autonomia(&chave, &registro, modo),
        })
    }

    fn autonomia(&self, chave: &str, registro: &Registro, modo: Via) -> Option<String> {
        if modo == Via::Desligado || modo == Via::Cabo {
            return None;
        }
        let percentual = registro.percentual.filter(|_| registro.precisao == Precisao::Exata)?;

        if let Some(minutos) = self.historico.autonomia_em_minutos(chave, percentual) {
            return Some(crate::modelo::descrever_autonomia(minutos));
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
                self.gravar(&format!("{endereco:012x}"), pct, agora, false, Via::Bluetooth);
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

fn modo_de(
    presenca: Option<&Presenca>,
    algum_no_cabo: bool,
    responde_no_gatt: impl FnOnce() -> bool,
) -> Via {
    let Some(presenca) = presenca else { return Via::Desligado };
    if presenca.endereco != 0 || responde_no_gatt() {
        return Via::Bluetooth;
    }
    if algum_no_cabo {
        Via::Cabo
    } else {
        Via::SemFio
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
        return Some(carga.percentual);
    }
    let temos = atual.percentual.is_some() && atual.precisao == Precisao::Exata;
    (!temos || atual.incerto).then_some(carga.percentual)
}

fn escolher_principal(estados: &[EstadoDoControle]) -> Option<&EstadoDoControle> {
    let ligados: Vec<&EstadoDoControle> =
        estados.iter().filter(|e| e.via != Via::Desligado).collect();

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

    fn com_hora(percentual: i32) -> Registro {
        Registro {
            percentual: Some(percentual),
            precisao: Precisao::Exata,
            em: Some(1),
            ..Default::default()
        }
    }

    fn estado(chave: &str, preenchimento: Option<i32>, modo: Via) -> EstadoDoControle {
        EstadoDoControle::montar(Bruto {
            via: modo,
            percentual: preenchimento,
            precisao: if preenchimento.is_some() { Precisao::Exata } else { Precisao::Nenhuma },
            nome: chave.into(),
            chave: chave.into(),
            quantos_conhecidos: 1,
            ..Default::default()
        })
    }

    #[test]
    fn desligar_o_controle_nao_e_ligar_no_cabo() {
        assert_eq!(modo_de(None, true, || false), Via::Desligado);
    }

    #[test]
    fn quem_chegou_por_bluetooth_nunca_esta_no_cabo() {
        let p = presente("408e2c82242f", "c1", 0x408e2c82242f);
        assert_eq!(modo_de(Some(&p), true, || false), Via::Bluetooth);
    }

    #[test]
    fn a_interface_de_bluetooth_dispensa_perguntar_ao_radio() {
        let p = presente("408e2c82242f", "c1", 0x408e2c82242f);
        let modo = modo_de(Some(&p), true, || panic!("perguntou ao radio sem precisar"));
        assert_eq!(modo, Via::Bluetooth);
    }

    #[test]
    fn controle_desligado_dispensa_perguntar_ao_radio() {
        let modo = modo_de(None, true, || panic!("perguntou ao radio por um controle que saiu"));
        assert_eq!(modo, Via::Desligado);
    }

    #[test]
    fn sem_endereco_na_interface_o_radio_ainda_responde() {
        let p = presente("hid:x", "c1", 0);
        assert_eq!(modo_de(Some(&p), true, || true), Via::Bluetooth);
    }

    #[test]
    fn sem_endereco_e_sem_radio_a_palavra_e_do_xinput() {
        let p = presente("hid:x", "c1", 0);
        assert_eq!(modo_de(Some(&p), true, || false), Via::Cabo);
        assert_eq!(modo_de(Some(&p), false, || false), Via::SemFio);
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
        let guardado = pnp::CargaGuardada { percentual: 64, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &com_hora(84)), None);
    }

    #[test]
    fn numero_sem_data_preenche_o_vazio() {
        let guardado = pnp::CargaGuardada { percentual: 64, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &Registro::default()), Some(64));
    }

    #[test]
    fn a_fonte_sem_data_continua_acompanhada() {
        let anterior = Registro { incerto: true, ..com_hora(64) };
        let guardado = pnp::CargaGuardada { percentual: 60, medido_em: None };
        assert_eq!(aceitar_guardada(&guardado, &anterior), Some(60));
    }

    #[test]
    fn numero_com_data_entra_sempre() {
        let guardado = pnp::CargaGuardada { percentual: 64, medido_em: Some(2) };
        assert_eq!(aceitar_guardada(&guardado, &com_hora(84)), Some(64));
    }

    #[test]
    fn o_icone_mostra_quem_esta_pior() {
        let lista = [
            estado("cheio", Some(90), Via::Bluetooth),
            estado("vazio", Some(12), Via::SemFio),
        ];
        assert_eq!(escolher_principal(&lista).unwrap().chave, "vazio");
    }

    #[test]
    fn controle_desligado_nao_rouba_o_icone_de_quem_esta_ligado() {
        let lista = [
            estado("desligado", Some(3), Via::Desligado),
            estado("ligado", Some(80), Via::Bluetooth),
        ];
        assert_eq!(escolher_principal(&lista).unwrap().chave, "ligado");
    }
}
