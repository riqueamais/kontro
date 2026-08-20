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
        private int _monitor = -1;   // -1 acompanha o foco

        public OverlayWindow()
        {
            InitializeComponent();

            // A silhueta cheia vira uma mancha neste tamanho. Com os analogicos
            // recortados -- a mesma geometria do icone da bandeja -- ela volta a
            // se ler como controle.
            Glifo.Data = ControllerGeometry.PadWithHollowSticks();

            SourceInitialized += (_, _) =>
                Native.MakeClickThrough(new WindowInteropHelper(this).Handle);
            // o tamanho muda com a escala, entao a posicao precisa acompanhar
            SizeChanged += (_, _) => Reposicionar();
        }

        public void AplicarPreferencias(Settings s)
        {
            _canto = s.OverlayCorner;
            _monitor = s.OverlayMonitor;
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

            // sem leitura no cabo o anel cheio em cinza ao menos diz "ligado"
            if (s.Mode == LinkMode.Cable && s.Preenchimento == null)
            {
                Ring.Value = 100;
                PercentText.Text = "cabo";
                return;
            }

            Ring.Value = s.Preenchimento ?? 0;
            // texto aproximado nao cabe na largura do numero, entao vira o degrau
            PercentText.Text = s.Precisao == Precisao.Aproximada && s.Nivel.HasValue
                ? new[] { "baixa", "baixa", "média", "cheia" }[Math.Clamp(s.Nivel.Value, 0, 3)]
                : s.TextoDaCarga;
        }

        /// <summary>
        /// Ancorado ao monitor que esta em foco, e nao ao principal: a sobreposicao
        /// tem que aparecer na tela onde o jogo esta.
        /// </summary>
        public void Reposicionar()
        {
            // Sem medida ainda -- janela escondida ou recem-criada -- ancorar pela
            // direita ou por baixo colocaria a pilula quase toda fora da tela. O
            // evento de tamanho chama este metodo de novo assim que houver medida.
            if (ActualWidth <= 0 || ActualHeight <= 0) return;

            var tela = EscolherTela();

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

        /// <summary>
        /// Monitor fixo quando o usuario escolheu um; senao, o que esta em foco.
        /// Indice invalido -- monitor desligado ou desconectado depois da escolha --
        /// cai de volta para o foco, em vez de sumir num lugar que nao existe mais.
        /// </summary>
        private WF.Screen EscolherTela()
        {
            var telas = WF.Screen.AllScreens;
            if (_monitor >= 0 && _monitor < telas.Length) return telas[_monitor];

            try
            {
                var foco = Native.GetForegroundWindow();
                if (foco != IntPtr.Zero) return WF.Screen.FromHandle(foco);
            }
            catch { }
            return WF.Screen.PrimaryScreen;
        }

        private static Color CorDoAnel(BatteryState s)
        {
            int? nivel = s.Preenchimento;
            if (nivel == null) return ControllerGeometry.Gray;
            if (nivel < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (nivel < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }
    }
}
