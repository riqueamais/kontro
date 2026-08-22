//! Tudo o que fala com o Windows para descobrir controle e ler carga.
//!
//! A divisao segue as vias reais, da mais precisa para a menos: o GATT do Bluetooth da
//! percentual exato e avisa sozinho; o HID e a propriedade do PnP dao percentual quando
//! o driver informa; o XInput da quatro degraus, mas enxerga controle que os outros nao
//! veem.

pub mod gaming;
pub mod gatt;
pub mod hid;
pub mod pnp;
pub mod xinput;
pub mod discovery;

/// Inicializa a apartamento COM da thread em modo multithread.
///
/// Todo acesso a WinRT exige apartamento. A thread do monitor faz chamadas bloqueantes,
/// entao ela precisa ser MTA -- num apartamento de thread unica as esperas travariam o
/// bombeamento de mensagens.
pub fn iniciar_apartamento() {
    use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
}
