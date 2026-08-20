using System;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using WF = System.Windows.Forms;

namespace Kontro
{
    /// <summary>
    /// Medidor fixo no canto da tela, no espirito dos contadores de quadros.
    ///
    /// Nao e desenhado sobre jogo em tela cheia exclusiva: ali so entra quem injeta
    /// codigo no processo, o que arriscaria banimento por anticheat. Em tela cheia em
    /// janela, que e o padrao da maioria dos jogos hoje, funciona normalmente.
    /// </summary>
    public partial class OverlayWindow : Window
    {
        private const int AmbarAbaixoDe = 60;
        private const int VermelhoAbaixoDe = 30;

        /// <summary>Folga da borda da tela, na escala de espaco do design.</summary>
        private const double Folga = 16;

        private OverlayCorner _canto = OverlayCorner.SuperiorDireito;

        public OverlayWindow()
        {
            InitializeComponent();
            SourceInitialized += (_, _) =>
                Native.MakeClickThrough(new WindowInteropHelper(this).Handle);
            // o tamanho muda com a escala, entao a posicao precisa acompanhar
            SizeChanged += (_, _) => Reposicionar();
        }

        public void AplicarPreferencias(Settings s)
        {
            _canto = s.OverlayCorner;
            Escala.ScaleX = Escala.ScaleY = s.OverlayScale;
            Painel.Opacity = s.OverlayOpacity;
            Reposicionar();
        }

        public void Aplicar(BatteryState s)
        {
            var cor = CorDoAnel(s);
            var pincel = new SolidColorBrush(cor);
            pincel.Freeze();
            Ring.RingBrush = pincel;

            if (s.Mode == LinkMode.Cable)
            {
                // no cabo nao existe percentual: o anel cheio em cinza diz "ligado"
                Ring.Value = 100;
                PercentText.Text = "cabo";
            }
            else
            {
                Ring.Value = s.Percent ?? 0;
                PercentText.Text = s.Percent.HasValue ? s.Percent.Value + "%" : "--";
            }
        }

        /// <summary>
        /// Ancorado ao monitor que esta em foco, e nao ao principal: a sobreposicao
        /// tem que aparecer na tela onde o jogo esta.
        /// </summary>
        public void Reposicionar()
        {
            var tela = WF.Screen.PrimaryScreen;
            try
            {
                var foco = Native.GetForegroundWindow();
                if (foco != IntPtr.Zero) tela = WF.Screen.FromHandle(foco);
            }
            catch { }

            var origem = PresentationSource.FromVisual(this)?.CompositionTarget;
            double ex = origem?.TransformToDevice.M11 ?? 1;
            double ey = origem?.TransformToDevice.M22 ?? 1;

            var area = tela.WorkingArea;
            double esquerda = area.Left / ex, topo = area.Top / ey;
            double largura = area.Width / ex, altura = area.Height / ey;

            bool aDireita = _canto is OverlayCorner.SuperiorDireito or OverlayCorner.InferiorDireito;
            bool embaixo = _canto is OverlayCorner.InferiorEsquerdo or OverlayCorner.InferiorDireito;

            Left = aDireita ? esquerda + largura - ActualWidth - Folga : esquerda + Folga;
            Top = embaixo ? topo + altura - ActualHeight - Folga : topo + Folga;
        }

        private static Color CorDoAnel(BatteryState s)
        {
            if (s.Mode == LinkMode.Cable || !s.Percent.HasValue) return ControllerGeometry.Gray;
            if (s.Percent < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (s.Percent < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }
    }
}
