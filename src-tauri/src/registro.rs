use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, KEY_READ,
    KEY_WRITE, REG_DWORD, REG_SAM_FLAGS, REG_SZ, REG_VALUE_TYPE,
};

pub fn texto(raiz: HKEY, chave: &str, nome: &str) -> Option<String> {
    let (tipo, bytes) = ler(raiz, chave, nome)?;
    (tipo == REG_SZ).then(|| decodificar(&bytes))
}

pub fn numero(raiz: HKEY, chave: &str, nome: &str) -> Option<u32> {
    let (tipo, bytes) = ler(raiz, chave, nome)?;
    if tipo != REG_DWORD || bytes.len() < 4 {
        return None;
    }
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub fn gravar_texto(raiz: HKEY, chave: &str, nome: &str, valor: &str) -> bool {
    let Some(aberta) = abrir(raiz, chave, KEY_WRITE.0) else { return false };

    let nome = larga(nome);
    let bytes: Vec<u8> = larga(valor).iter().flat_map(|u| u.to_le_bytes()).collect();

    let ok = unsafe {
        RegSetValueExW(aberta, PCWSTR(nome.as_ptr()), None, REG_SZ, Some(&bytes)).is_ok()
    };

    fechar(aberta);
    ok
}

pub fn apagar(raiz: HKEY, chave: &str, nome: &str) -> bool {
    let Some(aberta) = abrir(raiz, chave, KEY_WRITE.0) else { return false };

    let nome = larga(nome);
    let ok = unsafe { RegDeleteValueW(aberta, PCWSTR(nome.as_ptr())).is_ok() };

    fechar(aberta);
    ok
}

fn ler(raiz: HKEY, chave: &str, nome: &str) -> Option<(REG_VALUE_TYPE, Vec<u8>)> {
    let aberta = abrir(raiz, chave, KEY_READ.0)?;

    let nome = larga(nome);
    let nome = PCWSTR(nome.as_ptr());
    let mut tipo = REG_VALUE_TYPE::default();
    let mut tamanho = 0u32;

    let medida =
        unsafe { RegQueryValueExW(aberta, nome, None, Some(&mut tipo), None, Some(&mut tamanho)) };

    let achado = if medida.is_ok() && tamanho > 0 {
        let mut bytes = vec![0u8; tamanho as usize];
        let leitura = unsafe {
            RegQueryValueExW(
                aberta,
                nome,
                None,
                Some(&mut tipo),
                Some(bytes.as_mut_ptr()),
                Some(&mut tamanho),
            )
        };
        bytes.truncate((tamanho as usize).min(bytes.len()));
        leitura.is_ok().then_some((tipo, bytes))
    } else {
        None
    };

    fechar(aberta);
    achado
}

fn abrir(raiz: HKEY, chave: &str, acesso: u32) -> Option<HKEY> {
    let caminho = larga(chave);
    let mut aberta = HKEY::default();

    let ok = unsafe {
        RegOpenKeyExW(raiz, PCWSTR(caminho.as_ptr()), None, REG_SAM_FLAGS(acesso), &mut aberta)
    };

    ok.is_ok().then_some(aberta)
}

fn fechar(chave: HKEY) {
    unsafe {
        let _ = RegCloseKey(chave);
    }
}

fn larga(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

fn decodificar(bytes: &[u8]) -> String {
    let unidades: Vec<u16> =
        bytes.chunks_exact(2).map(|par| u16::from_le_bytes([par[0], par[1]])).collect();
    String::from_utf16_lossy(&unidades).trim_end_matches('\0').to_string()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_texto_do_registro_perde_o_zero_do_fim() {
        let bytes: Vec<u8> = "oi\0".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        assert_eq!(decodificar(&bytes), "oi");
    }

    #[test]
    fn a_conversao_para_wide_termina_em_zero() {
        assert_eq!(larga("ab"), vec![0x61, 0x62, 0x00]);
    }
}
