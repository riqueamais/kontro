use serde::{Deserialize, Serialize};

use crate::configuracoes::Limiares;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precisao {
    Nenhuma,
    Aproximada,
    Exata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Via {
    #[default]
    Desligado,
    Bluetooth,
    Cabo,
    SemFio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leitura {
    pub valor: i32,
    pub precisao: Precisao,
}

impl Leitura {
    pub const VAZIA: Leitura = Leitura { valor: 0, precisao: Precisao::Nenhuma };

    pub fn exata(pct: i32) -> Self {
        Leitura { valor: pct.clamp(0, 100), precisao: Precisao::Exata }
    }

    pub fn degrau(nivel: i32) -> Self {
        Leitura { valor: nivel.clamp(0, 3), precisao: Precisao::Aproximada }
    }

    pub fn tem(&self) -> bool {
        self.precisao != Precisao::Nenhuma
    }
}

pub fn descrever_nivel(nivel: i32) -> &'static str {
    match nivel {
        0 => "quase acabando",
        1 => "carga baixa",
        2 => "carga média",
        _ => "carga cheia",
    }
}

pub fn descrever_autonomia(minutos: i64) -> String {
    if minutos >= 60 {
        let h = minutos / 60;
        let m = minutos % 60;
        if m > 0 {
            format!("~{h} h {m} min de jogo")
        } else {
            format!("~{h} h de jogo")
        }
    } else {
        format!("~{} min de jogo", minutos.max(1))
    }
}

fn autonomia_curta(minutos: i64) -> String {
    match (minutos / 60, minutos % 60) {
        (0, m) => format!("~{} min", m.max(1)),
        (h, 0) => format!("~{h} h"),
        (h, m) => format!("~{h} h {m} min"),
    }
}

pub fn resumo_do_estado(estado: &EstadoDoControle, limiares: Limiares) -> String {
    let carga = match (estado.tem_numero, estado.percentual) {
        (true, Some(p)) => Some(format!("{p}%, {}", estado.faixa(limiares))),
        _ if estado.precisao == Precisao::Aproximada => Some(estado.texto_da_carga.clone()),
        _ => None,
    };

    if estado.via == Via::Desligado {
        return match (carga, estado.percentual, estado.lido_em) {
            (Some(_), Some(p), Some(em)) => {
                format!("Desligado · {p}% {}", crate::tempo::quando(em))
            }
            _ => "Desligado".to_string(),
        };
    }

    let mut partes = Vec::new();
    match carga {
        Some(carga) => partes.push(carga),
        None if estado.via == Via::Cabo && estado.carregando => {
            partes.push("No cabo, carregando".to_string())
        }
        None if estado.via == Via::Cabo => partes.push("No cabo".to_string()),
        None => partes.push("sem leitura".to_string()),
    }
    if estado.preenchimento.is_some() || estado.via != Via::Cabo {
        partes.push(estado.texto_da_ligacao.clone());
    }
    if let Some(minutos) = estado.autonomia_minutos {
        partes.push(autonomia_curta(minutos));
    }

    let resumo = partes.join(" · ");
    if estado.leitura_antiga {
        format!("leitura antiga · {resumo}")
    } else {
        resumo
    }
}

pub fn preenchimento_do_nivel(nivel: i32) -> i32 {
    match nivel {
        0 => 10,
        1 => 35,
        2 => 65,
        _ => 100,
    }
}

#[derive(Debug, Clone, Default)]
pub struct Bruto {
    pub via: Via,
    pub percentual: Option<i32>,
    pub precisao: Precisao,
    pub nivel: Option<i32>,
    pub lido_em: Option<i64>,
    pub carregando: bool,
    pub leitura_antiga: bool,
    pub nome: String,
    pub endereco: Option<String>,
    pub chave: String,
    pub quantos_conhecidos: usize,
    pub autonomia: Option<String>,
    pub autonomia_minutos: Option<i64>,
}

impl Default for Precisao {
    fn default() -> Self {
        Precisao::Nenhuma
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDoControle {
    pub via: Via,
    pub percentual: Option<i32>,
    pub precisao: Precisao,
    pub nivel: Option<i32>,
    pub lido_em: Option<i64>,
    pub carregando: bool,
    pub leitura_antiga: bool,
    pub nome: String,
    pub endereco: Option<String>,
    pub chave: String,
    pub quantos_conhecidos: usize,

    pub preenchimento: Option<i32>,
    pub texto_da_carga: String,
    pub texto_da_ligacao: String,
    pub tem_numero: bool,
    pub conectado_sem_carga: bool,
    pub girando: bool,

    pub autonomia: Option<String>,
    pub autonomia_minutos: Option<i64>,
}

impl EstadoDoControle {
    pub fn montar(bruto: Bruto) -> Self {
        let Bruto {
            via,
            percentual,
            precisao,
            nivel,
            lido_em,
            carregando,
            leitura_antiga,
            nome,
            endereco,
            chave,
            quantos_conhecidos,
            autonomia,
            autonomia_minutos,
        } = bruto;

        let preenchimento = match precisao {
            Precisao::Exata => percentual,
            Precisao::Aproximada => nivel.map(preenchimento_do_nivel),
            Precisao::Nenhuma => None,
        };

        let texto_da_carga = match (precisao, percentual, nivel) {
            (Precisao::Exata, Some(p), _) => format!("{p}%"),
            (Precisao::Aproximada, _, Some(n)) => descrever_nivel(n).to_string(),
            _ => String::new(),
        };

        let texto_da_ligacao = match via {
            Via::Bluetooth => "Bluetooth",
            Via::SemFio => "sem fio",
            Via::Cabo => {
                if carregando {
                    "carregando"
                } else {
                    "no cabo"
                }
            }
            Via::Desligado => "desconectado",
        }
        .to_string();

        let tem_numero = precisao == Precisao::Exata && percentual.is_some();
        let conectado_sem_carga = via != Via::Desligado && preenchimento.is_none();
        let girando = via == Via::Cabo && preenchimento.is_none();

        EstadoDoControle {
            via,
            percentual,
            precisao,
            nivel,
            lido_em,
            carregando,
            leitura_antiga,
            nome,
            endereco,
            chave,
            quantos_conhecidos,
            preenchimento,
            texto_da_carga,
            texto_da_ligacao,
            tem_numero,
            conectado_sem_carga,
            girando,
            autonomia,
            autonomia_minutos,
        }
    }

    pub fn faixa(&self, limiares: Limiares) -> &'static str {
        match self.preenchimento {
            Some(p) if p < limiares.critico => "carga crítica",
            Some(p) if p < limiares.aviso => "carga baixa",
            _ => "com folga",
        }
    }

    pub fn igual_a(&self, o: &EstadoDoControle) -> bool {
        self.via == o.via
            && self.percentual == o.percentual
            && self.carregando == o.carregando
            && self.leitura_antiga == o.leitura_antiga
            && self.chave == o.chave
            && self.nome == o.nome
            && self.quantos_conhecidos == o.quantos_conhecidos
            && self.precisao == o.precisao
            && self.nivel == o.nivel
            && self.autonomia == o.autonomia
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn estado(via: Via, percentual: Option<i32>) -> EstadoDoControle {
        EstadoDoControle::montar(Bruto {
            via,
            percentual,
            precisao: if percentual.is_some() { Precisao::Exata } else { Precisao::Nenhuma },
            nome: "Xbox Wireless Controller".into(),
            chave: "x".into(),
            ..Default::default()
        })
    }

    #[test]
    fn o_resumo_diz_a_faixa_com_os_limiares_de_quem_usa() {
        let mut baixo = estado(Via::Bluetooth, Some(18));
        baixo.autonomia_minutos = Some(130);
        assert_eq!(
            resumo_do_estado(&baixo, Limiares::PADRAO),
            "18%, carga baixa · Bluetooth · ~2 h 10 min"
        );
        let folga = estado(Via::Bluetooth, Some(77));
        assert_eq!(resumo_do_estado(&folga, Limiares::PADRAO), "77%, com folga · Bluetooth");
    }

    #[test]
    fn no_cabo_sem_numero_e_desligado_sem_leitura_nao_inventam_carga() {
        let mut cabo = estado(Via::Cabo, None);
        assert_eq!(resumo_do_estado(&cabo, Limiares::PADRAO), "No cabo");
        cabo.carregando = true;
        assert_eq!(resumo_do_estado(&cabo, Limiares::PADRAO), "No cabo, carregando");
        assert_eq!(resumo_do_estado(&estado(Via::Desligado, None), Limiares::PADRAO), "Desligado");
    }

    #[test]
    fn desligado_diz_quando_foi_a_ultima_leitura() {
        let mut parado = estado(Via::Desligado, Some(77));
        parado.lido_em = Some(crate::tempo::agora());
        assert!(resumo_do_estado(&parado, Limiares::PADRAO).starts_with("Desligado · 77% hoje às "));
    }
}
