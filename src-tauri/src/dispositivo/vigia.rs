use std::sync::mpsc::Sender;

use windows::core::HSTRING;
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher};
use windows::Devices::HumanInterfaceDevice::HidDevice;
use windows::Foundation::TypedEventHandler;

use super::{descoberta, hid};

pub struct Vigia {
    observadores: Vec<DeviceWatcher>,
}

impl Vigia {
    pub fn quantos(&self) -> usize {
        self.observadores.len()
    }
}

impl Drop for Vigia {
    fn drop(&mut self) {
        for observador in &self.observadores {
            let _ = observador.Stop();
        }
    }
}

pub fn observar(canal: Sender<()>) -> Vigia {
    let mut observadores = Vec::new();

    for seletor in seletores() {
        let Ok(observador) = DeviceInformation::CreateWatcherAqsFilter(&seletor) else {
            continue;
        };

        let entrou = canal.clone();
        let _ = observador.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(
            move |_, _| {
                let _ = entrou.send(());
                Ok(())
            },
        ));

        let saiu = canal.clone();
        let _ = observador.Removed(
            &TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new(move |_, _| {
                let _ = saiu.send(());
                Ok(())
            }),
        );

        if observador.Start().is_ok() {
            observadores.push(observador);
        }
    }

    Vigia { observadores }
}

fn seletores() -> Vec<HSTRING> {
    let mut lista = Vec::new();

    for uso in [hid::USO_GAMEPAD, hid::USO_JOYSTICK, hid::USO_MULTI_EIXO] {
        if let Ok(seletor) = HidDevice::GetDeviceSelector(hid::PAGINA_DESKTOP_GENERICO, uso) {
            lista.push(seletor);
        }
    }

    lista.push(HSTRING::from(format!(
        "System.Devices.InterfaceClassGuid:=\"{}\" AND \
         System.Devices.InterfaceEnabled:=System.StructuredQueryType.Boolean#True",
        descoberta::GUID_XUSB
    )));

    lista
}
