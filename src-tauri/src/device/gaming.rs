//! Gaming.Input, usado para uma coisa so: saber se o controle esta carregando.
//!
//! O XInput distingue cabo de bateria, mas nao diz se a bateria esta subindo. Quem sabe
//! isso e o relatorio de energia do Gaming.Input. Se a consulta falhar, o app segue
//! dizendo apenas "no cabo" -- e menos informacao, nunca informacao errada.

use windows::Gaming::Input::RawGameController;
use windows::System::Power::BatteryStatus;

pub fn carregando() -> bool {
    (|| -> Option<bool> {
        let controles = RawGameController::RawGameControllers().ok()?;
        for controle in controles {
            use windows::core::Interface;
            let Ok(info) = controle.cast::<windows::Gaming::Input::IGameControllerBatteryInfo>()
            else {
                continue;
            };
            let Ok(relatorio) = info.TryGetBatteryReport() else { continue };
            if relatorio.Status().ok()? == BatteryStatus::Charging {
                return Some(true);
            }
            if let Ok(taxa) = relatorio.ChargeRateInMilliwatts() {
                if taxa.Value().unwrap_or(0) > 0 {
                    return Some(true);
                }
            }
        }
        Some(false)
    })()
    .unwrap_or(false)
}

/// Quantos controles o sistema enxerga, seja qual for a via.
pub fn quantidade() -> usize {
    RawGameController::RawGameControllers()
        .and_then(|c| c.Size())
        .map(|n| n as usize)
        .unwrap_or(0)
}
