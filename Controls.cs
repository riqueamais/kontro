using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Animation;

namespace Kontro
{
    /// <summary>Anel de carga desenhado a mao, com cantos arredondados e animavel via Value.</summary>
    public sealed class RingControl : FrameworkElement
    {
        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register(
            nameof(Value), typeof(double), typeof(RingControl),
            new FrameworkPropertyMetadata(0.0, FrameworkPropertyMetadataOptions.AffectsRender));

        public static readonly DependencyProperty RingBrushProperty = DependencyProperty.Register(
            nameof(RingBrush), typeof(Brush), typeof(RingControl),
            new FrameworkPropertyMetadata(Brushes.LimeGreen, FrameworkPropertyMetadataOptions.AffectsRender,
                (d, _) => ((RingControl)d).RepintarGiro()));

        public static readonly DependencyProperty TrackBrushProperty = DependencyProperty.Register(
            nameof(TrackBrush), typeof(Brush), typeof(RingControl),
            new FrameworkPropertyMetadata(
                new SolidColorBrush(Color.FromArgb(38, 255, 255, 255)),
                FrameworkPropertyMetadataOptions.AffectsRender));

        public static readonly DependencyProperty StrokeThicknessProperty = DependencyProperty.Register(
            nameof(StrokeThickness), typeof(double), typeof(RingControl),
            new FrameworkPropertyMetadata(12.0, FrameworkPropertyMetadataOptions.AffectsRender));

        /// <summary>
        /// Giro continuo no lugar da carga.
        ///
        /// No cabo o controle troca de protocolo e para de informar percentual. Deixar o
        /// anel vazio parecia defeito; o giro diz o que de fato se sabe -- esta ligado na
        /// energia -- sem inventar um numero que ninguem mediu.
        /// </summary>
        public static readonly DependencyProperty CarregandoProperty = DependencyProperty.Register(
            nameof(Carregando), typeof(bool), typeof(RingControl),
            new FrameworkPropertyMetadata(false, FrameworkPropertyMetadataOptions.AffectsRender,
                (d, _) => ((RingControl)d).AtualizarGiro()));

        /// <summary>Quanto do circulo o arco que gira ocupa.</summary>
        private const double ArcoDoGiro = 96.0;

        public bool Carregando { get => (bool)GetValue(CarregandoProperty); set => SetValue(CarregandoProperty, value); }

        // O arco que gira vive num visual proprio, e nao no OnRender.
        //
        // A diferenca nao e de estilo: animar uma propriedade marcada AffectsRender
        // obriga o WPF a refazer o desenho inteiro a cada quadro, para sempre. Medido,
        // isso custava um quinto de um nucleo o tempo todo. Rodando um visual filho por
        // transformacao, a composicao acontece fora da thread de interface e o custo
        // por quadro praticamente some.
        private readonly DrawingVisual _giro = new();
        private readonly RotateTransform _rotacao = new(0);
        private bool _giroDesenhado;

        protected override int VisualChildrenCount => 1;
        protected override Visual GetVisualChild(int index) => _giro;

        public RingControl()
        {
            _giro.Transform = _rotacao;
            AddVisualChild(_giro);

            // Animacao sem fim custa para sempre. Um app que vive na bandeja nao pode
            // ficar girando um anel que ninguem esta vendo.
            IsVisibleChanged += (_, _) => AtualizarGiro();
        }

        /// <summary>A cor mudou: o arco ja desenhado precisa ser refeito com ela.</summary>
        private void RepintarGiro()
        {
            if (Carregando && IsVisible) DesenharGiro();
        }

        private void AtualizarGiro()
        {
            bool girar = Carregando && IsVisible;
            _giro.Opacity = girar ? 1 : 0;

            if (!girar)
            {
                _rotacao.BeginAnimation(RotateTransform.AngleProperty, null);
                _rotacao.Angle = 0;
                return;
            }

            DesenharGiro();

            // A sobreposicao usa transparencia, e janela transparente no WPF nao tem
            // composicao por hardware: cada quadro e redesenhado por software na thread
            // de interface. A sessenta quadros por segundo isso custa caro para sempre.
            // Vinte bastam para o giro parecer continuo, e o giro mais lento disfarca a
            // diferenca -- sao poucos graus por quadro de qualquer jeito.
            var giro = new DoubleAnimation(0, 360, TimeSpan.FromSeconds(2.2))
            {
                RepeatBehavior = RepeatBehavior.Forever
            };
            Timeline.SetDesiredFrameRate(giro, 20);
            _rotacao.BeginAnimation(RotateTransform.AngleProperty, giro);
        }

        /// <summary>Redesenha o arco do giro. So precisa acontecer quando algo muda.</summary>
        private void DesenharGiro()
        {
            double lado = Math.Min(ActualWidth, ActualHeight);
            if (lado <= 0) { _giroDesenhado = false; return; }

            double th = StrokeThickness;
            double r = (lado - th) / 2.0;
            if (r <= 0) { _giroDesenhado = false; return; }

            var centro = new Point(ActualWidth / 2.0, ActualHeight / 2.0);
            _rotacao.CenterX = centro.X;
            _rotacao.CenterY = centro.Y;

            using (var dc = _giro.RenderOpen())
            {
                var pen = new Pen(RingBrush, th)
                {
                    StartLineCap = PenLineCap.Round,
                    EndLineCap = PenLineCap.Round
                };
                pen.Freeze();
                DesenharArco(dc, pen, centro, r, -90, ArcoDoGiro);
            }
            _giroDesenhado = true;
        }

        public double Value { get => (double)GetValue(ValueProperty); set => SetValue(ValueProperty, value); }
        public Brush RingBrush { get => (Brush)GetValue(RingBrushProperty); set => SetValue(RingBrushProperty, value); }
        public Brush TrackBrush { get => (Brush)GetValue(TrackBrushProperty); set => SetValue(TrackBrushProperty, value); }
        public double StrokeThickness { get => (double)GetValue(StrokeThicknessProperty); set => SetValue(StrokeThicknessProperty, value); }

        private static Point PointOn(Point c, double r, double angleDeg)
        {
            double a = angleDeg * Math.PI / 180.0;
            return new Point(c.X + r * Math.Cos(a), c.Y + r * Math.Sin(a));
        }

        protected override void OnRenderSizeChanged(SizeChangedInfo info)
        {
            base.OnRenderSizeChanged(info);
            if (Carregando) DesenharGiro();
        }

        protected override void OnRender(DrawingContext dc)
        {
            double side = Math.Min(ActualWidth, ActualHeight);
            if (side <= 0) return;

            double th = StrokeThickness;
            double r = (side - th) / 2.0;
            if (r <= 0) return;
            var center = new Point(ActualWidth / 2.0, ActualHeight / 2.0);

            var trackPen = new Pen(TrackBrush, th) { StartLineCap = PenLineCap.Round, EndLineCap = PenLineCap.Round };
            dc.DrawEllipse(null, trackPen, center, r, r);

            if (Carregando)
            {
                // o arco fica no visual filho; aqui so o trilho e desenhado
                if (!_giroDesenhado) DesenharGiro();
                return;
            }

            double v = Math.Clamp(Value, 0, 100);
            if (v <= 0.01) return;

            if (v >= 99.99)
            {
                var cheio = new Pen(RingBrush, th) { StartLineCap = PenLineCap.Round, EndLineCap = PenLineCap.Round };
                dc.DrawEllipse(null, cheio, center, r, r);
                return;
            }

            DesenharArco(dc, center, r, th, -90, 360.0 * v / 100.0);
        }

        private void DesenharArco(DrawingContext dc, Point center, double r, double th,
                                  double inicio, double varredura)
        {
            var pen = new Pen(RingBrush, th) { StartLineCap = PenLineCap.Round, EndLineCap = PenLineCap.Round };
            pen.Freeze();
            DesenharArco(dc, pen, center, r, inicio, varredura);
        }

        private static void DesenharArco(DrawingContext dc, Pen pen, Point center, double r,
                                         double inicio, double varredura)
        {
            var start = PointOn(center, r, inicio);
            var end = PointOn(center, r, inicio + varredura);

            var geo = new StreamGeometry();
            using (var ctx = geo.Open())
            {
                ctx.BeginFigure(start, false, false);
                ctx.ArcTo(end, new Size(r, r), 0, varredura > 180, SweepDirection.Clockwise, true, false);
            }
            geo.Freeze();
            dc.DrawGeometry(null, pen, geo);
        }
    }

    /// <summary>Mini grafico do historico recente, com area preenchida em degrade.</summary>
    public sealed class Sparkline : FrameworkElement
    {
        public static readonly DependencyProperty PointsProperty = DependencyProperty.Register(
            nameof(Points), typeof(IReadOnlyList<Sample>), typeof(Sparkline),
            new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

        public static readonly DependencyProperty StrokeProperty = DependencyProperty.Register(
            nameof(Stroke), typeof(Brush), typeof(Sparkline),
            new FrameworkPropertyMetadata(Brushes.LimeGreen, FrameworkPropertyMetadataOptions.AffectsRender));

        public IReadOnlyList<Sample> Points
        {
            get => (IReadOnlyList<Sample>)GetValue(PointsProperty);
            set => SetValue(PointsProperty, value);
        }

        public Brush Stroke { get => (Brush)GetValue(StrokeProperty); set => SetValue(StrokeProperty, value); }

        protected override void OnRender(DrawingContext dc)
        {
            var data = Points;
            double w = ActualWidth, h = ActualHeight;
            if (w <= 0 || h <= 0) return;

            if (data == null || data.Count < 2) return;

            double tMin = data[0].T.ToOADate();
            double tMax = data[^1].T.ToOADate();
            double tSpan = tMax - tMin;
            if (tSpan <= 0) return;

            int pMin = data.Min(s => s.P);
            int pMax = data.Max(s => s.P);
            // margem para a curva nao encostar nas bordas, e um piso de amplitude
            // para variacoes minusculas nao virarem uma serra gigante
            int range = Math.Max(pMax - pMin, 8);
            double mid = (pMax + pMin) / 2.0;
            double lo = mid - range / 2.0;

            double pad = 6;
            Point Map(Sample s) => new(
                (s.T.ToOADate() - tMin) / tSpan * w,
                h - pad - ((s.P - lo) / range) * (h - pad * 2));

            var line = new StreamGeometry();

            using (var lc = line.Open())
            {
                lc.BeginFigure(Map(data[0]), false, false);
                for (int i = 1; i < data.Count; i++) lc.LineTo(Map(data[i]), true, true);
            }
            line.Freeze();

            // o design pede linha de 1px, sem area preenchida, sem eixo e sem grade:
            // o historico e contexto, nao protagonista
            dc.DrawGeometry(null, new Pen(Stroke, 1)
            {
                LineJoin = PenLineJoin.Round,
                StartLineCap = PenLineCap.Round,
                EndLineCap = PenLineCap.Round
            }, line);
        }
    }
}



