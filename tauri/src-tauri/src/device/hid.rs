//! Carga informada pelo proprio dispositivo HID.
//!
//! E por aqui que boa parte dos controles de dongle publica a bateria. O valor nao vem
//! em porcentagem: vem numa escala que o proprio dispositivo declara, e converter sem
//! olhar essa escala produziria numero errado com cara de certo.

use std::sync::mpsc;
use std::time::Duration;

use windows::core::{Result, HSTRING};
use windows::Devices::HumanInterfaceDevice::{HidDevice, HidReportType};
use windows::Storage::FileAccessMode;

use crate::model::Leitura;

pub const PAGINA_CONTROLES_GENERICOS: u16 = 0x0006;
pub const USO_CARGA_DA_BATERIA: u16 = 0x0020;

pub const PAGINA_DESKTOP_GENERICO: u16 = 0x01;
pub const USO_JOYSTICK: u16 = 0x04;
pub const USO_GAMEPAD: u16 = 0x05;
pub const USO_MULTI_EIXO: u16 = 0x08;

/// Le a carga pelo HID. Tenta o relatorio de recurso antes do de entrada.
///
/// O de recurso pode ser pedido a qualquer momento; o de entrada so chega quando o
/// controle resolve mandar algo. Um controle parado deixaria a leitura pendurada para
/// sempre, e por isso o segundo caminho tem prazo.
pub fn ler(id_hid: &str) -> Leitura {
    if id_hid.is_empty() {
        return Leitura::VAZIA;
    }

    let Ok(dispositivo) = abrir(id_hid) else {
        return Leitura::VAZIA;
    };

    if let Some(l) = por_recurso(&dispositivo) {
        return l;
    }
    por_entrada(&dispositivo).unwrap_or(Leitura::VAZIA)
}

fn abrir(id_hid: &str) -> Result<HidDevice> {
    // Devolver nulo aqui e normal, nao erro: acontece quando outro processo detem
    // acesso exclusivo ao controle.
    HidDevice::FromIdAsync(&HSTRING::from(id_hid), FileAccessMode::Read)?.join()
}

fn por_recurso(dispositivo: &HidDevice) -> Option<Leitura> {
    let descricoes = dispositivo
        .GetNumericControlDescriptions(
            HidReportType::Feature,
            PAGINA_CONTROLES_GENERICOS,
            USO_CARGA_DA_BATERIA,
        )
        .ok()?;

    for descricao in descricoes {
        let id = descricao.ReportId().ok()?;
        let Ok(relatorio) = dispositivo.GetFeatureReportByIdAsync(id).and_then(|op| op.join()) else {
            continue;
        };
        let Ok(controle) =
            relatorio.GetNumericControl(PAGINA_CONTROLES_GENERICOS, USO_CARGA_DA_BATERIA)
        else {
            continue;
        };
        let valor = controle.Value().ok()?;
        let minimo = descricao.LogicalMinimum().ok()?;
        let maximo = descricao.LogicalMaximum().ok()?;
        if let Some(l) = escalar(valor, minimo.into(), maximo.into()) {
            return Some(l);
        }
    }
    None
}

fn por_entrada(dispositivo: &HidDevice) -> Option<Leitura> {
    let descricoes = dispositivo
        .GetNumericControlDescriptions(
            HidReportType::Input,
            PAGINA_CONTROLES_GENERICOS,
            USO_CARGA_DA_BATERIA,
        )
        .ok()?;
    let primeira = descricoes.GetAt(0).ok()?;
    let id = primeira.ReportId().ok()?;
    let minimo = primeira.LogicalMinimum().ok()?;
    let maximo = primeira.LogicalMaximum().ok()?;

    let operacao = dispositivo.GetInputReportByIdAsync(id).ok()?;
    let (envio, recebimento) = mpsc::channel();

    // A espera roda em outra thread para poder ser abandonada: nao existe cancelamento
    // barato aqui, e travar o monitor por causa de um controle parado seria pior.
    std::thread::spawn(move || {
        crate::device::iniciar_apartamento();
        let _ = envio.send(operacao.join());
    });

    let relatorio = recebimento.recv_timeout(Duration::from_secs(2)).ok()?.ok()?;
    let controle = relatorio
        .GetNumericControl(PAGINA_CONTROLES_GENERICOS, USO_CARGA_DA_BATERIA)
        .ok()?;
    escalar(controle.Value().ok()?, minimo.into(), maximo.into())
}

/// Converte pela escala que o dispositivo declarou.
///
/// Faixa que comeca abaixo de zero nao e bateria -- e eixo, gatilho ou chapeu. Aceitar
/// isso daria 50% para um analogico centrado.
fn escalar(valor: i64, minimo: i64, maximo: i64) -> Option<Leitura> {
    if maximo <= minimo || minimo < 0 {
        return None;
    }
    let fracao = (valor - minimo) as f64 / (maximo - minimo) as f64;
    let pct = (fracao.clamp(0.0, 1.0) * 100.0).round() as i32;
    Some(Leitura::exata(pct))
}
