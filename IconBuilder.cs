using System;
using System.Collections.Generic;
using System.IO;
using D = System.Drawing;
using System.Drawing.Drawing2D;

namespace Kontro
{
    /// <summary>
    /// Gera o app.ico a partir da geometria do controle, em todas as resolucoes que o
    /// Windows pede. Cada tamanho e redesenhado, nunca escalado a partir de outro: escalar
    /// um icone pequeno borra o traco.
    /// </summary>
    internal static class IconBuilder
    {
        private static readonly D.Color Branco = D.Color.FromArgb(0xFF, 0xFF, 0xFF);
        private static readonly D.Color FundoTopo = D.Color.FromArgb(0x1E, 0x1F, 0x22);
        private static readonly D.Color FundoBase = D.Color.FromArgb(0x0A, 0x0B, 0x0C);
        private static readonly D.Color Vazado = D.Color.FromArgb(0x11, 0x12, 0x15);

        private static readonly int[] Tamanhos = { 16, 20, 24, 32, 48, 64, 128, 256 };

        internal static void Write(string icoPath, string previewPath = null)
        {
            var pngs = new List<byte[]>();
            foreach (var s in Tamanhos) pngs.Add(RenderPng(s));
            WriteIco(icoPath, Tamanhos, pngs);

            if (previewPath != null) WritePreview(previewPath);
        }

        private static byte[] RenderPng(int size)
        {
            using var bmp = Render(size);
            using var ms = new MemoryStream();
            bmp.Save(ms, D.Imaging.ImageFormat.Png);
            return ms.ToArray();
        }

        internal static D.Bitmap Render(int size)
        {
            var bmp = new D.Bitmap(size, size, D.Imaging.PixelFormat.Format32bppArgb);
            using var g = D.Graphics.FromImage(bmp);
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.Clear(D.Color.Transparent);

            using (var bg = new GraphicsPath())
            {
                AddRounded(bg, 0, 0, size, size, size * 0.26f);
                using var brush = new LinearGradientBrush(
                    new D.PointF(0, 0), new D.PointF(0, size), FundoTopo, FundoBase);
                g.FillPath(brush, bg);
            }

            DrawCentered(g, size, 0.80f, dentro =>
            {
                // o contorno so se sustenta enquanto o traco tem pelo menos um pixel;
                // abaixo disso a silhueta cheia comunica melhor
                if (size >= 48) ControllerShape.DrawOutline(g, 100, Branco);
                else ControllerShape.DrawSolid(g, 100, Branco, Vazado);
            });

            return bmp;
        }

        /// <summary>
        /// Encaixa o desenho de 100x100 no icone: o controle e mais largo que alto,
        /// entao centralizamos pelo miolo real da forma, nao pelo meio da caixa.
        /// </summary>
        internal static void DrawCentered(D.Graphics g, float size, float ocupacao, Action<float> desenhar)
        {
            const float larguraForma = 95f;   // a forma vai de x=2.5 a x=97.5
            const float centroX = 50f;
            const float centroY = 51f;        // miolo vertical real, entre y=15 e y=87

            float escala = size * ocupacao / larguraForma;
            var estado = g.Save();
            g.TranslateTransform(size / 2f - centroX * escala, size / 2f - centroY * escala);
            g.ScaleTransform(escala, escala);
            desenhar(100f);
            g.Restore(estado);
        }

        private static void AddRounded(GraphicsPath p, float x, float y, float w, float h, float r)
        {
            float d = Math.Min(r * 2, Math.Min(w, h));
            p.AddArc(x, y, d, d, 180, 90);
            p.AddArc(x + w - d, y, d, d, 270, 90);
            p.AddArc(x + w - d, y + h - d, d, d, 0, 90);
            p.AddArc(x, y + h - d, d, d, 90, 90);
            p.CloseFigure();
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
            int pad = 16, x = pad, width = pad;
            foreach (var s in Tamanhos) width += s + pad;

            using var sheet = new D.Bitmap(Math.Max(width, 600), 256 + pad * 2 + 22);
            using var g = D.Graphics.FromImage(sheet);
            g.Clear(D.Color.FromArgb(0x08, 0x09, 0x0A));
            using var font = new D.Font("Segoe UI", 9);
            using var cinza = new D.SolidBrush(D.Color.FromArgb(0x8B, 0x95, 0xA1));

            foreach (var s in Tamanhos)
            {
                using var img = Render(s);
                g.DrawImageUnscaled(img, x, pad + (256 - s));
                g.DrawString(s + "px", font, cinza, x, pad + 258);
                x += s + pad;
            }
            sheet.Save(path, D.Imaging.ImageFormat.Png);
        }
    }
}
