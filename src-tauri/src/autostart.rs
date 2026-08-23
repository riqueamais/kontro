//! Subir junto com o Windows.
//!
//! E uma entrada em `HKCU\...\Run`, e nao uma tarefa agendada nem um atalho na pasta de
//! inicializacao: a chave por usuario nao pede administrador, aparece na aba de
//! inicializacao do Gerenciador de Tarefas -- onde a pessoa espera encontrar e poder
//! desligar -- e some junto com o perfil.

use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const CHAVE: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NOME: windows::core::PCWSTR = w!("Kontro");

/// O app sobe com o sistema?
pub fn ligado() -> bool {
    let Some(chave) = abrir(KEY_READ.0) else { return false };
    let mut tamanho = 0u32;
    let existe = unsafe {
        RegQueryValueExW(chave, NOME, None, None, None, Some(&mut tamanho)).is_ok()
    };
    unsafe {
        let _ = RegCloseKey(chave);
    }
    existe
}

/// Liga ou desliga. Devolve false quando o Windows recusou a escrita.
pub fn definir(ligar: bool) -> bool {
    let Some(chave) = abrir(KEY_WRITE.0) else { return false };

    let ok = if ligar {
        match linha_de_comando() {
            Some(comando) => {
                // O registro guarda texto em UTF-16 terminado em nulo, e o tamanho vai
                // em bytes -- nao em caracteres. Errar isso grava o valor pela metade.
                let mut unidades: Vec<u16> = comando.encode_utf16().collect();
                unidades.push(0);
                let bytes: Vec<u8> =
                    unidades.iter().flat_map(|u| u.to_le_bytes()).collect();
                unsafe { RegSetValueExW(chave, NOME, None, REG_SZ, Some(&bytes)).is_ok() }
            }
            None => false,
        }
    } else {
        // apagar o que ja nao existe nao e falha
        unsafe { RegDeleteValueW(chave, NOME).is_ok() || !ligado() }
    };

    unsafe {
        let _ = RegCloseKey(chave);
    }
    ok
}

fn abrir(acesso: u32) -> Option<HKEY> {
    let mut chave = HKEY::default();
    let ok = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            CHAVE,
            None,
            windows::Win32::System::Registry::REG_SAM_FLAGS(acesso),
            &mut chave,
        )
    };
    ok.is_ok().then_some(chave)
}

/// O caminho do executavel entre aspas, com o pedido de comecar escondido.
///
/// As aspas nao sao decorativas: o caminho tem espaco em "Program Files" e em qualquer
/// pasta de usuario com espaco no nome, e sem elas o Windows tentaria executar so o
/// primeiro pedaco.
fn linha_de_comando() -> Option<String> {
    let caminho = std::env::current_exe().ok()?;
    Some(format!("\"{}\" --minimizado", caminho.display()))
}
