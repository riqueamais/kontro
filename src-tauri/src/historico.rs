use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::modelo::Via;
use crate::{caminhos, tempo};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AmostraEmDisco {
    #[serde(rename = "T")]
    t: String,
    #[serde(rename = "P")]
    p: i32,
    #[serde(rename = "V", default, skip_serializing_if = "Option::is_none")]
    v: Option<Via>,
    #[serde(rename = "J", default, skip_serializing_if = "Option::is_none")]
    j: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JogoSalvo {
    #[serde(rename = "N")]
    pub nome: String,
    #[serde(rename = "C")]
    pub caminho: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartidaEmDisco {
    #[serde(rename = "I")]
    inicio: String,
    #[serde(rename = "F")]
    fim: String,
    #[serde(rename = "J")]
    jogo: u16,
}

type Aberto = (Vec<JogoSalvo>, Vec<PartidaEmDisco>, HashMap<String, Vec<AmostraEmDisco>>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum EmDisco {
    ComJogos {
        jogos: Vec<JogoSalvo>,
        #[serde(default)]
        partidas: Vec<PartidaEmDisco>,
        controles: HashMap<String, Vec<AmostraEmDisco>>,
    },
    SoControles(HashMap<String, Vec<AmostraEmDisco>>),
}

impl EmDisco {
    fn abrir(self) -> Aberto {
        match self {
            EmDisco::ComJogos { jogos, partidas, controles } => (jogos, partidas, controles),
            EmDisco::SoControles(controles) => (Vec::new(), Vec::new(), controles),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Partida {
    inicio: i64,
    fim: i64,
    jogo: u16,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Amostra {
    pub t: i64,
    pub p: i32,
    pub via: Option<Via>,
    pub jogo: Option<u16>,
}

impl Amostra {
    fn desligado(&self) -> bool {
        self.via == Some(Via::Desligado)
    }

    fn no_cabo(&self) -> bool {
        self.via == Some(Via::Cabo)
    }
}

#[derive(Debug, Default)]
pub struct History {
    por_controle: HashMap<String, Vec<Amostra>>,
    jogos: Vec<JogoSalvo>,
    partidas: Vec<Partida>,
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
const FATIA_QUE_BATIZA_A_SESSAO: f64 = 0.6;
const PAUSA_QUE_NAO_ENCERRA_A_PARTIDA_MS: i64 = 5 * 60 * 1000;
const QUEDA_QUE_SUSTENTA_UMA_PROJECAO: f64 = 5.0;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sessao {
    pub inicio: i64,
    pub fim: i64,
    pub de: i32,
    pub ate: i32,
    pub jogo: Option<String>,
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
        let bruto = caminhos::ler("history.json").unwrap_or_default();
        let (jogos, partidas, mapa) =
            serde_json::from_str::<EmDisco>(&bruto).map(EmDisco::abrir).unwrap_or_default();
        let corte = tempo::agora() - JANELA_MS;

        let lidas = mapa.values().map(|s| s.len()).sum::<usize>() + partidas.len();

        let por_controle = mapa
            .into_iter()
            .map(|(chave, serie)| {
                let mut convertida: Vec<Amostra> = serie
                    .into_iter()
                    .filter_map(|a| {
                        tempo::de_texto(&a.t).map(|t| Amostra { t, p: a.p, via: a.v, jogo: a.j })
                    })
                    .filter(|a| a.t >= corte)
                    .collect();
                convertida.sort_by_key(|a| a.t);
                (chave, convertida)
            })
            .collect();

        let partidas: Vec<Partida> = partidas
            .into_iter()
            .filter_map(|p| {
                Some(Partida {
                    inicio: tempo::de_texto(&p.inicio)?,
                    fim: tempo::de_texto(&p.fim)?,
                    jogo: p.jogo,
                })
            })
            .filter(|p| p.fim >= corte)
            .collect();

        let podou = quantas(&por_controle) + partidas.len() != lidas;

        History { por_controle, jogos, partidas, sujo: podou }
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

    pub fn adicionar(&mut self, chave: &str, percentual: i32, quando: i64, via: Via) {
        let serie = self.por_controle.entry(chave.to_string()).or_default();

        if let Some(u) = serie.last().copied() {
            if quando <= u.t && u.p == percentual {
                return;
            }
            let mudou = u.p != percentual || u.desligado();
            let vencido = quando - u.t > DEZ_MINUTOS_MS;
            if !mudou && !vencido {
                return;
            }
        }

        serie.push(Amostra { t: quando, p: percentual, via: Some(via), jogo: None });
        serie.sort_by_key(|a| a.t);
        self.sujo = true;
    }

    pub fn anotar_jogo(&mut self, quando: i64, jogo: &crate::jogo::Jogo) {
        let indice = self.indice_do_jogo(jogo);
        match self.partidas.last_mut() {
            Some(p) if p.jogo == indice && quando - p.fim <= PAUSA_QUE_NAO_ENCERRA_A_PARTIDA_MS => {
                if quando <= p.fim {
                    return;
                }
                p.fim = quando;
            }
            _ => self.partidas.push(Partida { inicio: quando, fim: quando, jogo: indice }),
        }
        self.sujo = true;
    }

    fn indice_do_jogo(&mut self, jogo: &crate::jogo::Jogo) -> u16 {
        if let Some(i) = self.jogos.iter().position(|j| j.nome == jogo.nome) {
            return i as u16;
        }
        self.jogos.push(JogoSalvo {
            nome: jogo.nome.clone(),
            caminho: jogo.caminho.to_string_lossy().to_string(),
        });
        (self.jogos.len() - 1) as u16
    }

    fn nome_do_jogo(&self, indice: u16) -> Option<&str> {
        self.jogos.get(indice as usize).map(|j| j.nome.as_str())
    }

    pub fn jogos_conhecidos(&self) -> Vec<JogoSalvo> {
        self.jogos.clone()
    }

    pub fn marcar_desligado(&mut self, chave: &str, percentual: i32, quando: i64) {
        let serie = self.por_controle.entry(chave.to_string()).or_default();

        match serie.last() {
            None => return,
            Some(u) if u.desligado() => return,
            Some(u) if quando <= u.t => return,
            _ => {}
        }

        serie.push(Amostra { t: quando, p: percentual, via: Some(Via::Desligado), jogo: None });
        self.sujo = true;
    }

    pub fn salvar(&mut self) {
        caminhos::garantir_dir();
        let controles: HashMap<String, Vec<AmostraEmDisco>> = self
            .por_controle
            .iter()
            .filter(|(_, serie)| !serie.is_empty())
            .map(|(chave, serie)| {
                let convertida = serie
                    .iter()
                    .map(|a| AmostraEmDisco {
                        t: tempo::para_texto(a.t),
                        p: a.p,
                        v: a.via,
                        j: a.jogo,
                    })
                    .collect();
                (chave.clone(), convertida)
            })
            .collect();

        let partidas = self
            .partidas
            .iter()
            .map(|p| PartidaEmDisco {
                inicio: tempo::para_texto(p.inicio),
                fim: tempo::para_texto(p.fim),
                jogo: p.jogo,
            })
            .collect();

        let em_disco = EmDisco::ComJogos { jogos: self.jogos.clone(), partidas, controles };

        if let Ok(t) = serde_json::to_string(&em_disco) {
            if std::fs::write(caminhos::arquivo("history.json"), t).is_ok() {
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
                    jogo: self.jogo_da_faixa(&serie[inicio..=i]),
                });
            }
            i += 1;
        }

        saida.reverse();
        saida
    }

    fn jogo_da_faixa(&self, faixa: &[Amostra]) -> Option<String> {
        let (inicio, fim) = (faixa.first()?.t, faixa.last()?.t);
        if fim <= inicio {
            return None;
        }

        let mut tempo: HashMap<u16, i64> = HashMap::new();
        for p in &self.partidas {
            let dentro = p.fim.min(fim) - p.inicio.max(inicio);
            if dentro > 0 {
                *tempo.entry(p.jogo).or_default() += dentro;
            }
        }
        for par in faixa.windows(2) {
            if let Some(i) = par[0].jogo {
                *tempo.entry(i).or_default() += par[1].t - par[0].t;
            }
        }

        let (indice, ocupou) = tempo.into_iter().max_by_key(|(_, t)| *t)?;
        if (ocupou as f64) / ((fim - inicio) as f64) < FATIA_QUE_BATIZA_A_SESSAO {
            return None;
        }

        self.nome_do_jogo(indice).map(|n| n.to_string())
    }

    pub fn saude(&self, chave: &str) -> Saude {
        let inteira = self.serie(chave);
        let trocada_em = ultima_troca(inteira);
        let serie = desde_a_troca(inteira);
        let agora = tempo::agora();

        let dias = serie.first().map(|a| (agora - a.t) / DIA_MS).unwrap_or(0);

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

    pub fn autonomia_em_minutos(&self, chave: &str, percentual: i32) -> Option<i64> {
        let taxa = self.consumo_por_hora(chave)?;
        let horas = percentual as f64 / taxa;
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
    let limite =
        if anterior.via.is_some() { SALTO_DE_SEGURANCA_MS } else { SALTO_SEM_VIA_GRAVADA_MS };
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
        Amostra { t, p, via: None, jogo: None }
    }

    fn com_via(t: i64, p: i32, via: Via) -> Amostra {
        Amostra { t, p, via: Some(via), jogo: None }
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
        History { por_controle, jogos: Vec::new(), partidas: Vec::new(), sujo: false }
    }

    fn jogo(nome: &str) -> crate::jogo::Jogo {
        crate::jogo::Jogo {
            nome: nome.to_string(),
            caminho: std::path::PathBuf::from(format!("D:/{nome}.exe")),
        }
    }

    fn com_jogos(amostras: Vec<Amostra>, jogos: &[&str]) -> History {
        let mut h = historico(amostras);
        h.jogos = jogos
            .iter()
            .map(|j| JogoSalvo { nome: j.to_string(), caminho: format!("D:/{j}.exe") })
            .collect();
        h
    }

    fn jogando(comeca_h_atras: i64, duracao_h: i64, de: i32, ate: i32, jogo: u16) -> Vec<Amostra> {
        sessao(comeca_h_atras, duracao_h, de, ate)
            .into_iter()
            .map(|a| Amostra { jogo: Some(jogo), ..a })
            .collect()
    }

    #[test]
    fn a_sessao_herda_o_jogo_que_ocupou_ela() {
        let h = com_jogos(jogando(4, 3, 100, 55, 0), &["ELDEN RING"]);
        assert_eq!(h.sessoes("c")[0].jogo.as_deref(), Some("ELDEN RING"));
    }

    #[test]
    fn sessao_dividida_entre_dois_jogos_fica_sem_nome() {
        let mut a = jogando(4, 3, 100, 55, 0);
        for (i, amostra) in a.iter_mut().enumerate() {
            amostra.jogo = Some(if i % 2 == 0 { 0 } else { 1 });
        }
        let h = com_jogos(a, &["ELDEN RING", "HADES"]);
        assert_eq!(h.sessoes("c")[0].jogo, None);
    }

    #[test]
    fn sessao_sem_jogo_nenhum_fica_sem_nome() {
        let h = com_jogos(sessao(4, 3, 100, 55), &["ELDEN RING"]);
        assert_eq!(h.sessoes("c")[0].jogo, None);
    }

    #[test]
    fn a_leitura_rara_com_jogo_vale_pelo_tempo_que_cobre() {
        let inicio = tempo::agora() - 3 * HORA;
        let a = vec![
            com_via(inicio, 80, Via::Bluetooth),
            com_via(inicio + 30 * MINUTO, 80, Via::Bluetooth),
            Amostra { jogo: Some(0), ..com_via(inicio + 37 * MINUTO, 75, Via::Bluetooth) },
            com_via(inicio + 124 * MINUTO, 75, Via::Desligado),
        ];
        let h = com_jogos(a, &["EA SPORTS FC 26"]);
        assert_eq!(
            h.sessoes("c")[0].jogo.as_deref(),
            Some("EA SPORTS FC 26"),
            "uma amostra em quatro, mas 87 dos 124 minutos"
        );
    }

    #[test]
    fn a_sessao_leva_o_nome_da_partida_que_ocupou_ela() {
        let agora = tempo::agora();
        let mut h = com_jogos(sessao(4, 3, 100, 55), &["ELDEN RING"]);
        h.partidas.push(Partida { inicio: agora - 230 * MINUTO, fim: agora - HORA, jogo: 0 });
        assert_eq!(h.sessoes("c")[0].jogo.as_deref(), Some("ELDEN RING"));
    }

    #[test]
    fn sessao_dividida_entre_duas_partidas_fica_sem_nome() {
        let agora = tempo::agora();
        let mut h = com_jogos(sessao(4, 3, 100, 55), &["ELDEN RING", "HADES"]);
        h.partidas.push(Partida { inicio: agora - 4 * HORA, fim: agora - 150 * MINUTO, jogo: 0 });
        h.partidas.push(Partida { inicio: agora - 150 * MINUTO, fim: agora - HORA, jogo: 1 });
        assert_eq!(h.sessoes("c")[0].jogo, None);
    }

    #[test]
    fn a_partida_de_outro_dia_nao_batiza_a_sessao() {
        let agora = tempo::agora();
        let mut h = com_jogos(sessao(4, 3, 100, 55), &["ELDEN RING"]);
        h.partidas.push(Partida { inicio: agora - 30 * HORA, fim: agora - 26 * HORA, jogo: 0 });
        assert_eq!(h.sessoes("c")[0].jogo, None);
    }

    #[test]
    fn a_partida_cresce_enquanto_o_jogo_segue_em_foco() {
        let mut h = historico(Vec::new());
        let agora = tempo::agora();
        for ciclo in 0..10 {
            h.anotar_jogo(agora + ciclo * 2_000, &jogo("ELDEN RING"));
        }
        assert_eq!(h.partidas.len(), 1);
        assert_eq!(h.partidas[0].fim - h.partidas[0].inicio, 18_000);
    }

    #[test]
    fn um_alt_tab_curto_nao_encerra_a_partida() {
        let mut h = historico(Vec::new());
        let agora = tempo::agora();
        h.anotar_jogo(agora, &jogo("ELDEN RING"));
        h.anotar_jogo(agora + 3 * MINUTO, &jogo("ELDEN RING"));
        assert_eq!(h.partidas.len(), 1);
    }

    #[test]
    fn ficar_longe_do_jogo_abre_outra_partida() {
        let mut h = historico(Vec::new());
        let agora = tempo::agora();
        h.anotar_jogo(agora, &jogo("ELDEN RING"));
        h.anotar_jogo(agora + 20 * MINUTO, &jogo("ELDEN RING"));
        assert_eq!(h.partidas.len(), 2);
    }

    #[test]
    fn as_partidas_voltam_do_disco() {
        let bruto = r#"{"jogos":[{"N":"HADES","C":"D:/hades.exe"}],"partidas":[{"I":"2026-09-10T19:00:00-03:00","F":"2026-09-10T21:00:00-03:00","J":0}],"controles":{}}"#;
        let (_, partidas, _) = serde_json::from_str::<EmDisco>(bruto).map(EmDisco::abrir).unwrap();
        assert_eq!(partidas.len(), 1);
        assert_eq!(partidas[0].jogo, 0);
        let inicio = tempo::de_texto(&partidas[0].inicio).unwrap();
        let fim = tempo::de_texto(&partidas[0].fim).unwrap();
        assert_eq!(fim - inicio, 2 * HORA);
    }

    #[test]
    fn um_history_json_da_versao_anterior_continua_abrindo() {
        let antigo = r#"{"wired":[{"T":"2026-09-01T10:00:00Z","P":80,"V":"SemFio"}]}"#;
        let (jogos, partidas, mapa) =
            serde_json::from_str::<EmDisco>(antigo).map(EmDisco::abrir).unwrap();
        assert!(jogos.is_empty());
        assert!(partidas.is_empty());
        assert_eq!(mapa["wired"].len(), 1);
        assert_eq!(mapa["wired"][0].j, None);
    }

    #[test]
    fn o_arquivo_novo_traz_a_tabela_de_jogos() {
        let novo = r#"{"jogos":[{"N":"ELDEN RING","C":"D:/er.exe"}],"controles":{"wired":[{"T":"2026-09-01T10:00:00Z","P":80,"J":0}]}}"#;
        let (jogos, partidas, mapa) =
            serde_json::from_str::<EmDisco>(novo).map(EmDisco::abrir).unwrap();
        assert!(partidas.is_empty(), "o arquivo da 2.13 nao tinha partidas");
        assert_eq!(jogos[0].nome, "ELDEN RING");
        assert_eq!(jogos[0].caminho, "D:/er.exe");
        assert_eq!(mapa["wired"][0].j, Some(0));
    }

    #[test]
    fn o_mesmo_jogo_entra_uma_vez_so_na_tabela() {
        let mut h = historico(Vec::new());
        let agora = tempo::agora();
        h.anotar_jogo(agora, &jogo("ELDEN RING"));
        h.anotar_jogo(agora + MINUTO, &jogo("HADES"));
        h.anotar_jogo(agora + 2 * MINUTO, &jogo("ELDEN RING"));
        assert_eq!(h.jogos.len(), 2);
        assert_eq!(h.partidas.iter().map(|p| p.jogo).collect::<Vec<_>>(), vec![0, 1, 0]);
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
        let a = vec![amostra(agora - 3 * HORA, 70), amostra(agora - 3 * HORA + 5 * MINUTO, 69)];
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
        let a = vec![amostra(agora - 16 * MINUTO, 84), amostra(agora, 83)];
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
    fn o_arquivo_de_antes_da_via_continua_sendo_lido() {
        let bruto = r#"{"408e2c82242f":[{"T":"2026-08-20T00:28:00-03:00","P":73}]}"#;
        let mapa: HashMap<String, Vec<AmostraEmDisco>> = serde_json::from_str(bruto).unwrap();

        let serie = &mapa["408e2c82242f"];
        assert_eq!(serie.len(), 1);
        assert_eq!(serie[0].p, 73);
        assert_eq!(serie[0].v, None, "amostra sem V nao pode virar Desligado");
    }

    #[test]
    fn a_amostra_sem_via_nao_ganha_o_campo_ao_ser_gravada() {
        let sem = AmostraEmDisco { t: "2026-08-20T00:28:00-03:00".into(), p: 73, v: None, j: None };
        let com = AmostraEmDisco {
            t: "2026-08-20T00:28:00-03:00".into(),
            p: 73,
            v: Some(Via::Bluetooth),
            j: None,
        };

        assert!(!serde_json::to_string(&sem).unwrap().contains("\"V\""));
        assert!(serde_json::to_string(&com).unwrap().contains("\"V\":\"Bluetooth\""));
    }

    #[test]
    fn carregar_no_cabo_nao_e_trocar() {
        let agora = tempo::agora();
        let a = vec![
            com_via(agora - 20 * MINUTO, 12, Via::Cabo),
            com_via(agora - 18 * MINUTO, 40, Via::Cabo),
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
            com_via(agora - 60 * MINUTO, 70, Via::Bluetooth),
            com_via(agora - 50 * MINUTO, 70, Via::Desligado),
            com_via(agora - 45 * MINUTO, 65, Via::Bluetooth),
            com_via(agora - 5 * MINUTO, 60, Via::Bluetooth),
        ];
        let s = historico(a).sessoes("c");
        assert_eq!(s.len(), 2, "o desligamento gravado separa as duas");
        assert_eq!((s[0].de, s[0].ate), (65, 60));
    }

    #[test]
    fn com_a_via_gravada_meia_hora_sem_leitura_nao_quebra_a_sessao() {
        let agora = tempo::agora();
        let a = vec![
            com_via(agora - 60 * MINUTO, 70, Via::Bluetooth),
            com_via(agora - 15 * MINUTO, 65, Via::Bluetooth),
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
        let mut h = historico(vec![com_via(agora - 30 * MINUTO, 70, Via::Bluetooth)]);
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
