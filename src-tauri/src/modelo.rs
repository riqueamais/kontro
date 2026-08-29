use serde::{Deserialize, Serialize};

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
        } = bruto;

        let preenchimento = match precisao {
            Precisao::Exata => percentual,
            Precisao::Aproximada => nivel.map(preenchimento_do_nivel),
            Precisao::Nenhuma => None,
        };

        let texto_da_carga = match (precisao, percentual, nivel) {
            (Precisao::Exata, Some(p), _) => format!("{p}%"),
            (Precisao::Aproximada, _, Some(n)) => descrever_nivel(n).to_string(),
            _ => "--".to_string(),
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
