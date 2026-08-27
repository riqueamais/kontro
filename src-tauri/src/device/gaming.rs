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
