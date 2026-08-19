using System;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Kontro
{
    /// <summary>
    /// Folha com todos os estados do icone da bandeja, nos dois temas de barra e nos
    /// quatro tamanhos reais.
    ///
    /// Cada celula mostra dois desenhos: a esquerda o que o app desenha em runtime, a
    /// direita o PNG entregue com o pacote de design. Os dois saem da mesma especificacao,
    /// entao qualquer diferenca visivel entre eles e um defeito da implementacao.
    /// </summary>
    internal static class IconPreview
    {
        private static readonly int[] Tamanhos = { 16, 20, 24, 32 };

        private static readonly (string Rotulo, string Arquivo, int Percentual, ControllerGeometry.TrayState Estado)[] Estados =
        {
            ("100",  "level-100", 100, ControllerGeometry.TrayState.Level),
            ("75",   "level-75",   75, ControllerGeometry.TrayState.Level),
            ("50",   "level-50",   50, ControllerGeometry.TrayState.Level),
            ("25",   "level-25",   25, ControllerGeometry.TrayState.Level),
            ("10",   "level-10",   10, ControllerGeometry.TrayState.Level),
            ("cabo", "cable",       0, ControllerGeometry.TrayState.Cable),
            ("off",  "off",         0, ControllerGeometry.TrayState.Disconnected),
        };

        internal static void Write(string path)
        {
            const int celula = 96, margem = 28, cabecalho = 30, vao = 6;
            int largura = margem * 2 + Estados.Length * celula;
            int alturaBanda = cabecalho + Tamanhos.Length * celula + 12;
            int altura = margem + alturaBanda * 2 + margem;

            var visual = new DrawingVisual();
            using (var dc = visual.RenderOpen())
            {
                dc.DrawRectangle(new SolidColorBrush(Color.FromRgb(0x0B, 0x0E, 0x11)), null,
                                 new Rect(0, 0, largura, altura));

                double y = margem;
                foreach (var claro in new[] { false, true })
                {
                    // a faixa reproduz a barra de tarefas correspondente: sem isso nao da
                    // para julgar se o glifo tem contraste onde ele vai viver
                    var fundo = claro
                        ? new SolidColorBrush(Color.FromRgb(0xF3, 0xF3, 0xF3))
                        : new SolidColorBrush(Color.FromRgb(0x1F, 0x1F, 0x1F));
                    var tinta = claro
                        ? new SolidColorBrush(Color.FromRgb(0x1B, 0x1F, 0x24))
                        : new SolidColorBrush(Color.FromRgb(0xE8, 0xEC, 0xEF));

                    dc.DrawRectangle(fundo, null, new Rect(0, y, largura, alturaBanda));
                    Texto(dc, claro ? "BARRA CLARA" : "BARRA ESCURA", tinta, 12, margem, y + 8);
                    Texto(dc, "runtime | entregue", tinta, 10, margem + 130, y + 9);

                    for (int c = 0; c < Estados.Length; c++)
                    {
                        var (rotulo, arquivo, pct, estado) = Estados[c];
                        double x = margem + c * celula;
                        Texto(dc, rotulo, tinta, 11, x + 4, y + cabecalho - 14);

                        for (int r = 0; r < Tamanhos.Length; r++)
                        {
                            int s = Tamanhos[r];
                            double linhaY = y + cabecalho + r * celula + (celula - s) / 2.0;
                            double meio = x + celula / 2.0;

                            var runtime = ControllerGeometry.RenderTray(pct, estado, s, claro, 30, 60);
                            dc.DrawImage(runtime, new Rect(meio - s - vao / 2.0, linhaY, s, s));

                            var entregue = Carregar(claro ? "light" : "dark", arquivo, s);
                            if (entregue != null)
                                dc.DrawImage(entregue, new Rect(meio + vao / 2.0, linhaY, s, s));
                        }
                    }

                    for (int r = 0; r < Tamanhos.Length; r++)
                        Texto(dc, Tamanhos[r] + "px", tinta, 10, 4,
                              y + cabecalho + r * celula + celula / 2.0 - 6);

                    y += alturaBanda;
                }
            }

            var bmp = new RenderTargetBitmap(largura, altura, 96, 96, PixelFormats.Pbgra32);
            bmp.Render(visual);

            using var fs = new FileStream(path, FileMode.Create);
            var enc = new PngBitmapEncoder();
            enc.Frames.Add(BitmapFrame.Create(bmp));
            enc.Save(fs);
        }

        private static BitmapImage Carregar(string tema, string arquivo, int tamanho)
        {
            try
            {
                var uri = new Uri($"pack://application:,,,/Assets/tray/{tema}/{arquivo}-{tamanho}.png");
                var img = new BitmapImage();
                img.BeginInit();
                img.UriSource = uri;
                img.CacheOption = BitmapCacheOption.OnLoad;
                img.EndInit();
                img.Freeze();
                return img;
            }
            catch { return null; }
        }

        private static void Texto(DrawingContext dc, string s, Brush cor, double tamanho, double x, double y)
        {
            var ft = new FormattedText(s, System.Globalization.CultureInfo.InvariantCulture,
                FlowDirection.LeftToRight,
                new Typeface(new FontFamily("Segoe UI"), FontStyles.Normal, FontWeights.SemiBold, FontStretches.Normal),
                tamanho, cor, 96);
            dc.DrawText(ft, new Point(x, y));
        }
    }
}
