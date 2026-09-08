use std::path::Path;

use tiny_skia::{IntSize, Pixmap};
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

const LADO_MAXIMO: i32 = 256;

pub fn como_uri(caminho: &Path) -> Option<String> {
    let png = desenhar(caminho)?;
    Some(format!("data:image/png;base64,{}", base64(&png)))
}

fn desenhar(caminho: &Path) -> Option<Vec<u8>> {
    let icone = extrair(caminho)?;
    let pintado = em_pixels(icone);

    unsafe {
        let _ = DestroyIcon(icone);
    }

    let (largura, altura, rgba) = pintado?;
    let tamanho = IntSize::from_wh(largura, altura)?;
    let mapa = Pixmap::from_vec(rgba, tamanho)?;
    mapa.encode_png().ok()
}

fn extrair(caminho: &Path) -> Option<HICON> {
    let largo: Vec<u16> =
        caminho.as_os_str().to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut info = SHFILEINFOW::default();
        let ok = SHGetFileInfoW(
            PCWSTR(largo.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        (ok != 0 && !info.hIcon.is_invalid()).then_some(info.hIcon)
    }
}

fn em_pixels(icone: HICON) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        let mut partes = ICONINFO::default();
        GetIconInfo(icone, &mut partes).ok()?;

        let cor = partes.hbmColor;
        let mascara = partes.hbmMask;

        let medida = medir(cor.into());
        let apagar = || {
            let _ = DeleteObject(cor.into());
            let _ = DeleteObject(mascara.into());
        };

        let Some((largura, altura)) = medida else {
            apagar();
            return None;
        };

        let mut cabecalho = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: largura,
                biHeight: -altura,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bytes = vec![0u8; (largura * altura * 4) as usize];
        let tela = GetDC(None);
        let linhas = GetDIBits(
            tela,
            cor,
            0,
            altura as u32,
            Some(bytes.as_mut_ptr() as *mut core::ffi::c_void),
            &mut cabecalho,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, tela);
        apagar();

        if linhas == 0 {
            return None;
        }

        Some((largura as u32, altura as u32, para_rgba(bytes)))
    }
}

fn medir(bitmap: HGDIOBJ) -> Option<(i32, i32)> {
    unsafe {
        let mut forma = BITMAP::default();
        let lido = GetObjectW(
            bitmap,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut forma as *mut _ as *mut core::ffi::c_void),
        );

        if lido == 0 || forma.bmWidth <= 0 || forma.bmHeight <= 0 {
            return None;
        }
        if forma.bmWidth > LADO_MAXIMO || forma.bmHeight > LADO_MAXIMO {
            return None;
        }
        Some((forma.bmWidth, forma.bmHeight))
    }
}

fn para_rgba(mut bytes: Vec<u8>) -> Vec<u8> {
    let opaco = bytes.chunks_exact(4).all(|p| p[3] == 0);

    for pixel in bytes.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        if opaco {
            pixel[3] = 255;
        } else {
            let alfa = pixel[3] as u32;
            for canal in 0..3 {
                pixel[canal] = ((pixel[canal] as u32 * alfa + 127) / 255) as u8;
            }
        }
    }

    bytes
}

const ALFABETO: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut saida = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for trio in bytes.chunks(3) {
        let a = trio[0] as u32;
        let b = *trio.get(1).unwrap_or(&0) as u32;
        let c = *trio.get(2).unwrap_or(&0) as u32;
        let junto = (a << 16) | (b << 8) | c;

        saida.push(ALFABETO[(junto >> 18) as usize & 63] as char);
        saida.push(ALFABETO[(junto >> 12) as usize & 63] as char);
        saida.push(if trio.len() > 1 { ALFABETO[(junto >> 6) as usize & 63] as char } else { '=' });
        saida.push(if trio.len() > 2 { ALFABETO[junto as usize & 63] as char } else { '=' });
    }

    saida
}

#[cfg(test)]
pub fn base64_para_teste(bytes: &[u8]) -> String {
    base64(bytes)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_base64_bate_com_os_exemplos_da_rfc() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn o_bgra_do_windows_vira_rgba() {
        let bgra = vec![10, 20, 30, 255];
        assert_eq!(para_rgba(bgra), vec![30, 20, 10, 255]);
    }

    #[test]
    fn icone_sem_alfa_nenhum_vira_opaco() {
        let sem_alfa = vec![10, 20, 30, 0];
        assert_eq!(para_rgba(sem_alfa), vec![30, 20, 10, 255]);
    }

    #[test]
    fn o_icone_de_um_exe_de_verdade_vira_png() {
        let conhecido = Path::new(r"C:\Windows\System32\notepad.exe");
        if !conhecido.exists() {
            return;
        }
        let uri = como_uri(conhecido);
        assert!(uri.is_some(), "nao consegui extrair o icone do notepad");
        let uri = uri.unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
        assert!(uri.len() > 200);
    }
}
