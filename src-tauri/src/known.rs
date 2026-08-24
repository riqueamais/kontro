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
    /// Nome dado pelo usuario. Quando existe, manda sobre o do sistema.
    pub apelido: String,
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
            apelido: String::new(),
        }
    }
}

impl ControleSalvo {
    pub fn como_controle(&self) -> Controle {
        Controle {
            endereco: self.address,
            nome: if self.apelido.trim().is_empty() {
                self.name.clone()
            } else {
                self.apelido.clone()
            },
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

/// O nome novo vale a pena?
///
/// Nome de Bluetooth e o bom: o HID devolve rotulo generico, igual para qualquer
/// controle. Entao um nome vindo do Bluetooth sempre substitui, e um nome sem Bluetooth
/// nunca derruba um que veio de la.
fn melhor_nome(atual: &str, novo: &str, endereco_atual: u64, endereco_novo: u64) -> bool {
    if novo.trim().is_empty() || atual == novo {
        return false;
    }
    if atual.trim().is_empty() {
        return true;
    }
    if endereco_novo != 0 {
        return true;
    }
    endereco_atual == 0
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

        let mut conhecidos = Conhecidos { itens };
        if conhecidos.limpar() {
            conhecidos.salvar();
        }
        conhecidos
    }

    /// Junta copias do mesmo controle que ja tenham sido gravadas.
    ///
    /// A correcao na fusao evita criar novas, mas quem ja tem o arquivo sujo continuaria
    /// sujo para sempre -- e sao registros que nunca aparecem na tela, entao ninguem
    /// tem como limpar por fora.
    fn limpar(&mut self) -> bool {
        let antes = self.itens.len();

        // versoes antigas chegaram a gravar o nome do sistema como apelido
        let mut mudou = false;
        for item in &mut self.itens {
            if item.apelido == item.name {
                item.apelido.clear();
                mudou = true;
            }
        }

        // o mais recente de cada container fica; sem container, cada um e unico
        self.itens.sort_by(|a, b| b.visto_em().cmp(&a.visto_em()));
        let mut vistos: Vec<String> = Vec::new();
        self.itens.retain(|i| {
            if i.container_id.is_empty() {
                return true;
            }
            let chave = i.container_id.to_lowercase();
            if vistos.contains(&chave) {
                return false;
            }
            vistos.push(chave);
            true
        });

        mudou || self.itens.len() != antes
    }

    pub fn itens(&self) -> &[ControleSalvo] {
        &self.itens
    }

    pub fn quantidade(&self) -> usize {
        self.itens.len()
    }

    /// Funde a descoberta com o que ja era conhecido. Devolve true se algo mudou.
    ///
    /// O casamento e por chave ou por container. So a chave nao basta: o mesmo controle
    /// fisico ganha ids de interface diferentes conforme a via por onde aparece, e a
    /// lista ia acumulando copias dele -- todas com o nome generico que o Windows da, e
    /// nenhuma visivel para o usuario apagar.
    pub fn fundir(&mut self, descobertos: &[Controle]) -> bool {
        let mut mudou = false;
        let agora = tempo::para_texto(tempo::agora());

        for d in descobertos {
            let chave = d.chave();
            let achado = self.itens.iter_mut().find(|i| {
                i.como_controle().chave() == chave
                    || (!d.container.is_empty() && i.container_id.eq_ignore_ascii_case(&d.container))
            });

            match achado {
                Some(existente) => {
                    if melhor_nome(&existente.name, &d.nome, existente.address, d.endereco) {
                        existente.name = d.nome.clone();
                        mudou = true;
                    }
                    // o endereco Bluetooth so entra, nunca sai: perde-lo faria o controle
                    // trocar de chave e virar um registro novo
                    if d.endereco != 0 {
                        existente.address = d.endereco;
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
                        apelido: String::new(),
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

    /// Tira um controle da lista.
    ///
    /// Nao e um "nunca mais": a descoberta reencontra o aparelho na hora em que ele for
    /// ligado de novo. O que se apaga e a lembranca de um controle que nao se usa mais --
    /// que hoje fica ocupando espaco na tela para sempre.
    pub fn esquecer(&mut self, chave: &str) -> bool {
        let antes = self.itens.len();
        self.itens.retain(|i| i.como_controle().chave() != chave);
        if self.itens.len() == antes {
            return false;
        }
        self.salvar();
        true
    }

    /// Devolve true quando o apelido mudou de fato.
    pub fn renomear(&mut self, chave: &str, nome: &str) -> bool {
        let Some(item) = self.itens.iter_mut().find(|i| i.como_controle().chave() == chave)
        else {
            return false;
        };

        // Um apelido igual ao nome do sistema nao e apelido nenhum: guardar isso deixaria
        // o arquivo com uma escolha que o usuario nao fez, e que passaria a segurar o nome
        // caso o sistema viesse a informar um melhor depois.
        let novo = match nome.trim() {
            n if n == item.name => "",
            n => n,
        };
        // apagar o apelido devolve o nome do sistema, que continua guardado
        if item.apelido == novo {
            return false;
        }
        item.apelido = novo.to_string();
        self.salvar();
        true
    }

    pub fn salvar(&self) {
        paths::garantir_dir();
        if let Ok(t) = serde_json::to_string(&self.itens) {
            let _ = std::fs::write(paths::arquivo("controllers.json"), t);
        }
    }
}
