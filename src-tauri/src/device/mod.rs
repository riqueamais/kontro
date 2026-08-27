pub mod gaming;
pub mod gatt;
pub mod hid;
pub mod pnp;
pub mod xinput;
pub mod discovery;
pub mod vigia;

pub fn sem_resposta() -> windows::core::Error {
    windows::core::Error::from_hresult(windows::core::HRESULT(0x8000_4005_u32 as i32))
}

pub fn iniciar_apartamento() {
    use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
}
