//! Os controles que o app ja viu.
//!
//! Guardar isso permite continuar mostrando o controle certo quando ele esta desligado,
//! quando nao ha nenhum dispositivo para descobrir. O arquivo e o mesmo da versao em
//! .NET, com os mesmos nomes de campo.

use serde::{Deserialize, Serialize};

use crate::device::discovery::Controle;
use crate::{paths, tempo};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ControleSalvo {
    pub address: u64,
    pub name: String,
    pub last_seen: String,
    pub hid_id: String,
    pub instance_id: String,
    pub container_id: String,
    #[serde(rename = "XInputSlot")]
    pub xinput_slot: i32,
}

impl Default for ControleSalvo {
    fn default() -> Self {
        ControleSalvo {
            address: 0,
            name: String::new(),
            last_seen: String::new(),
            hid_id: String::new(),
            instance_id: String::new(),
            container_id: String::new(),
            xinput_slot: -1,
        }
    }
}

impl ControleSalvo {
    pub fn como_controle(&self) -> Controle {
        Controle {
            endereco: self.address,
            nome: self.name.clone(),
            id_hid: self.hid_id.clone(),
            id_instancia: self.instance_id.clone(),
            container: self.container_id.clone(),
            slot_xinput: self.xinput_slot,
        }
    }

    pub fn visto_em(&self) -> i64 {
        tempo::de_texto(&self.last_seen).unwrap_or(0)
    }
}

#[derive(Debug, Default)]
pub struct Conhecidos {
    itens: Vec<ControleSalvo>,
}

impl Conhecidos {
    pub fn carregar() -> Self {
        let itens = std::fs::read_to_string(paths::arquivo("controllers.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Vec<ControleSalvo>>(&t).ok())
            .unwrap_or_default();
        Conhecidos { itens }
    }

    pub fn itens(&self) -> &[ControleSalvo] {
        &self.itens
    }

    pub fn quantidade(&self) -> usize {
        self.itens.len()
    }

    /// Funde a descoberta com o que ja era conhecido. Devolve true se algo mudou.
    pub fn fundir(&mut self, descobertos: &[Controle]) -> bool {
        let mut mudou = false;
        let agora = tempo::para_texto(tempo::agora());

        for d in descobertos {
            let chave = d.chave();
            match self.itens.iter_mut().find(|i| i.como_controle().chave() == chave) {
                Some(existente) => {
                    if existente.name != d.nome {
                        existente.name = d.nome.clone();
                        mudou = true;
                    }
                    existente.hid_id = d.id_hid.clone();
                    existente.instance_id = d.id_instancia.clone();
                    existente.container_id = d.container.clone();
                    existente.xinput_slot = d.slot_xinput;
                    existente.last_seen = agora.clone();
                }
                None => {
                    self.itens.push(ControleSalvo {
                        address: d.endereco,
                        name: d.nome.clone(),
                        last_seen: agora.clone(),
                        hid_id: d.id_hid.clone(),
                        instance_id: d.id_instancia.clone(),
                        container_id: d.container.clone(),
                        xinput_slot: d.slot_xinput,
                    });
                    mudou = true;
                }
            }
        }

        if mudou {
            self.salvar();
        }
        mudou
    }

    pub fn salvar(&self) {
        paths::garantir_dir();
        if let Ok(t) = serde_json::to_string(&self.itens) {
            let _ = std::fs::write(paths::arquivo("controllers.json"), t);
        }
    }
}
