//! Quem esta ligado agora, seja qual for a marca ou a forma de ligacao.
//!
//! A descoberta nao reconhece modelo: pega o que se declara controle. Dispositivo HID
//! com uso de gamepad, joystick ou multi-eixo entra pela primeira via; quem so existe
//! para o XInput entra pela segunda. O nome exibido vem do proprio sistema.

use windows::core::{Result, HSTRING};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::HumanInterfaceDevice::HidDevice;
use windows_collections::IIterable;

use super::{hid, pnp, xinput};

/// Interface que o Windows publica para todo controle atendido pelo XUSB.
pub(crate) const GUID_XUSB: &str = "{EC87F1E3-C13B-4100-B5F7-8B84D54260CB}";

#[derive(Debug, Clone, Default)]
pub struct Controle {
    /// Endereco Bluetooth. Zero quando o controle so aparece por cabo ou dongle.
    pub endereco: u64,
    pub nome: String,
    pub id_hid: String,
    pub id_instancia: String,
    pub container: String,
    /// Slot do XInput, quando o controle so existe por ali. -1 para os demais.
    pub slot_xinput: i32,
}

impl Controle {
    /// Identidade estavel.
    ///
    /// Com Bluetooth e o endereco; sem ele, o proprio id da interface, que ja carrega
    /// fabricante, produto e a parte especifica daquela conexao.
    pub fn chave(&self) -> String {
        if self.endereco != 0 {
            return format!("{:012x}", self.endereco);
        }
        if self.slot_xinput >= 0 {
            return format!("xinput:{}", self.slot_xinput);
        }
        format!("hid:{}", chave_do_hid(&self.id_hid))
    }

    pub fn endereco_bonito(&self) -> Option<String> {
        if self.endereco == 0 {
            return None;
        }
        let b = self.endereco;
        Some(
            (0..6)
                .map(|i| format!("{:02X}", (b >> ((5 - i) * 8)) & 0xFF))
                .collect::<Vec<_>>()
                .join(":"),
        )
    }
}

fn chave_do_hid(id: &str) -> String {
    if id.is_empty() {
        return "desconhecido".to_string();
    }
    let limpo: String = id
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if limpo.len() <= 40 {
        limpo
    } else {
        limpo[limpo.len() - 40..].to_string()
    }
}

/// Um bloco de doze hexadecimais delimitado por & ou _ dentro do id da interface.
pub fn extrair_endereco(id: &str) -> u64 {
    let bytes: Vec<char> = id.chars().collect();
    let delimitador = |c: char| c == '&' || c == '_';

    for inicio in 0..bytes.len() {
        if !delimitador(bytes[inicio]) {
            continue;
        }
        let fim = inicio + 13;
        if fim >= bytes.len() || !delimitador(bytes[fim]) {
            continue;
        }
        let trecho: String = bytes[inicio + 1..fim].iter().collect();
        if !trecho.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        if let Ok(v) = u64::from_str_radix(&trecho, 16) {
            if v != 0 {
                return v;
            }
        }
    }
    0
}

pub fn descobrir() -> Vec<Controle> {
    descobrir_com(&super::gatt::pareados())
}

/// A varredura, com a lista de pareados vinda de fora.
///
/// `pareados` so serve para dar nome: ela custa uma abertura de dispositivo Bluetooth
/// por aparelho e o nome de um controle nao muda entre uma varredura e a seguinte. Quem
/// chama guarda a lista e a renova quando ela envelhece -- e essa separacao e o que
/// permite varrer de segundos em segundos sem acordar o radio a toa.
pub fn descobrir_com(pareados: &[(u64, String)]) -> Vec<Controle> {
    // Sem HID ainda pode haver controle: o XUSB e um mundo a parte, conferido mais
    // abaixo. Desistir aqui era justamente o que escondia controle de dongle.
    let achados = gamepads_hid();

    let mut resultado: Vec<Controle> = Vec::new();
    let mut containers_hid: Vec<String> = Vec::new();

    for g in achados {
        let nome = pareados
            .iter()
            .find(|(endereco, _)| *endereco == g.endereco && g.endereco != 0)
            .map(|(_, nome)| nome.clone())
            .unwrap_or_else(|| {
                if g.nome.trim().is_empty() {
                    "Controle".to_string()
                } else {
                    g.nome.clone()
                }
            });

        containers_hid.push(g.container.clone());
        resultado.push(Controle {
            endereco: g.endereco,
            nome,
            id_hid: g.id_hid,
            id_instancia: g.id_instancia,
            container: g.container,
            slot_xinput: -1,
        });
    }

    acrescentar_somente_xinput(&mut resultado, &containers_hid);
    resultado
}

struct GamepadHid {
    id_hid: String,
    id_instancia: String,
    container: String,
    nome: String,
    endereco: u64,
}

fn gamepads_hid() -> Vec<GamepadHid> {
    let usos = [hid::USO_GAMEPAD, hid::USO_JOYSTICK, hid::USO_MULTI_EIXO];
    let mut lista = Vec::new();
    let mut vistos: Vec<String> = Vec::new();

    for uso in usos {
        let Ok(encontrados) = por_uso(uso) else { continue };
        for g in encontrados {
            if vistos.iter().any(|v| v.eq_ignore_ascii_case(&g.id_hid)) {
                continue;
            }
            vistos.push(g.id_hid.clone());
            lista.push(g);
        }
    }
    lista
}

fn por_uso(uso: u16) -> Result<Vec<GamepadHid>> {
    let seletor = HidDevice::GetDeviceSelector(hid::PAGINA_DESKTOP_GENERICO, uso)?;
    let chaves = [pnp::CHAVE_INSTANCIA, pnp::CHAVE_CONTAINER];
    let props: Vec<HSTRING> = chaves.iter().map(|c| HSTRING::from(*c)).collect();
    let achados = DeviceInformation::FindAllAsyncAqsFilterAndAdditionalProperties(
        &seletor,
        &IIterable::<HSTRING>::from(props),
    )?
    .join()?;

    let mut saida = Vec::new();
    for info in achados {
        let id = info.Id()?.to_string();
        let propriedades = info.Properties()?;
        let texto = |chave: &str| -> String {
            let chave = HSTRING::from(chave);
            propriedades
                .HasKey(&chave)
                .ok()
                .and_then(|tem| tem.then(|| propriedades.Lookup(&chave).ok()).flatten())
                .and_then(|v| {
                    use windows::core::Interface;
                    let pv: windows::Foundation::IPropertyValue = v.cast().ok()?;
                    match pv.Type().ok()? {
                        windows::Foundation::PropertyType::String => {
                            pv.GetString().ok().map(|s| s.to_string())
                        }
                        windows::Foundation::PropertyType::Guid => {
                            pv.GetGuid().ok().map(|g| format!("{g:?}"))
                        }
                        _ => None,
                    }
                })
                .unwrap_or_default()
        };

        saida.push(GamepadHid {
            endereco: extrair_endereco(&id),
            id_instancia: texto(pnp::CHAVE_INSTANCIA),
            container: texto(pnp::CHAVE_CONTAINER),
            nome: info.Name()?.to_string(),
            id_hid: id,
        });
    }
    Ok(saida)
}

/// Acrescenta controles que existem apenas para o XInput.
///
/// Quem usa o driver do Xbox 360 -- o caso dos dongles que emulam esse controle -- nao
/// e dispositivo HID. A busca acima passa direto por ele, e sem isto esse controle
/// simplesmente nao existe para o app.
///
/// A comparacao e por container, que representa o aparelho fisico: um controle atendido
/// por HID e por XUSB publica as duas interfaces sob o mesmo container. Container de
/// XUSB que nao apareceu no HID e, por definicao, controle que so o XInput enxerga --
/// criterio que nao depende de contar dispositivos nem de adivinhar quem e quem.
fn acrescentar_somente_xinput(encontrados: &mut Vec<Controle>, containers_hid: &[String]) {
    let slots = xinput::slots_conectados();
    if slots.is_empty() {
        return;
    }

    let xusb = dispositivos_xusb();

    let somente: Vec<(String, String, String)> = if xusb.is_empty() {
        // Sem lista de XUSB nao ha container para comparar. Ainda assim, se o XInput ve
        // controle e o HID nao viu nenhum, nao existe com o que confundir.
        if !encontrados.is_empty() {
            return;
        }
        slots.iter().map(|_| (String::new(), String::new(), String::new())).collect()
    } else {
        xusb.into_iter()
            .filter(|(container, _, _)| {
                container.is_empty()
                    || !containers_hid.iter().any(|c| c.eq_ignore_ascii_case(container))
            })
            .collect()
    };

    if somente.is_empty() || somente.len() > slots.len() {
        return;
    }

    // na pratica os slots ocupados por controle HID vem primeiro na contagem do XInput,
    // entao o que sobra na ponta pertence a estes
    let meus = &slots[slots.len() - somente.len()..];
    let mut reserva = nomes_disponiveis(encontrados);

    for (i, (container, nome_xusb, instancia)) in somente.into_iter().enumerate() {
        let slot = meus[i];
        encontrados.push(Controle {
            endereco: 0,
            nome: escolher_nome(&nome_xusb, &mut reserva, slot),
            id_hid: String::new(),
            id_instancia: instancia,
            container,
            slot_xinput: slot as i32,
        });
    }
}

/// Devolve (container, nome, instancia) de cada interface do XUSB ativa.
fn dispositivos_xusb() -> Vec<(String, String, String)> {
    (|| -> Result<Vec<(String, String, String)>> {
        let seletor = HSTRING::from(format!(
            "System.Devices.InterfaceClassGuid:=\"{GUID_XUSB}\" AND \
             System.Devices.InterfaceEnabled:=System.StructuredQueryType.Boolean#True"
        ));
        let chaves = [pnp::CHAVE_CONTAINER, pnp::CHAVE_INSTANCIA];
        let props: Vec<HSTRING> = chaves.iter().map(|c| HSTRING::from(*c)).collect();
        let achados = DeviceInformation::FindAllAsyncAqsFilterAndAdditionalProperties(
            &seletor,
            &IIterable::<HSTRING>::from(props),
        )?
        .join()?;

        let mut saida = Vec::new();
        for info in achados {
            let propriedades = info.Properties()?;
            let texto = |chave: &str| -> String {
                use windows::core::Interface;
                let chave = HSTRING::from(chave);
                propriedades
                    .HasKey(&chave)
                    .ok()
                    .and_then(|tem| tem.then(|| propriedades.Lookup(&chave).ok()).flatten())
                    .and_then(|v| {
                        let pv: windows::Foundation::IPropertyValue = v.cast().ok()?;
                        match pv.Type().ok()? {
                            windows::Foundation::PropertyType::String => {
                                pv.GetString().ok().map(|s| s.to_string())
                            }
                            windows::Foundation::PropertyType::Guid => {
                                pv.GetGuid().ok().map(|g| format!("{g:?}"))
                            }
                            _ => None,
                        }
                    })
                    .unwrap_or_default()
            };
            saida.push((
                texto(pnp::CHAVE_CONTAINER),
                info.Name()?.to_string(),
                texto(pnp::CHAVE_INSTANCIA),
            ));
        }
        Ok(saida)
    })()
    .unwrap_or_default()
}

/// O XInput nao informa nome nenhum, e o nome da interface do XUSB costuma ser generico.
fn nomes_disponiveis(ja_encontrados: &[Controle]) -> Vec<String> {
    let usados: Vec<String> = ja_encontrados.iter().map(|c| c.nome.to_lowercase()).collect();
    pnp::nos_com_bateria()
        .into_iter()
        .map(|no| no.nome)
        .filter(|nome| !nome.trim().is_empty() && !usados.contains(&nome.to_lowercase()))
        .collect()
}

fn escolher_nome(do_xusb: &str, reserva: &mut Vec<String>, slot: u32) -> String {
    if !reserva.is_empty() {
        return reserva.remove(0);
    }
    if !do_xusb.trim().is_empty() {
        return do_xusb.to_string();
    }
    format!("Controle {}", slot + 1)
}
