//! O vocabulario do app.
//!
//! Estes tipos atravessam a fronteira para a interface, entao o que eles dizem e
//! exatamente o que a tela pode afirmar. A regra que vale em todo o app esta aqui
//! dentro: leitura carrega junto o quanto ela vale.

use serde::{Deserialize, Serialize};

/// Quanto vale a leitura que se conseguiu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precisao {
    Nenhuma,
    /// Quatro degraus, sem numero. E o que o XInput sabe.
    Aproximada,
    /// Percentual real, de 0 a 100.
    Exata,
}

/// Como o controle esta ligado.
///
/// Sem fio nao e sinonimo de Bluetooth: dongle de radio e adaptador sem fio ligam sem
/// cabo e sem Bluetooth nenhum, e a diferenca importa porque so o Bluetooth entrega
/// percentual exato e avisa sozinho quando muda.
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

/// Texto para uma leitura aproximada, que nao tem numero para mostrar.
pub fn descrever_nivel(nivel: i32) -> &'static str {
    match nivel {
        0 => "quase acabando",
        1 => "carga baixa",
        2 => "carga média",
        _ => "carga cheia",
    }
}

/// Texto da autonomia a partir dos minutos estimados.
///
/// Hora e minuto separados porque "4 h 20 min" se le num relance e "260 min" nao.
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

/// Quanto do anel preencher quando so ha degrau, e nao percentual.
pub fn preenchimento_do_nivel(nivel: i32) -> i32 {
    match nivel {
        0 => 10,
        1 => 35,
        2 => 65,
        _ => 100,
    }
}

/// O estado que a interface desenha. Tudo o que a tela mostra sai daqui.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    pub mode: LinkMode,
    pub percent: Option<i32>,
    pub precisao: Precisao,
    pub nivel: Option<i32>,
    /// Momento da leitura, em milissegundos desde a epoca. Nulo quando nunca houve.
    pub read_at: Option<i64>,
    pub charging: bool,
    /// Verdadeiro quando o numero e a ultima leitura conhecida, nao um valor ao vivo.
    pub stale: bool,
    pub device_name: String,
    pub address: Option<String>,
    pub key: String,
    pub known_count: usize,

    // --- derivados, calculados aqui para a interface nunca reimplementar a regra ---
    /// Quanto do anel preencher. Nulo quando nao ha leitura alguma.
    pub preenchimento: Option<i32>,
    /// O que escrever sobre a carga.
    pub texto_da_carga: String,
    /// Como o controle esta ligado, em uma palavra.
    pub texto_da_ligacao: String,
    /// Ha numero para mostrar dentro do anel.
    pub tem_numero: bool,
    /// Ligado, porem sem carga alguma para informar.
    pub conectado_sem_carga: bool,
    /// No cabo e sem percentual: o anel gira em vez de ficar vazio.
    pub girando: bool,

    /// Quanto tempo de jogo ainda cabe, em texto. Nulo quando nao ha o que estimar.
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
