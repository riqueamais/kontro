use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const CHAVE: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NOME: windows::core::PCWSTR = w!("Kontro");

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

pub fn definir(ligar: bool) -> bool {
    let Some(chave) = abrir(KEY_WRITE.0) else { return false };

    let ok = if ligar {
        match linha_de_comando() {
            Some(comando) => {
                let mut unidades: Vec<u16> = comando.encode_utf16().collect();
                unidades.push(0);
                let bytes: Vec<u8> =
                    unidades.iter().flat_map(|u| u.to_le_bytes()).collect();
                unsafe { RegSetValueExW(chave, NOME, None, REG_SZ, Some(&bytes)).is_ok() }
            }
            None => false,
        }
    } else {
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

fn linha_de_comando() -> Option<String> {
    let caminho = std::env::current_exe().ok()?;
    Some(format!("\"{}\" --minimizado", caminho.display()))
}
