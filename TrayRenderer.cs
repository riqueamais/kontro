using System;
using System.Collections.Generic;
using System.IO;
using System.Windows.Media.Imaging;
using Microsoft.Win32;
using D = System.Drawing;

namespace Kontro
{
    /// <summary>
    /// Monta o icone da bandeja a partir dos PNGs do pacote de design.
    ///
    /// A alternativa seria desenhar em runtime, o que daria percentual continuo no anel.
    /// Optamos pelos arquivos porque eles sao a referencia visual: desenhar de novo
    /// significaria perseguir a aparencia por aproximacao, e qualquer ajuste futuro no
    /// vetor abriria uma diferenca silenciosa entre o design e o app.
    ///
    /// O preco e que o anel mostra o degrau mais proximo entre 100, 75, 50, 25 e 10.
    /// O valor exato continua no tooltip e no painel, que e onde ele se le de verdade.
    /// </summary>
    internal static class TrayRenderer
    {
        /// <summary>Tamanhos rasterizados no pacote. Nunca escalamos um para outro.</summary>
        private static readonly int[] TamanhosDisponiveis = { 16, 20, 24, 32 };

        /// <summary>Degraus de carga que existem como arquivo.</summary>
        private static readonly int[] Degraus = { 100, 75, 50, 25, 10 };

        private static readonly Dictionary<string, D.Icon> Cache = new();

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
            string nome = NomeDoEstado(state);
            int tamanho = TamanhoMaisProximo(size);
            string tema = IsLightTaskbar() ? "light" : "dark";

            string chave = $"{tema}/{nome}-{tamanho}";
            if (Cache.TryGetValue(chave, out var pronto)) return pronto;

            var icone = Carregar(tema, nome, tamanho);
            if (icone != null) Cache[chave] = icone;
            return icone;
        }

        private static string NomeDoEstado(BatteryState s)
        {
            if (s.Mode == LinkMode.Cable) return "cable";
            if (s.Mode == LinkMode.Offline || !s.Percent.HasValue) return "off";
            return "level-" + DegrauMaisProximo(s.Percent.Value);
        }

        internal static int DegrauMaisProximo(int percentual)
        {
            int melhor = Degraus[0];
            int menorDistancia = int.MaxValue;
            foreach (var d in Degraus)
            {
                int dist = Math.Abs(d - percentual);
                if (dist < menorDistancia) { menorDistancia = dist; melhor = d; }
            }
            return melhor;
        }

        /// <summary>
        /// O Windows pede o icone no tamanho de SM_CXSMICON, que varia com a escala:
        /// 16 a 100%, 20 a 125%, 24 a 150%, 32 a 200%. Cair no arquivo exato evita o
        /// borrao de reescalar.
        /// </summary>
        private static int TamanhoMaisProximo(int pedido)
        {
            if (pedido <= 0) return 16;
            int melhor = TamanhosDisponiveis[0];
            int menorDistancia = int.MaxValue;
            foreach (var t in TamanhosDisponiveis)
            {
                int dist = Math.Abs(t - pedido);
                if (dist < menorDistancia) { menorDistancia = dist; melhor = t; }
            }
            return melhor;
        }

        private static D.Icon Carregar(string tema, string nome, int tamanho)
        {
            try
            {
                var uri = new Uri($"pack://application:,,,/Assets/tray/{tema}/{nome}-{tamanho}.png");
                var img = new BitmapImage();
                img.BeginInit();
                img.UriSource = uri;
                img.CacheOption = BitmapCacheOption.OnLoad;
                img.EndInit();
                img.Freeze();
                return ToIcon(img);
            }
            catch { return null; }
        }

        /// <summary>
        /// Converte a imagem em um icone que o NotifyIcon aceita. O caminho passa por PNG
        /// porque e o unico formato que preserva o canal alfa nessa travessia.
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

        /// <summary>Descarta o cache quando o tema da barra muda.</summary>
        internal static void LimparCache()
        {
            foreach (var i in Cache.Values) { try { i.Dispose(); } catch { } }
            Cache.Clear();
        }
    }
}
