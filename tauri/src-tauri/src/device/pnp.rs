//! O que o Windows sabe sobre os dispositivos, incluindo a carga que ele guarda.
//!
//! A propriedade de bateria vive no no do dispositivo, nao no ponto de acesso Bluetooth
//! -- por isso a enumeracao pede DeviceInformationKind::Device. E ela vem acompanhada de
//! uma data, que importa mais do que parece: o Windows guarda a ultima carga que o
//! controle informou por Bluetooth e continua devolvendo esse numero depois de plugado o
//! cabo. Sem olhar a data, carga de outra sessao se passa por leitura de agora.

use windows::core::{IInspectable, Interface, Result, HSTRING};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows::Foundation::{IPropertyValue, PropertyType};
use windows_collections::{IIterable, IMapView};

pub const CHAVE_NIVEL: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
pub const CHAVE_MOMENTO: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 7";
pub const CHAVE_INSTANCIA: &str = "System.Devices.DeviceInstanceId";
pub const CHAVE_CONTAINER: &str = "System.Devices.ContainerId";

/// Um no do Windows que expoe carga.
#[derive(Debug, Clone)]
pub struct NoDeBateria {
    pub nome: String,
    pub instancia: String,
    pub container: String,
    pub percent: i32,
    /// Milissegundos desde a epoca, quando o Windows anotou essa carga.
    pub medido_em: Option<i64>,
}

/// Monta a colecao de propriedades que a enumeracao aceita.
///
/// Isolado num lugar so porque e a unica peca de marshalling de colecao do WinRT em
/// todo o app -- se a forma mudar, muda aqui.
fn propriedades(chaves: &[&str]) -> Result<IIterable<HSTRING>> {
    let lista: Vec<HSTRING> = chaves.iter().map(|c| HSTRING::from(*c)).collect();
    Ok(IIterable::<HSTRING>::from(lista))
}

/// Todos os nos que publicam carga, com o que precisamos para casar com um controle.
pub fn nos_com_bateria() -> Vec<NoDeBateria> {
    (|| -> Result<Vec<NoDeBateria>> {
        let chaves = [CHAVE_NIVEL, CHAVE_MOMENTO, CHAVE_INSTANCIA, CHAVE_CONTAINER];
        let achados = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(
            &HSTRING::new(),
            &propriedades(&chaves)?,
            DeviceInformationKind::Device,
        )?
        .join()?;

        let mut saida = Vec::new();
        for info in achados {
            let props = info.Properties()?;

            let Some(bruto) = valor(&props, CHAVE_NIVEL) else { continue };
            let Some(percent) = como_inteiro(&bruto) else { continue };
            if !(0..=100).contains(&percent) {
                continue;
            }

            saida.push(NoDeBateria {
                nome: info.Name().map(|n| n.to_string()).unwrap_or_default(),
                instancia: valor(&props, CHAVE_INSTANCIA)
                    .and_then(|v| como_texto(&v))
                    .unwrap_or_default(),
                container: valor(&props, CHAVE_CONTAINER)
                    .and_then(|v| como_texto(&v))
                    .unwrap_or_default(),
                percent,
                medido_em: valor(&props, CHAVE_MOMENTO).and_then(|v| como_momento(&v)),
            });
        }
        Ok(saida)
    })()
    .unwrap_or_default()
}

/// Carga de um no cujo id de instancia contenha o trecho dado.
///
/// `desde` descarta valor anterior ao inicio da ligacao atual. Quando o driver nao
/// carimba data nenhuma, o valor passa: descartar por falta de carimbo tiraria do ar
/// fontes que funcionam.
pub fn por_instancia(instancia: &str, desde: Option<i64>) -> Option<i32> {
    if instancia.is_empty() {
        return None;
    }
    nos_com_bateria()
        .into_iter()
        .find(|no| {
            no.instancia.to_lowercase().contains(&instancia.to_lowercase()) && recente(no, desde)
        })
        .map(|no| no.percent)
}

/// Carga publicada em qualquer no do mesmo aparelho fisico.
///
/// Um controle aparece como varios nos -- a interface que o app abriu, o no do
/// dispositivo, o do adaptador -- e a carga nem sempre esta naquele cujo id o app
/// conhece. O container e o que amarra todos ao mesmo aparelho, e por isso encontra a
/// carga sem depender de reconhecer fabricante, que seria lista eternamente incompleta.
pub fn por_container(container: &str, desde: Option<i64>) -> Option<i32> {
    if container.is_empty() {
        return None;
    }
    nos_com_bateria()
        .into_iter()
        .find(|no| no.container.eq_ignore_ascii_case(container) && recente(no, desde))
        .map(|no| no.percent)
}

fn recente(no: &NoDeBateria, desde: Option<i64>) -> bool {
    match (desde, no.medido_em) {
        (Some(limite), Some(quando)) => quando >= limite,
        _ => true,
    }
}

// --------------------------------------------------------------- leitura de valores

fn valor(
    props: &IMapView<HSTRING, IInspectable>,
    chave: &str,
) -> Option<IInspectable> {
    let chave = HSTRING::from(chave);
    if !props.HasKey(&chave).unwrap_or(false) {
        return None;
    }
    props.Lookup(&chave).ok()
}

/// O tipo numerico varia conforme o driver, entao aceitamos os que fazem sentido.
fn como_inteiro(valor: &IInspectable) -> Option<i32> {
    let pv: IPropertyValue = valor.cast().ok()?;
    match pv.Type().ok()? {
        PropertyType::UInt8 => pv.GetUInt8().ok().map(|v| v as i32),
        PropertyType::Int16 => pv.GetInt16().ok().map(|v| v as i32),
        PropertyType::UInt16 => pv.GetUInt16().ok().map(|v| v as i32),
        PropertyType::Int32 => pv.GetInt32().ok(),
        PropertyType::UInt32 => pv.GetUInt32().ok().map(|v| v as i32),
        _ => None,
    }
}

fn como_texto(valor: &IInspectable) -> Option<String> {
    let pv: IPropertyValue = valor.cast().ok()?;
    match pv.Type().ok()? {
        PropertyType::String => pv.GetString().ok().map(|s| s.to_string()),
        PropertyType::Guid => pv.GetGuid().ok().map(|g| format!("{g:?}")),
        _ => None,
    }
}

/// WinRT conta em intervalos de 100ns desde 1601; o resto do app conta em
/// milissegundos desde 1970.
fn como_momento(valor: &IInspectable) -> Option<i64> {
    const EPOCA_1601_ATE_1970: i64 = 116_444_736_000_000_000;
    let pv: IPropertyValue = valor.cast().ok()?;
    if pv.Type().ok()? != PropertyType::DateTime {
        return None;
    }
    let dt = pv.GetDateTime().ok()?;
    Some((dt.UniversalTime - EPOCA_1601_ATE_1970) / 10_000)
}
