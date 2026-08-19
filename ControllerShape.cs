using System;
using D = System.Drawing;
using System.Drawing.Drawing2D;

namespace Kontro
{
    /// <summary>
    /// A geometria do controle, em um sistema de 100x100 escalado para o tamanho pedido.
    ///
    /// Existe em duas leituras porque nenhuma serve nos dois lugares: o logo usa o
    /// contorno, que tem ar e detalhe; a bandeja usa a silhueta cheia, porque em 16px
    /// um traco de meio pixel simplesmente desaparece.
    /// </summary>
    internal static class ControllerShape
    {
        /// <summary>Contorno do corpo: ombros no topo, empunhaduras descendo para os cantos.</summary>
        internal static GraphicsPath Body(float s)
        {
            float f = s / 100f;
            D.PointF P(float x, float y) => new(x * f, y * f);

            var p = new GraphicsPath();

            // topo: ponta do ombro esquerdo, plato central, ponta do ombro direito
            p.AddLine(P(17.5f, 16.5f), P(38.5f, 20.5f));
            p.AddLine(P(38.5f, 20.5f), P(61.5f, 20.5f));
            p.AddLine(P(61.5f, 20.5f), P(82.5f, 16.5f));

            // lado direito descendo ate a empunhadura, que abre para fora e para baixo
            p.AddBezier(P(82.5f, 16.5f), P(89.5f, 19.5f), P(94f, 32f), P(96.5f, 50f));
            p.AddBezier(P(96.5f, 50f), P(99f, 68f), P(97.5f, 82f), P(90.5f, 86.5f));
            p.AddBezier(P(90.5f, 86.5f), P(84.5f, 90f), P(78f, 86.5f), P(72.5f, 79f));

            // borda interna da empunhadura subindo ate a base do corpo
            p.AddBezier(P(72.5f, 79f), P(69.5f, 74.5f), P(68f, 72f), P(65f, 71f));
            p.AddLine(P(65f, 71f), P(35f, 71f));

            // espelho do lado esquerdo
            p.AddBezier(P(35f, 71f), P(32f, 72f), P(30.5f, 74.5f), P(27.5f, 79f));
            p.AddBezier(P(27.5f, 79f), P(22f, 86.5f), P(15.5f, 90f), P(9.5f, 86.5f));
            p.AddBezier(P(9.5f, 86.5f), P(2.5f, 82f), P(1f, 68f), P(3.5f, 50f));
            p.AddBezier(P(3.5f, 50f), P(6f, 32f), P(10.5f, 19.5f), P(17.5f, 16.5f));

            p.CloseFigure();
            return p;
        }

        /// <summary>Botao central, a lingueta arredondada que desce do topo.</summary>
        internal static GraphicsPath Guide(float s)
        {
            float f = s / 100f;
            var p = new GraphicsPath();
            p.AddArc(45.2f * f, 15.5f * f, 9.6f * f, 9.6f * f, 0, 180);
            p.CloseFigure();
            return p;
        }

        internal static GraphicsPath LeftStick(float s)
        {
            float f = s / 100f;
            var p = new GraphicsPath();
            p.AddEllipse(17.5f * f, 25.5f * f, 19f * f, 19f * f);
            return p;
        }

        internal static GraphicsPath RightStick(float s)
        {
            float f = s / 100f;
            var p = new GraphicsPath();
            p.AddEllipse(53f * f, 46f * f, 19f * f, 19f * f);
            return p;
        }

        /// <summary>Direcional em cruz, com os cantos suavizados.</summary>
        internal static GraphicsPath DPad(float s)
        {
            float f = s / 100f;
            var p = new GraphicsPath { FillMode = FillMode.Winding };
            AddRounded(p, 28.5f * f, 52.5f * f, 18f * f, 7f * f, 1.6f * f);
            AddRounded(p, 34f * f, 47f * f, 7f * f, 18f * f, 1.6f * f);
            return p;
        }

        /// <summary>Os quatro botoes de acao, em losango.</summary>
        internal static GraphicsPath FaceButtons(float s)
        {
            float f = s / 100f;
            float cx = 68f, cy = 34f, d = 8.2f, r = 3.4f;
            var p = new GraphicsPath();
            void Dot(float x, float y) => p.AddEllipse((x - r) * f, (y - r) * f, r * 2 * f, r * 2 * f);
            Dot(cx, cy - d);
            Dot(cx - d, cy);
            Dot(cx + d, cy);
            Dot(cx, cy + d);
            return p;
        }

        private static void AddRounded(GraphicsPath p, float x, float y, float w, float h, float r)
        {
            float d = Math.Min(r * 2, Math.Min(w, h));
            if (d <= 0) { p.AddRectangle(new D.RectangleF(x, y, w, h)); return; }
            p.AddArc(x, y, d, d, 180, 90);
            p.AddArc(x + w - d, y, d, d, 270, 90);
            p.AddArc(x + w - d, y + h - d, d, d, 0, 90);
            p.AddArc(x, y + h - d, d, d, 90, 90);
            p.CloseFigure();
        }

        /// <summary>Desenho completo em contorno, para o logo e tamanhos grandes.</summary>
        internal static void DrawOutline(D.Graphics g, float s, D.Color color)
        {
            float stroke = Math.Max(1f, s * 0.032f);
            using var pen = new D.Pen(color, stroke)
            {
                LineJoin = LineJoin.Round,
                StartCap = LineCap.Round,
                EndCap = LineCap.Round
            };
            using var fill = new D.SolidBrush(color);

            using (var body = Body(s)) g.DrawPath(pen, body);
            using (var guide = Guide(s)) g.DrawPath(pen, guide);
            using (var ls = LeftStick(s)) g.DrawPath(pen, ls);
            using (var rs = RightStick(s)) g.DrawPath(pen, rs);
            using (var dp = DPad(s)) g.DrawPath(pen, dp);
            using (var fb = FaceButtons(s)) g.FillPath(fill, fb);
        }

        /// <summary>
        /// Silhueta cheia com os detalhes vazados. E o que sobrevive na bandeja:
        /// a massa continua legivel mesmo quando o detalhe some.
        /// </summary>
        internal static void DrawSolid(D.Graphics g, float s, D.Color color, D.Color hollow)
        {
            using var fill = new D.SolidBrush(color);
            using (var body = Body(s)) g.FillPath(fill, body);

            if (s < 24) return;   // abaixo disso o detalhe vira sujeira

            using var cut = new D.SolidBrush(hollow);
            using (var ls = LeftStick(s)) g.FillPath(cut, ls);
            using (var rs = RightStick(s)) g.FillPath(cut, rs);
            using (var dp = DPad(s)) g.FillPath(cut, dp);
            using (var fb = FaceButtons(s)) g.FillPath(cut, fb);
        }
    }
}
