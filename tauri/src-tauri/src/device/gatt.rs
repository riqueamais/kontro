//! A fonte da verdade: o Battery Service do Bluetooth LE.
//!
//! E a unica via que da percentual exato, e a unica que avisa sozinha quando a carga
//! muda -- o controle empurra o valor pelo Notify em vez de esperar ser perguntado.
//! Perguntar de tempos em tempos acordaria o radio a toa e ainda daria numero atrasado.
//!
//! No cabo este servico desaparece: o controle troca de protocolo e para de publicar
//! carga. Nao ha o que consertar ali -- ha o que dizer ao usuario, e quem diz e a
//! interface.

use std::sync::mpsc::Sender;

use windows::core::Result;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristic, GattCharacteristicUuids, GattClientCharacteristicConfigurationDescriptorValue,
    GattCommunicationStatus, GattServiceUuids, GattValueChangedEventArgs,
};
use windows::Devices::Bluetooth::{BluetoothCacheMode, BluetoothLEDevice};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;

/// O que o vinculo GATT manda para o monitor.
pub enum AvisoGatt {
    /// Carga empurrada pelo controle. E sempre uma medida do vinculo atual.
    Carga { endereco: u64, percent: i32 },
}

pub struct VinculoGatt {
    endereco: u64,
    _dispositivo: BluetoothLEDevice,
    caracteristica: GattCharacteristic,
    inscricao: i64,
}

impl VinculoGatt {
    pub fn endereco(&self) -> u64 {
        self.endereco
    }
}

impl Drop for VinculoGatt {
    fn drop(&mut self) {
        let _ = self.caracteristica.RemoveValueChanged(self.inscricao);
    }
}

/// O controle esta conectado agora?
pub fn conectado(endereco: u64) -> bool {
    (|| -> Result<bool> {
        let dev = BluetoothLEDevice::FromBluetoothAddressAsync(endereco)?
        .join()?;
        Ok(dev.ConnectionStatus()?
            == windows::Devices::Bluetooth::BluetoothConnectionStatus::Connected)
    })()
    .unwrap_or(false)
}

/// Abre o vinculo, faz a leitura inicial e assina o Notify.
///
/// A leitura inicial volta separada de proposito: ela nao merece a mesma confianca que
/// as seguintes. Ha controle que responde a primeira consulta com um valor de espera --
/// 50%, num caso medido -- e so manda a medida real segundos depois, pelo Notify. Quem
/// decide o que fazer com isso e o monitor.
pub fn abrir(endereco: u64, canal: Sender<AvisoGatt>) -> Result<(VinculoGatt, Option<i32>)> {
    let dispositivo = BluetoothLEDevice::FromBluetoothAddressAsync(endereco)?
        .join()?;

    let servicos = dispositivo
        .GetGattServicesForUuidWithCacheModeAsync(GattServiceUuids::Battery()?, BluetoothCacheMode::Uncached)?
        .join()?;
    if servicos.Status()? != GattCommunicationStatus::Success {
        return Err(crate::device::sem_resposta());
    }
    let servico = servicos.Services()?.GetAt(0)?;

    let caracteristicas = servico
        .GetCharacteristicsForUuidWithCacheModeAsync(
            GattCharacteristicUuids::BatteryLevel()?,
            BluetoothCacheMode::Uncached,
        )?
        .join()?;
    if caracteristicas.Status()? != GattCommunicationStatus::Success {
        return Err(crate::device::sem_resposta());
    }
    let caracteristica = caracteristicas.Characteristics()?.GetAt(0)?;

    let inicial = ler(&caracteristica).ok();

    let manipulador = TypedEventHandler::<GattCharacteristic, GattValueChangedEventArgs>::new(
        move |_emissor, argumentos| {
            if let Some(argumentos) = argumentos.as_ref() {
                if let Ok(buffer) = argumentos.CharacteristicValue() {
                    if let Some(pct) = primeiro_byte(&buffer) {
                        let _ = canal.send(AvisoGatt::Carga { endereco, percent: pct });
                    }
                }
            }
            Ok(())
        },
    );
    let inscricao = caracteristica.ValueChanged(&manipulador)?;

    // sem isto o controle nunca envia nada por conta propria
    let _ = caracteristica
        .WriteClientCharacteristicConfigurationDescriptorAsync(
            GattClientCharacteristicConfigurationDescriptorValue::Notify,
        )?
        .join();

    Ok((
        VinculoGatt { endereco, _dispositivo: dispositivo, caracteristica, inscricao },
        inicial,
    ))
}

fn ler(caracteristica: &GattCharacteristic) -> Result<i32> {
    let resultado = caracteristica
        .ReadValueWithCacheModeAsync(BluetoothCacheMode::Uncached)?
        .join()?;
    if resultado.Status()? != GattCommunicationStatus::Success {
        return Err(crate::device::sem_resposta());
    }
    let buffer = resultado.Value()?;
    primeiro_byte(&buffer).ok_or_else(crate::device::sem_resposta)
}

/// A carga do Battery Service e um unico byte de 0 a 100.
fn primeiro_byte(buffer: &windows::Storage::Streams::IBuffer) -> Option<i32> {
    let leitor = DataReader::FromBuffer(buffer).ok()?;
    let valor = leitor.ReadByte().ok()? as i32;
    (0..=100).contains(&valor).then_some(valor)
}

/// Enderecos de todo dispositivo Bluetooth LE pareado, com o nome que o sistema da.
///
/// O nome vem daqui e nao do HID porque o HID devolve rotulo generico -- "controlador de
/// jogo compativel com HID" nao diz ao usuario qual controle e o dele.
pub fn pareados() -> Vec<(u64, String)> {
    use windows::Devices::Enumeration::DeviceInformation;

    (|| -> Result<Vec<(u64, String)>> {
        let seletor = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true)?;
        let achados = DeviceInformation::FindAllAsyncAqsFilter(&seletor)?
        .join()?;

        let mut saida = Vec::new();
        for info in achados {
            let id = info.Id()?;
            if let Ok(dev) = BluetoothLEDevice::FromIdAsync(&id)?
        .join() {
                let nome = dev.Name().map(|n| n.to_string()).unwrap_or_default();
                if let Ok(endereco) = dev.BluetoothAddress() {
                    if !nome.trim().is_empty() {
                        saida.push((endereco, nome));
                    }
                }
            }
        }
        Ok(saida)
    })()
    .unwrap_or_default()
}
