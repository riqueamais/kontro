//! Avisa quando um controle entra ou sai, sem ninguem ficar perguntando.
//!
//! A varredura completa e cara -- tres enumeracoes de HID, a do XUSB, e a lista de
//! pareados, que abre cada dispositivo Bluetooth so para ler o nome. Rodar isso de
//! segundo em segundo custaria caro num app que passa o dia parado; rodar de trinta em
//! trinta segundos, que era o que havia, deixava o app com uma lista de presentes velha.
//!
//! Era essa lista velha que respondia "esta no cabo" para um controle que o usuario
//! tinha acabado de desligar: o GATT ja dizia desconectado, mas o controle ainda constava
//! como presente, e presente sem Bluetooth e sem XInput so podia ser cabo.
//!
//! O Windows sabe a hora exata em que a interface aparece e some. O vigia so repassa
//! esse aviso: o monitor varre quando ha o que ver.

use std::sync::mpsc::Sender;

use windows::core::HSTRING;
use windows::Devices::Enumeration::{
    DeviceInformation, DeviceInformationUpdate, DeviceWatcher,
};
use windows::Devices::HumanInterfaceDevice::HidDevice;
use windows::Foundation::TypedEventHandler;

use super::{discovery, hid};

/// Os observadores vivos. Soltar isto para de observar.
pub struct Vigia {
    observadores: Vec<DeviceWatcher>,
}

impl Vigia {
    /// Quantos observadores estao de pe. Zero significa que a varredura depende so do
    /// relogio de seguranca -- e e a primeira coisa a conferir quando o app demora a
    /// perceber que o controle entrou ou saiu.
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

/// Comeca a observar. Cada entrada ou saida vira um aviso no canal.
///
/// O aviso nao diz o que mudou, so que mudou: quem sabe interpretar a mudanca e a
/// descoberta, e ela ja sabe fazer isso do zero. Mandar o dispositivo junto obrigaria a
/// manter dois caminhos que respondem a mesma pergunta.
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
        let _ = observador.Removed(&TypedEventHandler::<
            DeviceWatcher,
            DeviceInformationUpdate,
        >::new(move |_, _| {
            let _ = saiu.send(());
            Ok(())
        }));

        if observador.Start().is_ok() {
            observadores.push(observador);
        }
    }

    Vigia { observadores }
}

/// O que observar: as mesmas interfaces que a descoberta consulta.
///
/// Sao as duas portas por onde um controle entra. A do HID cobre quem se declara
/// gamepad, joystick ou multi-eixo; a do XUSB cobre quem so existe para o XInput, que
/// nao e dispositivo HID e nao apareceria na primeira.
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
        discovery::GUID_XUSB
    )));

    lista
}
