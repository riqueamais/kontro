using System;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Animation;
using System.Windows.Threading;
using WF = System.Windows.Forms;

namespace Kontro
{
    /// <summary>
    /// Aviso passageiro no topo da tela quando o controle conecta.
    ///
    /// E puramente visual: nao recebe clique, nunca rouba o foco e fica fora do Alt+Tab.
    /// Um aviso que tira o foco no meio de uma partida seria pior que nao avisar nada.
    /// </summary>
    public partial class ToastWindow : Window
    {
        private const int AmbarAbaixoDe = 60;
        private const int VermelhoAbaixoDe = 30;

        /// <summary>Distancia do topo da area util, seguindo a escala de espaco do design.</summary>
        private const double MargemDoTopo = 24;

        private readonly DispatcherTimer _permanencia;
        private bool _saindo;

        public ToastWindow()
        {
            InitializeComponent();

            SourceInitialized += (_, _) =>
                Native.MakeClickThrough(new WindowInteropHelper(this).Handle);

            _permanencia = new DispatcherTimer { Interval = TimeSpan.FromSeconds(4) };
            _permanencia.Tick += (_, _) => Esconder();
        }

        public void Mostrar(BatteryState s)
        {
            Aplicar(s);
            Posicionar();

            _permanencia.Stop();
            _saindo = false;

            Opacity = 0;
            Deslize.Y = -12;
            Show();

            var suave = new CubicEase { EasingMode = EasingMode.EaseOut };
            BeginAnimation(OpacityProperty,
                new DoubleAnimation(1, TimeSpan.FromMilliseconds(240)) { EasingFunction = suave });
            Deslize.BeginAnimation(TranslateTransform.YProperty,
                new DoubleAnimation(0, TimeSpan.FromMilliseconds(240)) { EasingFunction = suave });

            _permanencia.Start();
        }

        private void Aplicar(BatteryState s)
        {
            string nome = string.IsNullOrWhiteSpace(s.DeviceName) ? "Controle" : s.DeviceName;
            DeviceText.Text = nome;

            var cor = CorDoAnel(s);
            var pincel = new SolidColorBrush(cor);
            pincel.Freeze();
            Ring.RingBrush = pincel;

            if (s.Mode == LinkMode.Cable)
            {
                PercentText.Visibility = Visibility.Collapsed;
                StateText.Text = s.Charging ? "conectado · carregando" : "conectado pelo cabo";
                Ring.Value = 100;
            }
            else
            {
                PercentText.Visibility = Visibility.Visible;
                PercentText.Text = s.Percent.HasValue ? s.Percent.Value + "%" : "--";
                StateText.Text = s.Percent.HasValue
                    ? "conectado · Bluetooth"
                    : "conectado · lendo a carga";
                Ring.Value = s.Percent ?? 0;
            }
        }

        /// <summary>
        /// Centraliza no topo do monitor onde o usuario esta olhando, e nao sempre no
        /// principal: com dois monitores, avisar do lado errado nao avisa nada.
        /// </summary>
        private void Posicionar()
        {
            var tela = WF.Screen.PrimaryScreen;
            try
            {
                var foco = Native.GetForegroundWindow();
                if (foco != IntPtr.Zero) tela = WF.Screen.FromHandle(foco);
            }
            catch { }

            var area = tela.WorkingArea;
            var origem = PresentationSource.FromVisual(this)?.CompositionTarget;
            double escalaX = origem?.TransformToDevice.M11 ?? 1;
            double escalaY = origem?.TransformToDevice.M22 ?? 1;

            // WorkingArea vem em pixels do dispositivo; a janela se posiciona em
            // unidades independentes de DPI
            double largura = area.Width / escalaX;
            double esquerda = area.Left / escalaX;
            double topo = area.Top / escalaY;

            Left = esquerda + (largura - Width) / 2;
            Top = topo + MargemDoTopo - 16;   // 16 e a folga da sombra
        }

        private void Esconder()
        {
            if (_saindo) return;
            _saindo = true;
            _permanencia.Stop();

            var saida = new DoubleAnimation(0, TimeSpan.FromMilliseconds(240))
            {
                EasingFunction = new CubicEase { EasingMode = EasingMode.EaseIn }
            };
            saida.Completed += (_, _) => { Hide(); _saindo = false; };
            BeginAnimation(OpacityProperty, saida);
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
