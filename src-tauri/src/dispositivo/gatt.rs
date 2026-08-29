use std::sync::mpsc::Sender;

use windows::core::Result;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristic, GattCharacteristicUuids, GattClientCharacteristicConfigurationDescriptorValue,
    GattCommunicationStatus, GattServiceUuids, GattValueChangedEventArgs,
};
use windows::Devices::Bluetooth::{BluetoothCacheMode, BluetoothLEDevice};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;

pub enum AvisoGatt {
    Carga { endereco: u64, percentual: i32 },
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

pub fn conectado(endereco: u64) -> bool {
    (|| -> Result<bool> {
        let dev = BluetoothLEDevice::FromBluetoothAddressAsync(endereco)?
        .join()?;
        Ok(dev.ConnectionStatus()?
            == windows::Devices::Bluetooth::BluetoothConnectionStatus::Connected)
    })()
    .unwrap_or(false)
}

pub fn abrir(endereco: u64, canal: Sender<AvisoGatt>) -> Result<(VinculoGatt, Option<i32>)> {
    let dispositivo = BluetoothLEDevice::FromBluetoothAddressAsync(endereco)?
        .join()?;

    let servicos = dispositivo
        .GetGattServicesForUuidWithCacheModeAsync(GattServiceUuids::Battery()?, BluetoothCacheMode::Uncached)?
        .join()?;
    if servicos.Status()? != GattCommunicationStatus::Success {
        return Err(crate::dispositivo::sem_resposta());
    }
    let servico = servicos.Services()?.GetAt(0)?;

    let caracteristicas = servico
        .GetCharacteristicsForUuidWithCacheModeAsync(
            GattCharacteristicUuids::BatteryLevel()?,
            BluetoothCacheMode::Uncached,
        )?
        .join()?;
    if caracteristicas.Status()? != GattCommunicationStatus::Success {
        return Err(crate::dispositivo::sem_resposta());
    }
    let caracteristica = caracteristicas.Characteristics()?.GetAt(0)?;

    let inicial = ler(&caracteristica).ok();

    let manipulador = TypedEventHandler::<GattCharacteristic, GattValueChangedEventArgs>::new(
        move |_emissor, argumentos| {
            if let Some(argumentos) = argumentos.as_ref() {
                if let Ok(buffer) = argumentos.CharacteristicValue() {
                    if let Some(pct) = primeiro_byte(&buffer) {
                        let _ = canal.send(AvisoGatt::Carga { endereco, percentual: pct });
                    }
                }
            }
            Ok(())
        },
    );
    let inscricao = caracteristica.ValueChanged(&manipulador)?;

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

pub fn reler(vinculo: &VinculoGatt) -> Option<i32> {
    ler(&vinculo.caracteristica).ok()
}

fn ler(caracteristica: &GattCharacteristic) -> Result<i32> {
    let resultado = caracteristica
        .ReadValueWithCacheModeAsync(BluetoothCacheMode::Uncached)?
        .join()?;
    if resultado.Status()? != GattCommunicationStatus::Success {
        return Err(crate::dispositivo::sem_resposta());
    }
    let buffer = resultado.Value()?;
    primeiro_byte(&buffer).ok_or_else(crate::dispositivo::sem_resposta)
}

fn primeiro_byte(buffer: &windows::Storage::Streams::IBuffer) -> Option<i32> {
    let leitor = DataReader::FromBuffer(buffer).ok()?;
    let valor = leitor.ReadByte().ok()? as i32;
    (0..=100).contains(&valor).then_some(valor)
}

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
