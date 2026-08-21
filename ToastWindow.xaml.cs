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

        // folgas da janela que acomodam a sombra, iguais as margens do XAML
        private const double FolgaLateral = 20;
        private const double FolgaSuperior = 24;

        private readonly DispatcherTimer _permanencia;
        private bool _saindo;

        /// <summary>Comecou sem carga para mostrar e ainda espera a primeira leitura.</summary>
        private bool _esperandoCarga;

        public ToastWindow()
        {
            InitializeComponent();

            SourceInitialized += (_, _) =>
                Native.MakeClickThrough(new WindowInteropHelper(this).Handle);

            _permanencia = new DispatcherTimer { Interval = TimeSpan.FromSeconds(4) };
            _permanencia.Tick += (_, _) => Esconder();
        }

        /// <summary>
        /// Acompanha o estado enquanto o aviso esta na tela.
        ///
        /// O aviso nasce no instante em que o controle conecta, e nesse instante a carga
        /// ainda e a ultima que se conhecia. A leitura de verdade chega segundos depois:
        /// sem seguir o estado, o aviso ficaria os quatro segundos exibindo um numero
        /// velho enquanto o resto do app ja mostra o certo.
        /// </summary>
        public void Atualizar(BatteryState s)
        {
            if (!IsVisible || _saindo) return;

            bool chegou = _esperandoCarga && !s.Stale && s.Preenchimento.HasValue;
            Aplicar(s);

            // A carga chegando no ultimo segundo apareceria e sumiria junto. Reiniciar a
            // permanencia da ao numero o mesmo tempo de leitura que ele teria se
            // estivesse pronto desde o inicio.
            if (!chegou) return;
            _esperandoCarga = false;
            _permanencia.Stop();
            _permanencia.Start();
        }

        public void Mostrar(BatteryState s)
        {
            _esperandoCarga = s.Stale || !s.Preenchimento.HasValue;
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
            else if (s.Stale)
            {
                // numero velho nao entra: no instante da conexao a unica carga conhecida
                // e a da sessao passada, e mostra-la seria contradizer o app inteiro
                // alguns segundos depois, quando a leitura real chegar
                PercentText.Visibility = Visibility.Collapsed;
                StateText.Text = "conectado · " + s.TextoDaLigacao + " · lendo a carga";
                Ring.Value = 0;
            }
            else
            {
                // so o percentual cabe no miolo do anel; degrau e ausencia de leitura
                // viram texto embaixo, onde ha espaco para uma frase
                PercentText.Visibility = s.TemNumero ? Visibility.Visible : Visibility.Collapsed;
                PercentText.Text = s.TextoDaCarga;
                StateText.Text =
                    s.TemNumero ? "conectado · " + s.TextoDaLigacao
                    : s.Preenchimento.HasValue ? s.TextoDaCarga + " · " + s.TextoDaLigacao
                    : "conectado · " + s.TextoDaLigacao + " · sem leitura de bateria";
                Ring.Value = s.Preenchimento ?? 0;
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

            // A borda visivel do painel nao coincide com a da janela: a folga da sombra
            // fica entre as duas. Ancorar pela janela deixaria o aviso mais baixo e
            // fora do centro por todo o tamanho dessa folga.
            double larguraVisivel = Width - FolgaLateral * 2;
            Left = esquerda + (largura - larguraVisivel) / 2 - FolgaLateral;
            Top = topo + MargemDoTopo - FolgaSuperior;
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
            if (s.Mode == LinkMode.Cable || s.Stale || !s.Preenchimento.HasValue)
                return ControllerGeometry.Gray;
            if (s.Preenchimento < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (s.Preenchimento < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }
    }
}
