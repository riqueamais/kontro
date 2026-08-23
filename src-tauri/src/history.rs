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

    /// Quanto tempo de jogo ainda cabe, em minutos.
    pub fn autonomia_em_minutos(&self, chave: &str, percent: i32) -> Option<i64> {
        let taxa = self.consumo_por_hora(chave)?;
        let horas = percent as f64 / taxa;
        (horas > 0.0 && horas <= 200.0).then(|| (horas * 60.0).round() as i64)
    }
}
