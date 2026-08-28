use windows::core::{IInspectable, Interface, Result, HSTRING};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows::Foundation::{IPropertyValue, PropertyType};
use windows_collections::{IIterable, IMapView};

pub const CHAVE_NIVEL: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
pub const CHAVE_MOMENTO: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 7";
pub const CHAVE_INSTANCIA: &str = "System.Devices.DeviceInstanceId";
pub const CHAVE_CONTAINER: &str = "System.Devices.ContainerId";

#[derive(Debug, Clone)]
pub struct NoDeBateria {
    pub nome: String,
    pub instancia: String,
    pub container: String,
    pub percent: i32,
    pub medido_em: Option<i64>,
}

fn propriedades(chaves: &[&str]) -> Result<IIterable<HSTRING>> {
    let lista: Vec<HSTRING> = chaves.iter().map(|c| HSTRING::from(*c)).collect();
    Ok(IIterable::<HSTRING>::from(lista))
}

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

pub struct CargaGuardada {
    pub percent: i32,
    pub medido_em: Option<i64>,
}

impl From<&NoDeBateria> for CargaGuardada {
    fn from(no: &NoDeBateria) -> Self {
        CargaGuardada { percent: no.percent, medido_em: no.medido_em }
    }
}

pub fn por_instancia(
    nos: &[NoDeBateria],
    instancia: &str,
    desde: Option<i64>,
) -> Option<CargaGuardada> {
    if instancia.is_empty() {
        return None;
    }
    let alvo = instancia.to_lowercase();
    nos.iter()
        .find(|no| no.instancia.to_lowercase().contains(&alvo) && recente(no, desde))
        .map(CargaGuardada::from)
}

pub fn por_container(
    nos: &[NoDeBateria],
    container: &str,
    desde: Option<i64>,
) -> Option<CargaGuardada> {
    if container.is_empty() {
        return None;
    }
    nos.iter()
        .find(|no| no.container.eq_ignore_ascii_case(container) && recente(no, desde))
        .map(CargaGuardada::from)
}

fn recente(no: &NoDeBateria, desde: Option<i64>) -> bool {
    match (desde, no.medido_em) {
        (Some(limite), Some(quando)) => quando >= limite,
        _ => true,
    }
}

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

fn como_momento(valor: &IInspectable) -> Option<i64> {
    const EPOCA_1601_ATE_1970: i64 = 116_444_736_000_000_000;
    let pv: IPropertyValue = valor.cast().ok()?;
    if pv.Type().ok()? != PropertyType::DateTime {
        return None;
    }
    let dt = pv.GetDateTime().ok()?;
    Some((dt.UniversalTime - EPOCA_1601_ATE_1970) / 10_000)
}
