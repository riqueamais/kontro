use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetSystemMetrics,
    PostQuitMessage, RegisterClassW, SetTimer, SetWindowPos, HWND_TOPMOST, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WM_DESTROY, WM_TIMER, WNDCLASSW,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

const REAFIRMAR: usize = 1;
const ENCERRAR: usize = 2;

struct Opcoes {
    a_cada_ms: u32,
    por_segundos: u32,
    retangulo: Option<(i32, i32, i32, i32)>,
}

fn opcoes() -> Opcoes {
    let args: Vec<String> = std::env::args().collect();
    let valor = |nome: &str| args.iter().position(|a| a == nome).and_then(|i| args.get(i + 1));

    let a_cada_ms = if args.iter().any(|a| a == "--quadro") {
        16
    } else {
        valor("--a-cada").and_then(|v| v.parse().ok()).unwrap_or(0)
    };
    let por_segundos = valor("--por").and_then(|v| v.parse().ok()).unwrap_or(20);
    let retangulo = valor("--ret").and_then(|v| {
        let n: Vec<i32> = v.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (n.len() == 4).then(|| (n[0], n[1], n[2], n[3]))
    });

    Opcoes { a_cada_ms, por_segundos, retangulo }
}

unsafe extern "system" fn ao_receber(
    janela: HWND,
    mensagem: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match mensagem {
        WM_TIMER if wparam.0 == REAFIRMAR => {
            let _ = SetWindowPos(
                janela,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == ENCERRAR => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(janela, mensagem, wparam, lparam),
    }
}

fn main() -> windows::core::Result<()> {
    let opcoes = opcoes();

    unsafe {
        let instancia = GetModuleHandleW(None)?;
        let classe = WNDCLASSW {
            lpfnWndProc: Some(ao_receber),
            hInstance: instancia.into(),
            lpszClassName: w!("KontroJanelaTeimosa"),
            hbrBackground: CreateSolidBrush(windows::Win32::Foundation::COLORREF(0x00302020)),
            ..Default::default()
        };
        RegisterClassW(&classe);

        let (x, y, largura, altura) = opcoes.retangulo.unwrap_or((
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        ));

        let janela = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("KontroJanelaTeimosa"),
            w!("Janela teimosa"),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            largura,
            altura,
            None,
            None,
            Some(instancia.into()),
            None,
        )?;

        if opcoes.a_cada_ms > 0 {
            SetTimer(Some(janela), REAFIRMAR, opcoes.a_cada_ms, None);
        }
        SetTimer(Some(janela), ENCERRAR, opcoes.por_segundos * 1000, None);

        println!(
            "janela teimosa em ({x},{y}) {largura}x{altura}, reafirmando {}, por {} s",
            if opcoes.a_cada_ms > 0 {
                format!("a cada {} ms", opcoes.a_cada_ms)
            } else {
                "so ao abrir".to_string()
            },
            opcoes.por_segundos
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
