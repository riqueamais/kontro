use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precisao {
    Nenhuma,
    Aproximada,
    Exata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkMode {
    #[default]
    Offline,
    Bluetooth,
    Cable,
    Wireless,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    pub mode: LinkMode,
    pub percent: Option<i32>,
    pub precisao: Precisao,
    pub nivel: Option<i32>,
    pub read_at: Option<i64>,
    pub charging: bool,
    pub stale: bool,
    pub device_name: String,
    pub address: Option<String>,
    pub key: String,
    pub known_count: usize,

    pub preenchimento: Option<i32>,
    pub texto_da_carga: String,
    pub texto_da_ligacao: String,
    pub tem_numero: bool,
    pub conectado_sem_carga: bool,
    pub girando: bool,

    pub autonomia: Option<String>,
}

impl BatteryState {
    pub fn montar(
        mode: LinkMode,
        percent: Option<i32>,
        precisao: Precisao,
        nivel: Option<i32>,
        read_at: Option<i64>,
        charging: bool,
        stale: bool,
        device_name: String,
        address: Option<String>,
        key: String,
        known_count: usize,
        autonomia: Option<String>,
    ) -> Self {
        let preenchimento = match precisao {
            Precisao::Exata => percent,
            Precisao::Aproximada => nivel.map(preenchimento_do_nivel),
            Precisao::Nenhuma => None,
        };

        let texto_da_carga = match (precisao, percent, nivel) {
            (Precisao::Exata, Some(p), _) => format!("{p}%"),
            (Precisao::Aproximada, _, Some(n)) => descrever_nivel(n).to_string(),
            _ => "--".to_string(),
        };

        let texto_da_ligacao = match mode {
            LinkMode::Bluetooth => "Bluetooth",
            LinkMode::Wireless => "sem fio",
            LinkMode::Cable => {
                if charging {
                    "carregando"
                } else {
                    "no cabo"
                }
            }
            LinkMode::Offline => "desconectado",
        }
        .to_string();

        let tem_numero = precisao == Precisao::Exata && percent.is_some();
        let conectado_sem_carga = mode != LinkMode::Offline && preenchimento.is_none();
        let girando = mode == LinkMode::Cable && preenchimento.is_none();

        BatteryState {
            mode,
            percent,
            precisao,
            nivel,
            read_at,
            charging,
            stale,
            device_name,
            address,
            key,
            known_count,
            preenchimento,
            texto_da_carga,
            texto_da_ligacao,
            tem_numero,
            conectado_sem_carga,
            girando,
            autonomia,
        }
    }

    pub fn igual_a(&self, o: &BatteryState) -> bool {
        self.mode == o.mode
            && self.percent == o.percent
            && self.charging == o.charging
            && self.stale == o.stale
            && self.key == o.key
            && self.device_name == o.device_name
            && self.known_count == o.known_count
            && self.precisao == o.precisao
            && self.nivel == o.nivel
    }
}
