using System;
using System.Collections.Generic;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Kontro
{
    /// <summary>
    /// Gera o app.ico a partir da mesma geometria do sistema "Anel", nas nove resolucoes
    /// que o Windows pede. Cada tamanho e redesenhado no proprio tamanho, nunca escalado.
    ///
    /// O arquivo que acompanha o repositorio veio rasterizado do vetor original; isto aqui
    /// existe para regenerar um equivalente quando a geometria mudar.
    /// </summary>
    internal static class IconBuilder
    {
        private static readonly int[] Tamanhos = { 16, 20, 24, 32, 40, 48, 64, 128, 256 };

        /// <summary>Fracao do anel no icone de marca. E ilustrativa, nao mede nada.</summary>
        private const double FracaoIlustrativa = 0.72;

        internal static void Write(string icoPath, string previewPath = null)
        {
            var pngs = new List<byte[]>();
            foreach (var s in Tamanhos) pngs.Add(Encode(Render(s)));
            WriteIco(icoPath, Tamanhos, pngs);

            if (previewPath != null) WritePreview(previewPath);
        }

        /// <summary>Icone do aplicativo: fundo circular, anel em degrade e o controle solido.</summary>
        internal static BitmapSource Render(int size)
        {
            var visual = new DrawingVisual();
            using (var dc = visual.RenderOpen())
            {
                double s = size / 512.0;
                dc.PushTransform(new ScaleTransform(s, s));
                var centro = new Point(256, 256);

                var fundo = new SolidColorBrush(ControllerGeometry.Shell);
                dc.DrawEllipse(fundo, null, centro, 256, 256);

                var borda = new Pen(new SolidColorBrush(Color.FromArgb(26, 255, 255, 255)), 8);
                dc.DrawEllipse(null, borda, centro, 252, 252);

                // trilha e anel de carga
                var trilha = new Pen(new SolidColorBrush(Color.FromArgb(33, 255, 255, 255)), 30);
                dc.DrawEllipse(null, trilha, centro, 202, 202);

                var degrade = new LinearGradientBrush(
                    ControllerGeometry.Green, ControllerGeometry.Teal,
                    new Point(120.0 / 512, 80.0 / 512), new Point(400.0 / 512, 440.0 / 512));
                degrade.Freeze();
                DrawArc(dc, centro, 202, 30, FracaoIlustrativa, degrade);

                // controle: aqui os sticks sao pintados na cor do fundo, nao vazados,
                // porque o icone tem fundo proprio
                dc.PushTransform(new TranslateTransform(256, 274));
                dc.PushTransform(new ScaleTransform(0.6, 0.6));
                dc.PushTransform(new TranslateTransform(-256, -288));
                dc.DrawGeometry(Brushes.White, null, Geometry.Parse(ControllerGeometry.PadPath));
                dc.Pop(); dc.Pop(); dc.Pop();

                dc.DrawEllipse(fundo, null, new Point(180, 238), 29, 29);
                dc.DrawEllipse(fundo, null, new Point(332, 238), 29, 29);

                dc.Pop();
            }

            var bmp = new RenderTargetBitmap(size, size, 96, 96, PixelFormats.Pbgra32);
            bmp.Render(visual);
            bmp.Freeze();
            return bmp;
        }

        private static void DrawArc(DrawingContext dc, Point c, double r, double width,
                                    double fracao, Brush pincel)
        {
            var caneta = new Pen(pincel, width)
            {
                StartLineCap = PenLineCap.Round,
                EndLineCap = PenLineCap.Round
            };
            if (fracao >= 1) { dc.DrawEllipse(null, caneta, c, r, r); return; }

            double a0 = -Math.PI / 2;
            double a1 = a0 + fracao * 2 * Math.PI;
            var figura = new PathFigure
            {
                StartPoint = new Point(c.X + r * Math.Cos(a0), c.Y + r * Math.Sin(a0)),
                IsClosed = false
            };
            figura.Segments.Add(new ArcSegment(
                new Point(c.X + r * Math.Cos(a1), c.Y + r * Math.Sin(a1)),
                new Size(r, r), 0, fracao > 0.5, SweepDirection.Clockwise, true));
            var geo = new PathGeometry();
            geo.Figures.Add(figura);
            geo.Freeze();
            dc.DrawGeometry(null, caneta, geo);
        }

        private static byte[] Encode(BitmapSource bmp)
        {
            using var ms = new MemoryStream();
            var enc = new PngBitmapEncoder();
            enc.Frames.Add(BitmapFrame.Create(bmp));
            enc.Save(ms);
            return ms.ToArray();
        }

        /// <summary>ICO com cada imagem embutida como PNG, suportado desde o Vista.</summary>
        private static void WriteIco(string path, int[] sizes, List<byte[]> pngs)
        {
            using var fs = new FileStream(path, FileMode.Create, FileAccess.Write);
            using var w = new BinaryWriter(fs);
            w.Write((ushort)0); w.Write((ushort)1); w.Write((ushort)sizes.Length);

            int offset = 6 + 16 * sizes.Length;
            for (int i = 0; i < sizes.Length; i++)
            {
                int s = sizes[i];
                w.Write((byte)(s >= 256 ? 0 : s));
                w.Write((byte)(s >= 256 ? 0 : s));
                w.Write((byte)0); w.Write((byte)0);
                w.Write((ushort)1); w.Write((ushort)32);
                w.Write(pngs[i].Length); w.Write(offset);
                offset += pngs[i].Length;
            }
            foreach (var p in pngs) w.Write(p);
        }

        private static void WritePreview(string path)
        {
            const int pad = 16;
            int largura = pad;
            foreach (var s in Tamanhos) largura += s + pad;

            var visual = new DrawingVisual();
            using (var dc = visual.RenderOpen())
            {
                dc.DrawRectangle(new SolidColorBrush(Color.FromRgb(8, 9, 10)), null,
                                 new Rect(0, 0, largura, 256 + pad * 2));
                int x = pad;
                foreach (var s in Tamanhos)
                {
                    dc.DrawImage(Render(s), new Rect(x, pad + (256 - s), s, s));
                    x += s + pad;
                }
            }
            var bmp = new RenderTargetBitmap(largura, 256 + pad * 2, 96, 96, PixelFormats.Pbgra32);
            bmp.Render(visual);

            using var fs = new FileStream(path, FileMode.Create);
            var enc = new PngBitmapEncoder();
            enc.Frames.Add(BitmapFrame.Create(bmp));
            enc.Save(fs);
        }
    }
}
