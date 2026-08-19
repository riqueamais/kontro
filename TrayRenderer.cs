using System;
using D = System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;

namespace Kontro
{
    /// <summary>
    /// Desenha o icone da bandeja em tempo de execucao. Com controle ligado mostra um anel
    /// de carga com o numero no meio; no cabo, um raio; sem controle nenhum, a silhueta do
    /// gamepad cortada por uma barra. Renderizado no tamanho exato que o Windows pede para
    /// o DPI atual, senao o icone sai borrado.
    /// </summary>
    internal static class TrayRenderer
    {
        internal static readonly D.Color Good = D.Color.FromArgb(0xFF, 0xFF, 0xFF);
        internal static readonly D.Color Warn = D.Color.FromArgb(0xE3, 0xA9, 0x3C);
        internal static readonly D.Color Crit = D.Color.FromArgb(0xE5, 0x54, 0x4B);
        internal static readonly D.Color Idle = D.Color.FromArgb(0x8A, 0x90, 0x97);

        internal static D.Color ColorFor(int? percent, LinkMode mode)
        {
            if (mode == LinkMode.Offline || percent == null) return Idle;
            if (percent <= 10) return Crit;
            if (percent <= 25) return Warn;
            return Good;
        }

        internal static D.Icon Render(BatteryState state, int size)
        {
            if (size <= 0) size = 16;
            using var bmp = new D.Bitmap(size, size, D.Imaging.PixelFormat.Format32bppArgb);
            using (var g = D.Graphics.FromImage(bmp))
            {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
                g.InterpolationMode = InterpolationMode.HighQualityBicubic;
                g.Clear(D.Color.Transparent);

                // sem controle: nao ha carga para mostrar, entao o icone vira o proprio
                // gamepad cortado, que se explica sozinho
                if (state.Mode == LinkMode.Offline || !state.Percent.HasValue)
                {
                    DrawDisconnectedPad(g, size);
                }
                else
                {
                    DrawRing(g, size, state);
                }
            }

            IntPtr h = bmp.GetHicon();
            try
            {
                // clonamos porque Icon.FromHandle nao assume a posse do handle
                using var tmp = D.Icon.FromHandle(h);
                return (D.Icon)tmp.Clone();
            }
            finally { Native.DestroyIcon(h); }
        }

        private static void DrawRing(D.Graphics g, int size, BatteryState state)
        {
            var color = ColorFor(state.Percent, state.Mode);
            float thickness = Math.Max(1.6f, size / 9f);
            float inset = thickness / 2f + 0.5f;
            var rect = new D.RectangleF(inset, inset, size - inset * 2, size - inset * 2);

            using (var track = new D.Pen(D.Color.FromArgb(60, 255, 255, 255), thickness))
            {
                track.StartCap = LineCap.Round;
                track.EndCap = LineCap.Round;
                g.DrawEllipse(track, rect);
            }

            int pct = state.Percent ?? 0;
            float sweep = 360f * Math.Clamp(pct, 0, 100) / 100f;
            if (sweep > 0.5f)
            {
                using var arc = new D.Pen(color, thickness);
                arc.StartCap = LineCap.Round;
                arc.EndCap = LineCap.Round;
                g.DrawArc(arc, rect, -90f, sweep);
            }

            if (state.Mode == LinkMode.Cable) DrawBolt(g, size, color);
            else DrawNumber(g, size, pct);
        }

        private static void DrawNumber(D.Graphics g, int size, int pct)
        {
            // em icone pequeno "100" nao cabe de forma legivel, e o anel cheio ja diz tudo
            if (pct >= 100 && size <= 24) return;

            string text = pct >= 100 ? "100" : pct.ToString();
            float ratio = text.Length >= 3 ? 0.40f : (size <= 20 ? 0.66f : 0.56f);
            float em = size * ratio;

            using var font = new D.Font("Segoe UI", em, D.FontStyle.Bold, D.GraphicsUnit.Pixel);
            using var brush = new D.SolidBrush(D.Color.White);
            var fmt = D.StringFormat.GenericTypographic;
            var m = g.MeasureString(text, font, size, fmt);
            g.DrawString(text, font, brush, (size - m.Width) / 2f, (size - m.Height) / 2f, fmt);
        }

        private static void DrawBolt(D.Graphics g, int size, D.Color color)
        {
            var pts = new[]
            {
                new D.PointF(0.62f, 0.06f), new D.PointF(0.28f, 0.55f),
                new D.PointF(0.47f, 0.55f), new D.PointF(0.39f, 0.94f),
                new D.PointF(0.73f, 0.45f), new D.PointF(0.54f, 0.45f),
            };
            var scaled = new D.PointF[pts.Length];
            for (int i = 0; i < pts.Length; i++)
                scaled[i] = new D.PointF(pts[i].X * size, pts[i].Y * size);

            // contorno escuro para o raio nao sumir sobre o anel
            using (var outline = new D.Pen(D.Color.FromArgb(190, 0, 0, 0), Math.Max(1.2f, size / 13f)))
            {
                outline.LineJoin = LineJoin.Round;
                g.DrawPolygon(outline, scaled);
            }
            using var fill = new D.SolidBrush(D.Color.White);
            g.FillPolygon(fill, scaled);
        }

        /// <summary>Silhueta do gamepad cortada por uma barra: nenhum controle presente.</summary>
        private static void DrawDisconnectedPad(D.Graphics g, int s)
        {
            // mesma geometria do logo, em silhueta cheia: e o unico jeito de o controle
            // continuar reconhecivel nos 16px da bandeja
            IconBuilder.DrawCentered(g, s, 0.94f,
                _ => ControllerShape.DrawSolid(g, 100, Idle, D.Color.FromArgb(0x12, 0x13, 0x16)));

            var p1 = new D.PointF(s * 0.13f, s * 0.13f);
            var p2 = new D.PointF(s * 0.87f, s * 0.87f);
            float stroke = Math.Max(1.6f, s / 8.5f);

            // abre um vao transparente sob a barra para ela nao se fundir com o gamepad
            var modo = g.CompositingMode;
            g.CompositingMode = CompositingMode.SourceCopy;
            using (var vao = new D.Pen(D.Color.FromArgb(0, 0, 0, 0), stroke * 2.1f))
            {
                vao.StartCap = LineCap.Round;
                vao.EndCap = LineCap.Round;
                g.DrawLine(vao, p1, p2);
            }
            g.CompositingMode = modo;

            using var barra = new D.Pen(Idle, stroke);
            barra.StartCap = LineCap.Round;
            barra.EndCap = LineCap.Round;
            g.DrawLine(barra, p1, p2);
        }

        private static void AddRoundedRect(D.Drawing2D.GraphicsPath path, D.RectangleF r, float radius)
        {
            float d = radius * 2;
            if (d <= 0) { path.AddRectangle(r); return; }
            d = Math.Min(d, Math.Min(r.Width, r.Height));
            path.AddArc(r.X, r.Y, d, d, 180, 90);
            path.AddArc(r.Right - d, r.Y, d, d, 270, 90);
            path.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
            path.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
            path.CloseFigure();
        }
    }
}
