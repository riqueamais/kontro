use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUERY_USER_NOTIFICATION_STATE, QUNS_ACCEPTS_NOTIFICATIONS,
    QUNS_APP, QUNS_BUSY, QUNS_NOT_PRESENT, QUNS_PRESENTATION_MODE, QUNS_QUIET_TIME,
    QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsWindow,
    IsWindowVisible, GWL_EXSTYLE, GW_HWNDPREV, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

const PASSOS_ATE_O_TOPO: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tela {
    Livre,
    Composta,
    Exclusiva,
    Apresentacao,
    Outro(i32),
}

impl Tela {
    pub fn atual() -> Tela {
        match consultar() {
            Some(estado) => Tela::de(estado),
            None => Tela::Outro(-1),
        }
    }

    fn de(estado: QUERY_USER_NOTIFICATION_STATE) -> Tela {
        match estado {
            QUNS_BUSY => Tela::Composta,
            QUNS_RUNNING_D3D_FULL_SCREEN => Tela::Exclusiva,
            QUNS_PRESENTATION_MODE => Tela::Apresentacao,
            QUNS_ACCEPTS_NOTIFICATIONS => Tela::Livre,
            outro => Tela::Outro(outro.0),
        }
    }

    pub fn conta_como_jogo(self) -> bool {
        matches!(self, Tela::Composta | Tela::Exclusiva | Tela::Apresentacao)
    }

    pub fn descrever(self) -> String {
        match self {
            Tela::Livre => "Livre (QUNS_ACCEPTS_NOTIFICATIONS)".into(),
            Tela::Composta => "Composta (QUNS_BUSY)".into(),
            Tela::Exclusiva => "Exclusiva (QUNS_RUNNING_D3D_FULL_SCREEN)".into(),
            Tela::Apresentacao => "Apresentacao (QUNS_PRESENTATION_MODE)".into(),
            Tela::Outro(-1) => "desconhecida (a consulta falhou)".into(),
            Tela::Outro(bruto) => {
                let nome = match QUERY_USER_NOTIFICATION_STATE(bruto) {
                    QUNS_NOT_PRESENT => "QUNS_NOT_PRESENT",
                    QUNS_QUIET_TIME => "QUNS_QUIET_TIME",
                    QUNS_APP => "QUNS_APP",
                    _ => "sem nome",
                };
                format!("Outro ({nome})")
            }
        }
    }

    pub fn bruto(self) -> i32 {
        match self {
            Tela::Livre => QUNS_ACCEPTS_NOTIFICATIONS.0,
            Tela::Composta => QUNS_BUSY.0,
            Tela::Exclusiva => QUNS_RUNNING_D3D_FULL_SCREEN.0,
            Tela::Apresentacao => QUNS_PRESENTATION_MODE.0,
            Tela::Outro(bruto) => bruto,
        }
    }
}

fn consultar() -> Option<QUERY_USER_NOTIFICATION_STATE> {
    unsafe { SHQueryUserNotificationState().ok() }
}

pub fn centro_da_janela_em_foco() -> Option<(f64, f64)> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe {
        let janela = GetForegroundWindow();
        if janela.is_invalid() {
            return None;
        }
        let area = retangulo(janela)?;
        Some(((area.left + area.right) as f64 / 2.0, (area.top + area.bottom) as f64 / 2.0))
    }
}

pub fn retangulo(janela: HWND) -> Option<RECT> {
    let mut area = RECT::default();
    unsafe { GetWindowRect(janela, &mut area).ok()? };
    (area.right > area.left && area.bottom > area.top).then_some(area)
}

pub fn alguem_por_cima(pilula: HWND) -> Option<HWND> {
    let alvo = retangulo(pilula)?;
    let meu = std::process::id();

    let mut atual = pilula;
    for _ in 0..PASSOS_ATE_O_TOPO {
        atual = unsafe { GetWindow(atual, GW_HWNDPREV).ok()? };
        if atual.is_invalid() || !unsafe { IsWindow(Some(atual)).as_bool() } {
            return None;
        }
        if cobre(atual, &alvo, meu) {
            return Some(atual);
        }
    }
    None
}

fn cobre(janela: HWND, alvo: &RECT, meu: u32) -> bool {
    unsafe {
        if !IsWindowVisible(janela).as_bool() || escondida_pelo_dwm(janela) {
            return false;
        }

        let estilo = GetWindowLongPtrW(janela, GWL_EXSTYLE) as u32;
        if estilo & (WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0) != 0 {
            return false;
        }

        let mut processo = 0u32;
        GetWindowThreadProcessId(janela, Some(&mut processo));
        if processo == meu {
            return false;
        }
    }

    retangulo(janela).is_some_and(|area| se_cruzam(&area, alvo))
}

fn escondida_pelo_dwm(janela: HWND) -> bool {
    let mut encoberta = 0u32;
    let leu = unsafe {
        DwmGetWindowAttribute(
            janela,
            DWMWA_CLOAKED,
            &mut encoberta as *mut u32 as *mut core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    leu.is_ok() && encoberta != 0
}

fn se_cruzam(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

#[cfg(test)]
mod testes {
    use super::*;

    fn caixa(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT { left, top, right, bottom }
    }

    #[test]
    fn os_tres_estados_de_tela_cheia_continuam_contando_como_jogo() {
        assert!(Tela::de(QUNS_BUSY).conta_como_jogo());
        assert!(Tela::de(QUNS_RUNNING_D3D_FULL_SCREEN).conta_como_jogo());
        assert!(Tela::de(QUNS_PRESENTATION_MODE).conta_como_jogo());
    }

    #[test]
    fn o_resto_nao_conta_como_jogo() {
        assert!(!Tela::de(QUNS_ACCEPTS_NOTIFICATIONS).conta_como_jogo());
        assert!(!Tela::de(QUNS_APP).conta_como_jogo());
        assert!(!Tela::de(QUNS_QUIET_TIME).conta_como_jogo());
        assert!(!Tela::de(QUNS_NOT_PRESENT).conta_como_jogo());
    }

    #[test]
    fn composta_e_exclusiva_sao_estados_diferentes() {
        assert_eq!(Tela::de(QUNS_BUSY), Tela::Composta);
        assert_eq!(Tela::de(QUNS_RUNNING_D3D_FULL_SCREEN), Tela::Exclusiva);
    }

    #[test]
    fn o_valor_bruto_volta_igual() {
        for estado in [QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, QUNS_APP, QUNS_QUIET_TIME] {
            assert_eq!(Tela::de(estado).bruto(), estado.0);
        }
    }

    #[test]
    fn caixas_que_so_encostam_nao_se_cruzam() {
        assert!(!se_cruzam(&caixa(0, 0, 100, 100), &caixa(100, 0, 200, 100)));
        assert!(se_cruzam(&caixa(0, 0, 100, 100), &caixa(99, 99, 200, 200)));
        assert!(se_cruzam(&caixa(0, 0, 1920, 1080), &caixa(1700, 1000, 1900, 1070)));
    }
}
