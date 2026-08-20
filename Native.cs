using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace Kontro
{
    /// <summary>P/Invokes crus: XInput (deteccao de cabo) e limpeza de HICON.</summary>
    internal static class Native
    {
        [DllImport("user32.dll", SetLastError = true)]
        internal static extern bool DestroyIcon(IntPtr hIcon);

        [DllImport("dwmapi.dll")]
        private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);

        private const int DwmUseImmersiveDarkMode = 20;
        private const int DwmUseImmersiveDarkModeLegacy = 19; // builds do Windows 10 anteriores a 20H1

        [DllImport("user32.dll", SetLastError = true)]
        private static extern int GetWindowLong(IntPtr hwnd, int index);

        [DllImport("user32.dll", SetLastError = true)]
        private static extern int SetWindowLong(IntPtr hwnd, int index, int novoEstilo);

        [DllImport("user32.dll")]
        internal static extern IntPtr GetForegroundWindow();

        private const int GwlExStyle = -20;
        private const int WsExTransparent = 0x00000020;  // o mouse atravessa a janela
        private const int WsExLayered = 0x00080000;      // exigido pelo transparente
        private const int WsExNoActivate = 0x08000000;   // nunca recebe foco
        private const int WsExToolWindow = 0x00000080;   // fora do Alt+Tab

        /// <summary>
        /// Deixa a janela puramente visual: o clique passa direto para o que estiver
        /// atras, ela nunca rouba o foco e nao aparece no Alt+Tab.
        ///
        /// Sem isso, um aviso sobreposto durante o jogo tiraria o foco da partida no
        /// pior momento possivel, e um clique perdido pousaria nela em vez do jogo.
        /// </summary>
        internal static void MakeClickThrough(IntPtr hwnd)
        {
            if (hwnd == IntPtr.Zero) return;
            try
            {
                int estilo = GetWindowLong(hwnd, GwlExStyle);
                SetWindowLong(hwnd, GwlExStyle,
                    estilo | WsExTransparent | WsExLayered | WsExNoActivate | WsExToolWindow);
            }
            catch { }
        }

        [DllImport("shell32.dll")]
        private static extern int SHQueryUserNotificationState(out int estado);

        /// <summary>Estados relatados pelo Windows sobre o que ocupa a tela.</summary>
        internal enum EstadoDaTela
        {
            Desconhecido = 0,
            NaoExisteSessao = 1,
            TelaCheiaDirectX = 2,   // jogo em tela cheia exclusiva
            JanelaOcupandoTudo = 3, // tela cheia em janela, o caso comum hoje
            Apresentacao = 4,
            AceitaNotificacoes = 5,
            HorarioSilencioso = 6,
            AppDaLojaEmTelaCheia = 7
        }

        internal static EstadoDaTela ConsultarTela()
        {
            try
            {
                if (SHQueryUserNotificationState(out int estado) == 0)
                    return (EstadoDaTela)estado;
            }
            catch { }
            return EstadoDaTela.Desconhecido;
        }

        /// <summary>
        /// Ha algo ocupando a tela inteira, seja jogo ou apresentacao.
        ///
        /// Nao ha API para "esta jogando", e vasculhar nomes de processo seria uma lista
        /// infinita e sempre desatualizada. O que da para saber com precisao e se algo
        /// tomou a tela, que na pratica e quando o usuario quer o dado sobreposto.
        /// </summary>
        internal static bool EmTelaCheia()
        {
            var e = ConsultarTela();
            return e == EstadoDaTela.TelaCheiaDirectX
                || e == EstadoDaTela.JanelaOcupandoTudo
                || e == EstadoDaTela.Apresentacao
                || e == EstadoDaTela.AppDaLojaEmTelaCheia;
        }

        /// <summary>
        /// Tela cheia exclusiva: nenhuma janela comum e desenhada por cima. Saber disso
        /// permite explicar ao usuario por que a sobreposicao nao aparece, em vez de
        /// deixa-lo achando que quebrou.
        /// </summary>
        internal static bool EmTelaCheiaExclusiva() =>
            ConsultarTela() == EstadoDaTela.TelaCheiaDirectX;

        private const int DwmCaptionColor = 35;
        private const int DwmTextColor = 36;
        private const int DwmBorderColor = 34;

        /// <summary>
        /// Pinta a barra de titulo de escuro. Sem isso a janela fica com uma faixa clara
        /// em cima do conteudo escuro, o que denuncia na hora que o app foi feito as pressas.
        ///
        /// So o modo escuro nao basta: no Windows 11 a barra usa material translucido e
        /// acaba puxando o brilho do que estiver atras da janela. Fixar a cor da legenda
        /// resolve, e os atributos de cor sao ignorados em versoes antigas.
        /// </summary>
        internal static void UseDarkTitleBar(IntPtr hwnd, int captionRgb = 0x0C0D0F, int textRgb = 0xF5F6F7)
        {
            if (hwnd == IntPtr.Zero) return;
            try
            {
                int on = 1;
                if (DwmSetWindowAttribute(hwnd, DwmUseImmersiveDarkMode, ref on, sizeof(int)) != 0)
                    DwmSetWindowAttribute(hwnd, DwmUseImmersiveDarkModeLegacy, ref on, sizeof(int));

                int caption = ToColorRef(captionRgb);
                int text = ToColorRef(textRgb);
                int borda = ToColorRef(0x232427);
                DwmSetWindowAttribute(hwnd, DwmCaptionColor, ref caption, sizeof(int));
                DwmSetWindowAttribute(hwnd, DwmTextColor, ref text, sizeof(int));
                DwmSetWindowAttribute(hwnd, DwmBorderColor, ref borda, sizeof(int));
            }
            catch { }
        }

        /// <summary>O DWM espera COLORREF (0x00BBGGRR), nao o RGB usual.</summary>
        private static int ToColorRef(int rgb)
        {
            int r = (rgb >> 16) & 0xFF, g = (rgb >> 8) & 0xFF, b = rgb & 0xFF;
            return (b << 16) | (g << 8) | r;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct XINPUT_BATTERY_INFORMATION
        {
            public byte BatteryType;
            public byte BatteryLevel;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct XINPUT_CAPABILITIES
        {
            public byte Type;
            public byte SubType;
            public ushort Flags;
            public ushort wButtons;
            public byte bLeftTrigger, bRightTrigger;
            public short sThumbLX, sThumbLY, sThumbRX, sThumbRY;
            public ushort wLeftMotorSpeed, wRightMotorSpeed;
        }

        internal const byte BATTERY_TYPE_DISCONNECTED = 0x00;
        internal const byte BATTERY_TYPE_WIRED = 0x01;
        internal const uint ERROR_SUCCESS = 0;
        internal const uint BATTERY_DEVTYPE_GAMEPAD = 0;

        [DllImport("xinput1_4.dll")]
        private static extern uint XInputGetBatteryInformation(uint dwUserIndex, byte devType, out XINPUT_BATTERY_INFORMATION info);

        [DllImport("xinput1_4.dll")]
        private static extern uint XInputGetCapabilities(uint dwUserIndex, uint dwFlags, out XINPUT_CAPABILITIES caps);

        /// <summary>
        /// XInput distingue ligacao com fio de ligacao a bateria direto no BatteryType.
        /// E a forma mais confiavel de saber que o controle esta no cabo: no modo GIP/USB
        /// ele reporta WIRED, enquanto no Bluetooth reporta ALKALINE ou NIMH.
        /// </summary>
        internal static bool AnyControllerWired()
        {
            for (uint i = 0; i < 4; i++)
            {
                try
                {
                    if (XInputGetCapabilities(i, 0, out _) != ERROR_SUCCESS) continue;
                    if (XInputGetBatteryInformation(i, (byte)BATTERY_DEVTYPE_GAMEPAD, out var info) != ERROR_SUCCESS) continue;
                    if (info.BatteryType == BATTERY_TYPE_WIRED) return true;
                }
                catch (DllNotFoundException) { return false; }
                catch (EntryPointNotFoundException) { return false; }
            }
            return false;
        }

        /// <summary>
        /// Controle visivel ao XInput e alimentado por bateria (alcalina ou NiMH), ou seja,
        /// sem fio. Serve para nao confundir uma ligacao Bluetooth com uma ligacao por cabo.
        /// </summary>
        internal static bool AnyControllerBatteryPowered()
        {
            for (uint i = 0; i < 4; i++)
            {
                try
                {
                    if (XInputGetCapabilities(i, 0, out _) != ERROR_SUCCESS) continue;
                    if (XInputGetBatteryInformation(i, (byte)BATTERY_DEVTYPE_GAMEPAD, out var info) != ERROR_SUCCESS) continue;
                    if (info.BatteryType is 0x02 or 0x03) return true;
                }
                catch { return false; }
            }
            return false;
        }

        /// <summary>
        /// Bateria que o XInput relata por slot. E grosseira -- quatro niveis, nao
        /// percentual -- mas e a unica fonte que funciona igual para controle no cabo,
        /// no adaptador sem fio e em dongle generico, sem depender de Bluetooth.
        /// </summary>
        internal static IEnumerable<(int Slot, byte Tipo, byte Nivel, string Descricao)> BateriasXInput()
        {
            string[] tipos = { "desconectado", "com fio", "alcalina", "recarregavel" };
            string[] niveis = { "vazia", "baixa", "media", "cheia" };

            for (uint i = 0; i < 4; i++)
            {
                XINPUT_BATTERY_INFORMATION info;
                try
                {
                    if (XInputGetCapabilities(i, 0, out _) != ERROR_SUCCESS) continue;
                    if (XInputGetBatteryInformation(i, (byte)BATTERY_DEVTYPE_GAMEPAD, out info) != ERROR_SUCCESS)
                        continue;
                }
                catch { yield break; }

                string t = info.BatteryType < tipos.Length ? tipos[info.BatteryType] : "desconhecido";
                string n = info.BatteryLevel < niveis.Length ? niveis[info.BatteryLevel] : "desconhecido";
                yield return ((int)i, info.BatteryType, info.BatteryLevel, $"{t} / {n}");
            }
        }

        internal static bool AnyControllerPresent()
        {
            for (uint i = 0; i < 4; i++)
            {
                try { if (XInputGetCapabilities(i, 0, out _) == ERROR_SUCCESS) return true; }
                catch { return false; }
            }
            return false;
        }
    }
}



