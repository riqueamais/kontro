using System;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Kontro
{
    /// <summary>
    /// Geometria e cores do sistema "Anel". Os PNGs em Assets/tray sairam desta mesma
    /// especificacao: mexer aqui muda o runtime, nao os arquivos.
    /// Espaco de desenho canonico: 512 x 512.
    ///
    /// Duas escolhas seguem o DESIGN.md onde ele diverge do exemplo que veio junto:
    /// os sticks sao vazados na bandeja (recorte, nao pintura) e o estado desconectado
    /// nao desenha anel nenhum.
    /// </summary>
    public static class ControllerGeometry
    {
        public const string PadPath =
            "M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 " +
            "366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 " +
            "372C146 400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z";

        public static readonly Color Green = FromHex("#5FE083");
        public static readonly Color Teal = FromHex("#35D7A8");
        public static readonly Color Amber = FromHex("#F2C14E");
        public static readonly Color Red = FromHex("#F2564E");
        public static readonly Color Gray = FromHex("#8D979F");
        public static readonly Color Shell = FromHex("#0F1318");
        public static readonly Color GlyphOnLight = FromHex("#1B1F24");

        private static Color FromHex(string s) => (Color)ColorConverter.ConvertFromString(s);

        public enum TrayState { Level, Cable, Disconnected }

        // bandeja: anel grosso, sem fundo. Precisa existir em 16px.
        private const double TrayRingRadius = 194;
        private const double TrayRingWidth = 56;
        private const double TrayPadScale = 0.5;
        private const double TrayPadCenterY = 268;

        // sticks no espaco do proprio path, para acompanharem qualquer escala
        private const double StickRadius = 48.3;
        private static readonly Point StickLeft = new(129.3, 228);
        private static readonly Point StickRight = new(382.7, 228);

        /// <summary>Cor do anel para um percentual, pelos limiares que o usuario configurou.</summary>
        public static Color LevelColor(int percent, int redBelow, int amberBelow)
        {
            if (percent < redBelow) return Red;
            if (percent < amberBelow) return Amber;
            return Green;
        }

        /// <summary>Silhueta do controle com os sticks recortados, nao pintados.</summary>
        public static Geometry PadWithHollowSticks()
        {
            var pad = Geometry.Parse(PadPath);
            var sticks = new GeometryGroup();
            sticks.Children.Add(new EllipseGeometry(StickLeft, StickRadius, StickRadius));
            sticks.Children.Add(new EllipseGeometry(StickRight, StickRadius, StickRadius));
            var recortado = new CombinedGeometry(GeometryCombineMode.Exclude, pad, sticks);
            recortado.Freeze();
            return recortado;
        }

        /// <summary>
        /// Desenha o icone da bandeja. O tamanho deve ser o real em pixels
        /// (16, 20, 24 ou 32, conforme SM_CXSMICON) — nunca escale o resultado depois.
        /// </summary>
        public static BitmapSource RenderTray(int percent, TrayState state, int size,
                                              bool lightTaskbar, int redBelow, int amberBelow)
        {
            var fg = lightTaskbar ? GlyphOnLight : Colors.White;

            var visual = new DrawingVisual();
            using (var dc = visual.RenderOpen())
            {
                double s = size / 512.0;
                dc.PushTransform(new ScaleTransform(s, s));
                var centro = new Point(256, 256);

                if (state == TrayState.Cable)
                {
                    // anel inteiro cinza, sem arco: plugado e sem numero confiavel
                    var cheio = Gray; cheio.A = 230;
                    dc.DrawEllipse(null, new Pen(new SolidColorBrush(cheio), TrayRingWidth),
                                   centro, TrayRingRadius, TrayRingRadius);
                }
                else if (state == TrayState.Level)
                {
                    var trilha = fg; trilha.A = (byte)(0.22 * 255);
                    dc.DrawEllipse(null, new Pen(new SolidColorBrush(trilha), TrayRingWidth),
                                   centro, TrayRingRadius, TrayRingRadius);
                    DrawArc(dc, centro, TrayRingRadius, TrayRingWidth,
                            Math.Clamp(percent, 0, 100) / 100.0,
                            LevelColor(percent, redBelow, amberBelow));
                }
                // desconectado nao tem anel: so o controle apagado e a barra

                double opacidade = state == TrayState.Disconnected ? 0.45 : 1.0;
                dc.PushOpacity(opacidade);
                dc.PushTransform(new TranslateTransform(256, TrayPadCenterY));
                dc.PushTransform(new ScaleTransform(TrayPadScale, TrayPadScale));
                dc.PushTransform(new TranslateTransform(-256, -288));
                dc.DrawGeometry(new SolidColorBrush(fg), null, PadWithHollowSticks());
                dc.Pop(); dc.Pop(); dc.Pop(); dc.Pop();

                if (state == TrayState.Disconnected)
                {
                    var barra = new Pen(new SolidColorBrush(fg), 46)
                    {
                        StartLineCap = PenLineCap.Round,
                        EndLineCap = PenLineCap.Round
                    };
                    dc.DrawLine(barra, new Point(120, 392), new Point(392, 120));
                }

                dc.Pop();
            }

            var bmp = new RenderTargetBitmap(size, size, 96, 96, PixelFormats.Pbgra32);
            bmp.Render(visual);
            bmp.Freeze();
            return bmp;
        }

        private static void DrawArc(DrawingContext dc, Point c, double r, double width,
                                    double fracao, Color cor)
        {
            if (fracao <= 0) return;

            var caneta = new Pen(new SolidColorBrush(cor), width)
            {
                StartLineCap = PenLineCap.Round,
                EndLineCap = PenLineCap.Round
            };

            if (fracao >= 1)
            {
                dc.DrawEllipse(null, caneta, c, r, r);
                return;
            }

            // comeca as 12 horas e corre no sentido horario
            double a0 = -Math.PI / 2;
            double a1 = a0 + fracao * 2 * Math.PI;
            var p0 = new Point(c.X + r * Math.Cos(a0), c.Y + r * Math.Sin(a0));
            var p1 = new Point(c.X + r * Math.Cos(a1), c.Y + r * Math.Sin(a1));

            var figura = new PathFigure { StartPoint = p0, IsClosed = false };
            figura.Segments.Add(new ArcSegment(p1, new Size(r, r), 0,
                fracao > 0.5, SweepDirection.Clockwise, true));
            var geo = new PathGeometry();
            geo.Figures.Add(figura);
            geo.Freeze();

            dc.DrawGeometry(null, caneta, geo);
        }
    }
}
