//! Historico de carga por controle, e a estimativa de autonomia.
//!
//! So grava quando o valor muda ou a cada dez minutos: registrar cada leitura faria o
//! arquivo crescer sem acrescentar informacao. O formato em disco e o mesmo da versao em
//! .NET -- data como texto ISO -- para que o usuario nao perca a serie ao atualizar.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{paths, tempo};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AmostraEmDisco {
    #[serde(rename = "T")]
    t: String,
    #[serde(rename = "P")]
    p: i32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Amostra {
    pub t: i64,
    pub p: i32,
}

#[derive(Debug, Default)]
pub struct History {
    por_controle: HashMap<String, Vec<Amostra>>,
}

/// Trinta dias: o suficiente para uma media de consumo honesta sem virar arquivao.
const JANELA_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const DEZ_MINUTOS_MS: i64 = 10 * 60 * 1000;
const SEMANA_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const DIA_MS: i64 = 24 * 60 * 60 * 1000;

const DIAS_PARA_COMPARAR: i64 = 14;
const HORAS_MINIMAS_POR_JANELA: f64 = 2.0;
const VARIACAO_QUE_IMPORTA: f64 = 15.0;
const SALTO_QUE_QUEBRA_O_TRECHO_MS: i64 = 30 * 60 * 1000;

#[derive(Debug, Clone, Copy)]
struct Descarga {
    fim: i64,
    queda: f64,
    horas: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Saude {
    pub estado: &'static str,
    pub dias: i64,
    pub consumo_recente: Option<f64>,
    pub consumo_antes: Option<f64>,
    pub variacao: Option<i32>,
}

impl History {
    pub fn carregar() -> Self {
        let bruto = std::fs::read_to_string(paths::arquivo("history.json")).unwrap_or_default();
        let mapa: HashMap<String, Vec<AmostraEmDisco>> =
            serde_json::from_str(&bruto).unwrap_or_default();
        let corte = tempo::agora() - JANELA_MS;

        let por_controle = mapa
            .into_iter()
            .map(|(chave, serie)| {
                let mut convertida: Vec<Amostra> = serie
                    .into_iter()
                    .filter_map(|a| tempo::de_texto(&a.t).map(|t| Amostra { t, p: a.p }))
                    .filter(|a| a.t >= corte)
                    .collect();
                convertida.sort_by_key(|a| a.t);
                (chave, convertida)
            })
            .collect();

        History { por_controle }
    }

    pub fn ultimo(&self, chave: &str) -> Option<Amostra> {
        self.por_controle.get(chave).and_then(|s| s.last().copied())
    }

    pub fn serie(&self, chave: &str) -> &[Amostra] {
        self.por_controle.get(chave).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Apaga a serie de um controle esquecido.
    ///
    /// Guardar o historico de um aparelho que saiu da lista deixaria a media de consumo
    /// de um controle futuro contaminada pela bateria de outro, caso a chave se repita.
    pub fn esquecer(&mut self, chave: &str) {
        if self.por_controle.remove(chave).is_some() {
            self.salvar();
        }
    }

    pub fn adicionar(&mut self, chave: &str, percent: i32, quando: i64) {
        let serie = self.por_controle.entry(chave.to_string()).or_default();

        if let Some(u) = serie.last().copied() {
            if quando <= u.t && u.p == percent {
                return;
            }
            let mudou = u.p != percent;
            let vencido = quando - u.t > DEZ_MINUTOS_MS;
            if !mudou && !vencido {
                return;
            }
        }

        serie.push(Amostra { t: quando, p: percent });
        serie.sort_by_key(|a| a.t);
    }

    pub fn salvar(&self) {
        paths::garantir_dir();
        let em_disco: HashMap<&String, Vec<AmostraEmDisco>> = self
            .por_controle
            .iter()
            .map(|(chave, serie)| {
                let convertida = serie
                    .iter()
                    .map(|a| AmostraEmDisco { t: tempo::para_texto(a.t), p: a.p })
                    .collect();
                (chave, convertida)
            })
            .collect();

        if let Ok(t) = serde_json::to_string(&em_disco) {
            let _ = std::fs::write(paths::arquivo("history.json"), t);
        }
    }

    /// Consumo em pontos percentuais por hora, medido no trecho recente em que a carga
    /// so caiu. Trecho que sobe e carga, nao consumo, e por isso nao entra na conta.
    pub fn consumo_por_hora(&self, chave: &str) -> Option<f64> {
        let serie = self.serie(chave);
        if serie.len() < 2 {
            return None;
        }

        let fim = serie.len() - 1;
        let mut inicio = fim;
        while inicio > 0 && serie[inicio - 1].p >= serie[inicio].p {
            inicio -= 1;
        }
        if inicio == fim {
            return None;
        }

        let queda = (serie[inicio].p - serie[fim].p) as f64;
        let horas = (serie[fim].t - serie[inicio].t) as f64 / 3_600_000.0;
        if queda <= 0.0 || horas < 0.25 {
            return None;
        }

        // consumo absurdo denuncia leitura ruim, nao bateria ruim
        let taxa = queda / horas;
        (0.1..=60.0).contains(&taxa).then_some(taxa)
    }

    pub fn saude(&self, chave: &str) -> Saude {
        let serie = self.serie(chave);
        let agora = tempo::agora();

        let dias = serie
            .first()
            .map(|a| (agora - a.t) / DIA_MS)
            .unwrap_or(0);

        let medindo = Saude {
            estado: "medindo",
            dias,
            consumo_recente: None,
            consumo_antes: None,
            variacao: None,
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
        }
    }

    /// Quanto tempo de jogo ainda cabe, em minutos.
    pub fn autonomia_em_minutos(&self, chave: &str, percent: i32) -> Option<i64> {
        let taxa = self.consumo_por_hora(chave)?;
        let horas = percent as f64 / taxa;
        (horas > 0.0 && horas <= 200.0).then(|| (horas * 60.0).round() as i64)
    }
}

fn descargas(serie: &[Amostra]) -> Vec<Descarga> {
    let mut saida = Vec::new();
    let mut i = 0;

    while i + 1 < serie.len() {
        if serie[i + 1].p >= serie[i].p || serie[i + 1].t - serie[i].t > SALTO_QUE_QUEBRA_O_TRECHO_MS
        {
            i += 1;
            continue;
        }

        let inicio = i;
        let mut fim = i + 1;
        while fim + 1 < serie.len()
            && serie[fim + 1].p <= serie[fim].p
            && serie[fim + 1].t - serie[fim].t <= SALTO_QUE_QUEBRA_O_TRECHO_MS
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

    fn sessao(comeca_h_atras: i64, duracao_h: i64, de: i32, ate: i32) -> Vec<Amostra> {
        let agora = tempo::agora();
        let passos = duracao_h * 6;
        (0..=passos)
            .map(|i| Amostra {
                t: agora - comeca_h_atras * HORA + i * 10 * MINUTO,
                p: de + ((ate - de) as i64 * i / passos) as i32,
            })
            .collect()
    }

    fn historico(mut amostras: Vec<Amostra>) -> History {
        amostras.sort_by_key(|a| a.t);
        let mut por_controle = HashMap::new();
        por_controle.insert("c".to_string(), amostras);
        History { por_controle }
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
