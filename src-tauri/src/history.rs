use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::LinkMode;
use crate::{paths, tempo};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AmostraEmDisco {
    #[serde(rename = "T")]
    t: String,
    #[serde(rename = "P")]
    p: i32,
    #[serde(rename = "V", default, skip_serializing_if = "Option::is_none")]
    v: Option<LinkMode>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Amostra {
    pub t: i64,
    pub p: i32,
    pub via: Option<LinkMode>,
}

impl Amostra {
    fn desligado(&self) -> bool {
        self.via == Some(LinkMode::Offline)
    }

    fn no_cabo(&self) -> bool {
        self.via == Some(LinkMode::Cable)
    }
}

#[derive(Debug, Default)]
pub struct History {
    por_controle: HashMap<String, Vec<Amostra>>,
    sujo: bool,
}

const JANELA_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const DEZ_MINUTOS_MS: i64 = 10 * 60 * 1000;
const SEMANA_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const DIA_MS: i64 = 24 * 60 * 60 * 1000;

const DIAS_PARA_COMPARAR: i64 = 14;
const HORAS_MINIMAS_POR_JANELA: f64 = 2.0;
const VARIACAO_QUE_IMPORTA: f64 = 15.0;
const SALTO_SEM_VIA_GRAVADA_MS: i64 = 30 * 60 * 1000;
const SALTO_DE_SEGURANCA_MS: i64 = 6 * 60 * 60 * 1000;
const SUBIDA_QUE_DENUNCIA_TROCA: i32 = 15;
const SUBIDA_INSTANTANEA_MS: i64 = 5 * 60 * 1000;
const DURACAO_MINIMA_DE_SESSAO_MS: i64 = 10 * 60 * 1000;
const QUEDA_QUE_SUSTENTA_UMA_PROJECAO: f64 = 5.0;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sessao {
    pub inicio: i64,
    pub fim: i64,
    pub de: i32,
    pub ate: i32,
}

#[derive(Debug, Clone, Copy)]
struct Descarga {
    fim: i64,
    queda: f64,
    horas: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Saude {
    pub estado: &'static str,
    pub dias: i64,
    pub consumo_recente: Option<f64>,
    pub consumo_antes: Option<f64>,
    pub variacao: Option<i32>,
    pub trocada_em: Option<i64>,
    pub carga_cheia_minutos: Option<i64>,
}

impl History {
    pub fn carregar() -> Self {
        let bruto = paths::ler("history.json").unwrap_or_default();
        let mapa: HashMap<String, Vec<AmostraEmDisco>> =
            serde_json::from_str(&bruto).unwrap_or_default();
        let corte = tempo::agora() - JANELA_MS;

        let lidas: usize = mapa.values().map(|s| s.len()).sum();

        let por_controle = mapa
            .into_iter()
            .map(|(chave, serie)| {
                let mut convertida: Vec<Amostra> = serie
                    .into_iter()
                    .filter_map(|a| tempo::de_texto(&a.t).map(|t| Amostra { t, p: a.p, via: a.v }))
                    .filter(|a| a.t >= corte)
                    .collect();
                convertida.sort_by_key(|a| a.t);
                (chave, convertida)
            })
            .collect();

        let podou = quantas(&por_controle) != lidas;

        History { por_controle, sujo: podou }
    }

    pub fn precisa_salvar(&self) -> bool {
        self.sujo
    }

    pub fn ultimo(&self, chave: &str) -> Option<Amostra> {
        self.por_controle.get(chave).and_then(|s| s.last().copied())
    }

    pub fn serie(&self, chave: &str) -> &[Amostra] {
        self.por_controle.get(chave).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn esquecer(&mut self, chave: &str) {
        if self.por_controle.remove(chave).is_some() {
            self.sujo = true;
            self.salvar();
        }
    }

    pub fn adicionar(&mut self, chave: &str, percent: i32, quando: i64, via: LinkMode) {
        let serie = self.por_controle.entry(chave.to_string()).or_default();

        if let Some(u) = serie.last().copied() {
            if quando <= u.t && u.p == percent {
                return;
            }
            let mudou = u.p != percent || u.desligado();
            let vencido = quando - u.t > DEZ_MINUTOS_MS;
            if !mudou && !vencido {
                return;
            }
        }

        serie.push(Amostra { t: quando, p: percent, via: Some(via) });
        serie.sort_by_key(|a| a.t);
        self.sujo = true;
    }

    pub fn marcar_desligado(&mut self, chave: &str, percent: i32, quando: i64) {
        let serie = self.por_controle.entry(chave.to_string()).or_default();

        match serie.last() {
            None => return,
            Some(u) if u.desligado() => return,
            Some(u) if quando <= u.t => return,
            _ => {}
        }

        serie.push(Amostra { t: quando, p: percent, via: Some(LinkMode::Offline) });
        self.sujo = true;
    }

    pub fn salvar(&mut self) {
        paths::garantir_dir();
        let em_disco: HashMap<&String, Vec<AmostraEmDisco>> = self
            .por_controle
            .iter()
            .filter(|(_, serie)| !serie.is_empty())
            .map(|(chave, serie)| {
                let convertida = serie
                    .iter()
                    .map(|a| AmostraEmDisco { t: tempo::para_texto(a.t), p: a.p, v: a.via })
                    .collect();
                (chave, convertida)
            })
            .collect();

        if let Ok(t) = serde_json::to_string(&em_disco) {
            if std::fs::write(paths::arquivo("history.json"), t).is_ok() {
                self.sujo = false;
            }
        }
    }

    pub fn consumo_por_hora(&self, chave: &str) -> Option<f64> {
        let serie = desde_a_troca(self.serie(chave));
        let trechos = descargas(serie);
        let fim_da_serie = serie.last()?.t;

        let de_agora = trechos.iter().find(|d| d.fim == fim_da_serie);
        if let Some(d) = de_agora.filter(|d| d.queda >= QUEDA_QUE_SUSTENTA_UMA_PROJECAO) {
            return Some(d.queda / d.horas);
        }

        let corte = tempo::agora() - SEMANA_MS;
        taxa(trechos.iter().filter(|d| d.fim >= corte))
    }

    pub fn sessoes(&self, chave: &str) -> Vec<Sessao> {
        let serie = self.serie(chave);
        let mut saida = Vec::new();
        let mut i = 0;

        while i < serie.len() {
            let inicio = i;
            while i + 1 < serie.len() && !quebra(&serie[i], &serie[i + 1]) {
                i += 1;
            }
            if serie[i].t - serie[inicio].t >= DURACAO_MINIMA_DE_SESSAO_MS {
                saida.push(Sessao {
                    inicio: serie[inicio].t,
                    fim: serie[i].t,
                    de: serie[inicio].p,
                    ate: serie[i].p,
                });
            }
            i += 1;
        }

        saida.reverse();
        saida
    }

    pub fn saude(&self, chave: &str) -> Saude {
        let inteira = self.serie(chave);
        let trocada_em = ultima_troca(inteira);
        let serie = desde_a_troca(inteira);
        let agora = tempo::agora();

        let dias = serie
            .first()
            .map(|a| (agora - a.t) / DIA_MS)
            .unwrap_or(0);

        let carga_cheia_minutos = self.autonomia_em_minutos(chave, 100);

        let medindo = Saude {
            estado: "medindo",
            dias,
            consumo_recente: None,
            consumo_antes: None,
            variacao: None,
            trocada_em,
            carga_cheia_minutos,
        };

        if dias < DIAS_PARA_COMPARAR {
            return medindo;
        }

        let corte = agora - SEMANA_MS;
        let descargas = descargas(serie);

        let Some(recente) = taxa(descargas.iter().filter(|d| d.fim >= corte)) else {
            return medindo;
        };
        let Some(antes) = taxa(descargas.iter().filter(|d| d.fim < corte)) else {
            return medindo;
        };

        let variacao = (recente - antes) / antes * 100.0;

        Saude {
            estado: if variacao.abs() < VARIACAO_QUE_IMPORTA {
                "estavel"
            } else if variacao > 0.0 {
                "piorando"
            } else {
                "melhorando"
            },
            dias,
            consumo_recente: Some(recente),
            consumo_antes: Some(antes),
            variacao: Some(variacao.round() as i32),
            trocada_em,
            carga_cheia_minutos,
        }
    }

    pub fn autonomia_em_minutos(&self, chave: &str, percent: i32) -> Option<i64> {
        let taxa = self.consumo_por_hora(chave)?;
        let horas = percent as f64 / taxa;
        (horas > 0.0 && horas <= 200.0).then(|| (horas * 60.0).round() as i64)
    }
}

fn quantas(por_controle: &HashMap<String, Vec<Amostra>>) -> usize {
    por_controle.values().map(|s| s.len()).sum()
}

fn quebra(anterior: &Amostra, seguinte: &Amostra) -> bool {
    if anterior.desligado() {
        return true;
    }
    let limite = if anterior.via.is_some() {
        SALTO_DE_SEGURANCA_MS
    } else {
        SALTO_SEM_VIA_GRAVADA_MS
    };
    seguinte.t - anterior.t > limite
}

fn troca_de_bateria(serie: &[Amostra], i: usize) -> bool {
    let (antes, depois) = (serie[i], serie[i + 1]);

    if depois.p - antes.p < SUBIDA_QUE_DENUNCIA_TROCA {
        return false;
    }
    if depois.t - antes.t > SUBIDA_INSTANTANEA_MS {
        return false;
    }
    if antes.no_cabo() || depois.no_cabo() {
        return false;
    }
    let vinha_subindo = i > 0 && antes.p > serie[i - 1].p;
    let segue_subindo = i + 2 < serie.len() && serie[i + 2].p > depois.p;
    !vinha_subindo && !segue_subindo
}

fn ultima_troca(serie: &[Amostra]) -> Option<i64> {
    (0..serie.len().saturating_sub(1))
        .rev()
        .find(|&i| troca_de_bateria(serie, i))
        .map(|i| serie[i + 1].t)
}

fn desde_a_troca(serie: &[Amostra]) -> &[Amostra] {
    let Some(quando) = ultima_troca(serie) else { return serie };
    let corte = serie.iter().position(|a| a.t >= quando).unwrap_or(0);
    &serie[corte..]
}

fn descargas(serie: &[Amostra]) -> Vec<Descarga> {
    let mut saida = Vec::new();
    let mut i = 0;

    while i + 1 < serie.len() {
        if serie[i + 1].p >= serie[i].p || quebra(&serie[i], &serie[i + 1]) {
            i += 1;
            continue;
        }

        let inicio = i;
        let mut fim = i + 1;
        while fim + 1 < serie.len()
            && serie[fim + 1].p <= serie[fim].p
            && !quebra(&serie[fim], &serie[fim + 1])
        {
            fim += 1;
        }

        let queda = (serie[inicio].p - serie[fim].p) as f64;
        let horas = (serie[fim].t - serie[inicio].t) as f64 / 3_600_000.0;
        if queda > 0.0 && horas >= 0.25 {
            let taxa = queda / horas;
            if (0.1..=60.0).contains(&taxa) {
                saida.push(Descarga { fim: serie[fim].t, queda, horas });
            }
        }

        i = fim;
    }

    saida
}

fn taxa<'a>(trechos: impl Iterator<Item = &'a Descarga>) -> Option<f64> {
    let mut queda = 0.0;
    let mut horas = 0.0;
    for d in trechos {
        queda += d.queda;
        horas += d.horas;
    }
    (horas >= HORAS_MINIMAS_POR_JANELA && queda > 0.0).then(|| queda / horas)
}

#[cfg(test)]
mod testes {
    use super::*;

    const MINUTO: i64 = 60_000;
    const HORA: i64 = 60 * MINUTO;

    fn amostra(t: i64, p: i32) -> Amostra {
        Amostra { t, p, via: None }
    }

    fn com_via(t: i64, p: i32, via: LinkMode) -> Amostra {
        Amostra { t, p, via: Some(via) }
    }

    fn sessao(comeca_h_atras: i64, duracao_h: i64, de: i32, ate: i32) -> Vec<Amostra> {
        let agora = tempo::agora();
        let passos = duracao_h * 6;
        (0..=passos)
            .map(|i| {
                amostra(
                    agora - comeca_h_atras * HORA + i * 10 * MINUTO,
                    de + ((ate - de) as i64 * i / passos) as i32,
                )
            })
            .collect()
    }

    fn historico(mut amostras: Vec<Amostra>) -> History {
        amostras.sort_by_key(|a| a.t);
        let mut por_controle = HashMap::new();
        por_controle.insert("c".to_string(), amostras);
        History { por_controle, sujo: false }
    }

    #[test]
    fn cada_sessao_e_um_periodo_com_o_controle_ligado() {
        let mut a = sessao(30, 2, 80, 60);
        a.extend(sessao(4, 3, 100, 55));

        let s = historico(a).sessoes("c");
        assert_eq!(s.len(), 2, "as duas viraram uma so");
        assert_eq!((s[0].de, s[0].ate), (100, 55), "a mais recente vem primeiro");
        assert_eq!((s[1].de, s[1].ate), (80, 60));
    }

    #[test]
    fn um_piscar_de_conexao_nao_e_sessao() {
        let agora = tempo::agora();
        let a = vec![
            amostra(agora - 3 * HORA, 70),
            amostra(agora - 3 * HORA + 5 * MINUTO, 69),
        ];
        assert!(historico(a).sessoes("c").is_empty());
    }

    #[test]
    fn a_carga_quebra_o_trecho() {
        let mut a = sessao(10, 4, 100, 60);
        a.extend(sessao(5, 2, 60, 90));
        a.extend(sessao(3, 2, 90, 70));

        let h = historico(a);
        let trechos = descargas(h.serie("c"));

        assert_eq!(trechos.len(), 2, "a subida no meio separa as duas descargas");
        assert_eq!(trechos[0].queda, 40.0);
        assert_eq!(trechos[1].queda, 20.0);
    }

    #[test]
    fn o_controle_desligado_nao_conta_como_consumo() {
        let mut a = sessao(30, 2, 80, 70);
        a.extend(sessao(4, 2, 70, 60));

        let h = historico(a);
        let trechos = descargas(h.serie("c"));

        assert_eq!(trechos.len(), 2, "duas sessoes, e nao uma de 26 horas");
        let horas: f64 = trechos.iter().map(|d| d.horas).sum();
        assert!(horas < 5.0, "as 24h desligado entraram na conta: {horas}");
    }

    #[test]
    fn a_noite_desligado_nao_entra_no_consumo() {
        let mut a = sessao(30, 2, 80, 70);
        a.extend(sessao(4, 2, 70, 60));

        let taxa = historico(a).consumo_por_hora("c").expect("ha descarga medida");
        assert!((4.0..7.0).contains(&taxa), "a conta pegou o buraco entre as sessoes: {taxa}");
    }

    #[test]
    fn o_consumo_de_agora_manda_sobre_o_de_antes() {
        let mut a = sessao(40, 4, 100, 90);
        a.extend(sessao(3, 2, 90, 70));

        let taxa = historico(a).consumo_por_hora("c").expect("ha descarga medida");
        assert!((9.0..11.0).contains(&taxa), "esperava o trecho de agora: {taxa}");
    }

    #[test]
    fn no_cabo_vale_a_media_da_semana() {
        let mut a = sessao(30, 3, 90, 60);
        a.extend(sessao(5, 2, 60, 90));

        let taxa = historico(a).consumo_por_hora("c").expect("ha descarga na semana");
        assert!((9.0..12.0).contains(&taxa), "esperava a media da semana: {taxa}");
    }

    #[test]
    fn um_ponto_de_queda_nao_sustenta_uma_projecao() {
        let agora = tempo::agora();
        let a = vec![
            amostra(agora - 16 * MINUTO, 84),
            amostra(agora, 83),
        ];
        assert_eq!(
            historico(a).consumo_por_hora("c"),
            None,
            "dezesseis minutos parado viraram uma projecao de vinte e duas horas"
        );
    }

    #[test]
    fn sem_descarga_nenhuma_nao_ha_o_que_dizer() {
        let a = sessao(3, 2, 60, 90);
        assert_eq!(historico(a).consumo_por_hora("c"), None);
    }

    fn com_troca(momento_h_atras: i64, para: i32) -> Amostra {
        amostra(tempo::agora() - momento_h_atras * HORA + MINUTO, para)
    }

    fn duas_baterias(com_a_troca: bool) -> Vec<Amostra> {
        let mut a = Vec::new();
        for dia in [29i64, 27, 25, 23, 21, 19, 17, 15, 13, 11] {
            a.extend(sessao(dia * 24, 4, 100, 60));
        }
        if com_a_troca {
            a.push(com_troca(11 * 24 - 4, 100));
        }
        for dia in [9i64, 5, 4, 3, 2, 1] {
            a.extend(sessao(dia * 24, 4, 100, 80));
        }
        a
    }

    #[test]
    fn a_bateria_nova_recomeca_a_medicao() {
        let s = historico(duas_baterias(true)).saude("c");
        assert_eq!(s.estado, "medindo", "comparou uma bateria com a outra");
        assert!(s.dias < DIAS_PARA_COMPARAR, "a idade veio da bateria que saiu: {}", s.dias);
        assert!(s.trocada_em.is_some());
    }

    #[test]
    fn sem_troca_o_mesmo_historico_da_veredito() {
        let s = historico(duas_baterias(false)).saude("c");
        assert_eq!(s.estado, "melhorando", "sem a troca ha o que comparar");
        assert_eq!(s.trocada_em, None);
    }

    #[test]
    fn carregar_no_cabo_nao_e_trocar() {
        let agora = tempo::agora();
        let a = vec![
            com_via(agora - 20 * MINUTO, 12, LinkMode::Cable),
            com_via(agora - 18 * MINUTO, 40, LinkMode::Cable),
        ];
        assert_eq!(
            ultima_troca(&historico(a).por_controle["c"]),
            None,
            "a subida aconteceu com o cabo na mao, e o app estava vendo"
        );
    }

    #[test]
    fn a_subida_encadeada_de_uma_carga_nao_e_troca() {
        let agora = tempo::agora();
        let a = vec![
            amostra(agora - 130 * MINUTO, 12),
            amostra(agora - 61 * MINUTO, 51),
            amostra(agora - 60 * MINUTO, 87),
            amostra(agora - 50 * MINUTO, 86),
        ];
        assert_eq!(
            ultima_troca(&historico(a).por_controle["c"]),
            None,
            "12 -> 51 -> 87 e uma carga subindo, nao uma pilha nova no meio dela"
        );
    }

    #[test]
    fn desligar_quebra_a_sessao_mesmo_sem_buraco_no_relogio() {
        let agora = tempo::agora();
        let a = vec![
            com_via(agora - 60 * MINUTO, 70, LinkMode::Bluetooth),
            com_via(agora - 50 * MINUTO, 70, LinkMode::Offline),
            com_via(agora - 45 * MINUTO, 65, LinkMode::Bluetooth),
            com_via(agora - 5 * MINUTO, 60, LinkMode::Bluetooth),
        ];
        let s = historico(a).sessoes("c");
        assert_eq!(s.len(), 2, "o desligamento gravado separa as duas");
        assert_eq!((s[0].de, s[0].ate), (65, 60));
    }

    #[test]
    fn com_a_via_gravada_meia_hora_sem_leitura_nao_quebra_a_sessao() {
        let agora = tempo::agora();
        let a = vec![
            com_via(agora - 60 * MINUTO, 70, LinkMode::Bluetooth),
            com_via(agora - 15 * MINUTO, 65, LinkMode::Bluetooth),
        ];
        let s = historico(a).sessoes("c");
        assert_eq!(s.len(), 1, "o GATT so avisa quando muda: 45 min calado e uso, nao ausencia");
        assert_eq!(s[0].fim - s[0].inicio, 45 * MINUTO);
    }

    #[test]
    fn sem_via_gravada_o_relogio_continua_valendo() {
        let agora = tempo::agora();
        let a = vec![amostra(agora - 60 * MINUTO, 70), amostra(agora - 15 * MINUTO, 65)];
        assert!(
            historico(a).sessoes("c").is_empty(),
            "serie antiga nao tem a via, entao ela mantem a regra dos 30 min"
        );
    }

    #[test]
    fn o_marcador_de_desligado_nao_se_repete() {
        let agora = tempo::agora();
        let mut h = historico(vec![com_via(agora - 30 * MINUTO, 70, LinkMode::Bluetooth)]);
        h.marcar_desligado("c", 70, agora - 20 * MINUTO);
        h.marcar_desligado("c", 70, agora - 10 * MINUTO);
        assert_eq!(h.serie("c").len(), 2);
    }

    #[test]
    fn nao_ha_o_que_marcar_num_controle_sem_serie() {
        let mut h = History::default();
        h.marcar_desligado("c", 70, tempo::agora());
        assert!(h.serie("c").is_empty());
    }

    #[test]
    fn carregar_devagar_nao_e_trocar() {
        let mut a = sessao(30, 3, 90, 60);
        a.extend(sessao(20, 2, 60, 95));
        assert_eq!(ultima_troca(&historico(a).por_controle["c"]), None);
    }

    #[test]
    fn o_consumo_ignora_a_bateria_que_saiu() {
        let mut a = sessao(30, 3, 60, 12);
        a.push(com_troca(27, 90));
        a.extend(sessao(20, 2, 90, 80));
        a.extend(sessao(9, 3, 80, 60));

        let taxa = historico(a).consumo_por_hora("c").expect("ha descarga medida");
        assert!(taxa < 9.0, "a media pegou a bateria velha: {taxa}");
    }

    #[test]
    fn diz_quanto_dura_uma_carga_cheia() {
        let a = sessao(4, 3, 90, 60);
        let s = historico(a).saude("c");
        let minutos = s.carga_cheia_minutos.expect("ha consumo medido");
        assert!((520..=640).contains(&minutos), "dez pontos por hora dariam ~10 h: {minutos}");
    }

    #[test]
    fn sem_consumo_medido_nao_inventa_a_carga_cheia() {
        let a = sessao(3, 2, 60, 90);
        assert_eq!(historico(a).saude("c").carga_cheia_minutos, None);
    }

    #[test]
    fn poucos_dias_nao_dao_veredito() {
        let mut a = sessao(48, 4, 100, 60);
        a.extend(sessao(24, 4, 100, 60));

        let s = historico(a).saude("c");
        assert_eq!(s.estado, "medindo");
        assert_eq!(s.variacao, None);
    }

    #[test]
    fn consumo_maior_agora_e_piora() {
        let mut a = Vec::new();
        for dia in [29i64, 27, 25, 23, 21, 19, 17, 15, 13, 11, 9] {
            a.extend(sessao(dia * 24, 4, 100, 60));
        }
        for dia in [5i64, 4, 3, 2, 1] {
            a.extend(sessao(dia * 24, 4, 100, 20));
        }

        let s = historico(a).saude("c");
        assert_eq!(s.estado, "piorando");
        assert_eq!(s.variacao, Some(100), "10 pontos/h contra 20 pontos/h");
    }

    #[test]
    fn consumo_parecido_e_estavel() {
        let mut a = Vec::new();
        for dia in [29i64, 27, 25, 23, 21, 19, 17, 15, 13, 11, 9, 5, 4, 3, 2, 1] {
            a.extend(sessao(dia * 24, 4, 100, 60));
        }

        let s = historico(a).saude("c");
        assert_eq!(s.estado, "estavel");
        assert_eq!(s.variacao, Some(0));
    }

    #[test]
    fn sem_dados_na_semana_volta_a_medir() {
        let mut a = Vec::new();
        for dia in [29i64, 27, 25, 23, 21, 19, 17] {
            a.extend(sessao(dia * 24, 4, 100, 60));
        }

        let s = historico(a).saude("c");
        assert_eq!(s.estado, "medindo", "sem descarga recente nao da para comparar");
    }
}
