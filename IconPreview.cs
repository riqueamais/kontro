using System;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Kontro
{
    /// <summary>
    /// Folha com todos os estados do icone da bandeja, nos dois temas de barra e nos
    /// quatro tamanhos reais — exatamente os arquivos que o app carrega em execucao.
    ///
    /// Existe porque na bandeja o icone e pequeno demais para se avaliar a olho, e
    /// porque falta de contraste so vira problema visivel quando os dois temas ficam
    /// lado a lado.
    /// </summary>
    internal static class IconPreview
    {
        private static readonly int[] Tamanhos = { 16, 20, 24, 32 };

        private static readonly (string Rotulo, string Arquivo)[] Estados =
        {
            ("100%", "level-100"),
            ("75%",  "level-75"),
            ("50%",  "level-50"),
            ("25%",  "level-25"),
            ("10%",  "level-10"),
            ("cabo", "cable"),
            ("off",  "off"),
        };

        internal static void Write(string path)
        {
            const int celula = 78, margem = 28, cabecalho = 34;
            int largura = margem * 2 + Estados.Length * celula;
            int alturaBanda = cabecalho + Tamanhos.Length * celula + 14;
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

                    for (int c = 0; c < Estados.Length; c++)
                    {
                        var (rotulo, arquivo) = Estados[c];
                        double x = margem + c * celula;
                        Texto(dc, rotulo, tinta, 11, x + celula / 2.0 - 12, y + cabecalho - 16);

                        for (int r = 0; r < Tamanhos.Length; r++)
                        {
                            int s = Tamanhos[r];
                            var img = TrayRenderer.CarregarImagem(claro ? "light" : "dark", arquivo, s);
                            if (img == null) continue;
                            dc.DrawImage(img, new Rect(
                                x + (celula - s) / 2.0,
                                y + cabecalho + r * celula + (celula - s) / 2.0,
                                s, s));
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
