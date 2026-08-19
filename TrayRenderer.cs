using System;
using System.IO;
using System.Windows.Media.Imaging;
using Microsoft.Win32;
using D = System.Drawing;

namespace Kontro
{
    /// <summary>
    /// Produz o icone da bandeja a partir da geometria do sistema "Anel".
    ///
    /// O desenho e feito no tamanho exato que o Windows vai exibir. Escalar um icone de
    /// 16 para 32, ou o contrario, borra o traco — e a bandeja e justamente onde o
    /// desenho tem menos pixels para se explicar.
    /// </summary>
    internal static class TrayRenderer
    {
        /// <summary>
        /// A barra de tarefas tem tema proprio, independente do tema das janelas.
        /// Glifo branco sobre barra escura, escuro sobre barra clara.
        /// </summary>
        internal static bool IsLightTaskbar()
        {
            try
            {
                using var k = Registry.CurrentUser.OpenSubKey(
                    @"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
                return k?.GetValue("SystemUsesLightTheme") is int v && v == 1;
            }
            catch { return false; }
        }

        internal static D.Icon Render(BatteryState state, int size, Settings settings)
        {
            if (size <= 0) size = 16;

            var (estado, percentual) = Classify(state);
            int vermelhoAbaixo = settings?.RingRedBelow ?? 30;
            int ambarAbaixo = settings?.RingAmberBelow ?? 60;

            var bitmap = ControllerGeometry.RenderTray(
                percentual, estado, size, IsLightTaskbar(), vermelhoAbaixo, ambarAbaixo);

            return ToIcon(bitmap);
        }

        private static (ControllerGeometry.TrayState, int) Classify(BatteryState s)
        {
            if (s.Mode == LinkMode.Cable) return (ControllerGeometry.TrayState.Cable, 0);
            if (s.Mode == LinkMode.Offline || !s.Percent.HasValue)
                return (ControllerGeometry.TrayState.Disconnected, 0);
            return (ControllerGeometry.TrayState.Level, s.Percent.Value);
        }

        /// <summary>
        /// Converte o desenho do WPF em um icone que o NotifyIcon aceita. O caminho passa
        /// por PNG porque e o unico formato que preserva o canal alfa nessa travessia.
        /// </summary>
        private static D.Icon ToIcon(BitmapSource origem)
        {
            using var ms = new MemoryStream();
            var codificador = new PngBitmapEncoder();
            codificador.Frames.Add(BitmapFrame.Create(origem));
            codificador.Save(ms);
            ms.Position = 0;

            using var bmp = new D.Bitmap(ms);
            IntPtr h = bmp.GetHicon();
            try
            {
                // clonamos porque Icon.FromHandle nao assume a posse do handle
                using var tmp = D.Icon.FromHandle(h);
                return (D.Icon)tmp.Clone();
            }
            finally { Native.DestroyIcon(h); }
        }
    }
}
