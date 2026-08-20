using System;
using System.Globalization;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Animation;

namespace Kontro
{
    public partial class FlyoutWindow : Window
    {
        private static readonly CultureInfo Br = CultureInfo.GetCultureInfo("pt-BR");

        /// <summary>Respiro entre o painel e o canto da tela.</summary>
        private const double Respiro = 12;

        // folgas da janela que acomodam a sombra, iguais as margens do XAML
        private const double FolgaDireita = 20;
        private const double FolgaInferior = 44;

        /// <summary>Faixas de cor do anel, as mesmas que estao gravadas nos icones.</summary>
        private const int AmbarAbaixoDe = 60;
        private const int VermelhoAbaixoDe = 30;

        private readonly History _history;
        private BatteryState _state;
        private bool _saindo;

        public event Action SettingsRequested;
        public event Action RefreshRequested;

        /// <summary>Impede o auto-ocultar ao perder foco. Usado so para inspecionar a UI.</summary>
        public bool Pinned { get; set; }

        public FlyoutWindow(History history)
        {
            InitializeComponent();
            _history = history;

            SettingsButton.Click += (_, _) => { Hide(); SettingsRequested?.Invoke(); };
            RefreshButton.Click += (_, _) => RefreshRequested?.Invoke();
            Deactivated += (_, _) => { if (!Pinned) Esconder(); };
        }

        // ---------- apresentacao ----------

        public void Apply(BatteryState s)
        {
            _state = s;

            var cor = CorDoAnel(s);
            var pincel = new SolidColorBrush(cor);
            pincel.Freeze();
            Ring.RingBrush = pincel;

            AplicarLeitura(s);
            AnimarAnel(s);
        }

        private void AplicarLeitura(BatteryState s)
        {
            string nome = string.IsNullOrWhiteSpace(s.DeviceName) ? "Controle" : s.DeviceName;

            switch (s.Mode)
            {
                case LinkMode.Cable when s.Preenchimento == null:
                    // no cabo, sem via de leitura, nao existe percentual confiavel
                    PercentText.Visibility = Visibility.Collapsed;
                    DeviceText.Text = s.Charging ? "Carregando" : "No cabo";
                    EstimateText.Text = nome;
                    break;

                case LinkMode.Offline:
                    PercentText.Visibility = s.TemNumero ? Visibility.Visible : Visibility.Collapsed;
                    PercentText.Text = s.TextoDaCarga;
                    DeviceText.Text = "Desconectado";
                    EstimateText.Text = nome;
                    break;

                default:
                    // com leitura aproximada o texto vai embaixo, nao no miolo do anel:
                    // "carga cheia" nao cabe ali e nao e um numero
                    PercentText.Visibility = s.TemNumero ? Visibility.Visible : Visibility.Collapsed;
                    PercentText.Text = s.TextoDaCarga;
                    DeviceText.Text = nome;
                    EstimateText.Text = s.Precisao == Precisao.Aproximada
                        ? s.TextoDaCarga + " · sem percentual neste controle"
                        : Autonomia(s);
                    break;
            }

            UpdatedText.Text = UltimaLeitura(s);
        }

        private string Autonomia(BatteryState s)
        {
            if (!s.Percent.HasValue) return "sem leitura";
            var restante = _history.EstimateRemaining(s.Key, s.Percent.Value);
            if (restante.HasValue) return "~" + Formatar(restante.Value) + " de jogo";

            var consumo = _history.DrainPerHour(s.Key);
            return consumo.HasValue
                ? string.Format(Br, "consumo de {0:0.0} %/h", consumo.Value)
                : "medindo o consumo";
        }

        private static string UltimaLeitura(BatteryState s)
        {
            if (!s.ReadAt.HasValue) return "sem leitura ainda";
            var quando = s.ReadAt.Value;
            var idade = DateTime.Now - quando;

            string texto = idade < TimeSpan.FromMinutes(1)
                ? "agora"
                : "às " + quando.ToString("HH:mm", Br);

            return s.Stale ? "lido " + texto : "atualizado " + texto;
        }

        private static string Formatar(TimeSpan t)
        {
            if (t.TotalHours >= 1)
            {
                int h = (int)t.TotalHours, m = t.Minutes;
                return m > 0 ? $"{h} h {m} min" : $"{h} h";
            }
            return $"{Math.Max(1, (int)t.TotalMinutes)} min";
        }

        private static Color CorDoAnel(BatteryState s)
        {
            int? nivel = s.Mode == LinkMode.Offline ? null : s.Preenchimento;
            if (nivel == null) return ControllerGeometry.Gray;
            if (nivel < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (nivel < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }

        /// <summary>O anel nunca salta entre percentuais: a troca leva 180ms.</summary>
        private void AnimarAnel(BatteryState s)
        {
            // no cabo o anel fica cheio e cinza, dizendo "estou ligado, sem numero"
            double alvo = s.Preenchimento ?? (s.Mode == LinkMode.Cable ? 100 : 0);
            Ring.BeginAnimation(RingControl.ValueProperty,
                new DoubleAnimation(alvo, TimeSpan.FromMilliseconds(180))
                {
                    EasingFunction = new CubicEase { EasingMode = EasingMode.EaseOut }
                });
        }

        // ---------- janela ----------

        public void ShowNearTray()
        {
            if (_state != null) Apply(_state);

            var area = SystemParameters.WorkArea;
            // A borda visivel do painel nao coincide com a da janela: as folgas que
            // acomodam a sombra ficam entre as duas. Ancorar pela janela deixaria o
            // painel longe do canto, com um vao do tamanho da folga.
            Left = area.Right - Width + FolgaDireita - Respiro;
            Top = area.Bottom - Height + FolgaInferior - Respiro;

            Show();
            Activate();

            Animar(entrando: true);
        }

        /// <summary>
        /// Sobe deslizando e aparecendo, no tempo de abertura do sistema de design.
        /// Sair e mais rapido que entrar: esperar uma saida lenta incomoda, esperar
        /// uma entrada lenta nao.
        /// </summary>
        private void Animar(bool entrando)
        {
            var duracao = TimeSpan.FromMilliseconds(entrando ? 240 : 120);
            var suave = new CubicEase { EasingMode = entrando ? EasingMode.EaseOut : EasingMode.EaseIn };

            if (entrando)
            {
                Opacity = 0;
                Deslize.Y = 16;
            }

            BeginAnimation(OpacityProperty,
                new DoubleAnimation(entrando ? 1 : 0, duracao) { EasingFunction = suave });
            Deslize.BeginAnimation(TranslateTransform.YProperty,
                new DoubleAnimation(entrando ? 0 : 10, duracao) { EasingFunction = suave });
        }

        private void Esconder()
        {
            if (!IsVisible || _saindo) return;
            _saindo = true;

            var saida = new DoubleAnimation(0, TimeSpan.FromMilliseconds(120))
            {
                EasingFunction = new CubicEase { EasingMode = EasingMode.EaseIn }
            };
            saida.Completed += (_, _) => { Hide(); _saindo = false; };
            Deslize.BeginAnimation(TranslateTransform.YProperty,
                new DoubleAnimation(10, TimeSpan.FromMilliseconds(120)));
            BeginAnimation(OpacityProperty, saida);
        }

        public void Toggle()
        {
            if (IsVisible) Esconder();
            else ShowNearTray();
        }
    }
}
