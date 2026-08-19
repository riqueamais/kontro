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

        /// <summary>Folga entre o painel e o canto da bandeja.</summary>
        private const double Folga = 12;

        /// <summary>Faixas de cor do anel, as mesmas que estao gravadas nos icones.</summary>
        private const int AmbarAbaixoDe = 60;
        private const int VermelhoAbaixoDe = 30;

        private readonly History _history;
        private BatteryState _state;

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
            Deactivated += (_, _) => { if (!Pinned) Hide(); };
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
                case LinkMode.Cable:
                    // no cabo nao existe percentual confiavel: melhor nada que errado
                    PercentText.Visibility = Visibility.Collapsed;
                    DeviceText.Text = s.Charging ? "Carregando" : "No cabo";
                    EstimateText.Text = nome;
                    break;

                case LinkMode.Offline:
                    PercentText.Visibility = Visibility.Visible;
                    PercentText.Text = s.Percent.HasValue ? s.Percent.Value + "%" : "--";
                    DeviceText.Text = "Desconectado";
                    EstimateText.Text = nome;
                    break;

                default:
                    PercentText.Visibility = Visibility.Visible;
                    PercentText.Text = s.Percent.HasValue ? s.Percent.Value + "%" : "--";
                    DeviceText.Text = nome;
                    EstimateText.Text = Autonomia(s);
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
            if (s.Mode == LinkMode.Cable) return ControllerGeometry.Gray;
            if (s.Mode == LinkMode.Offline || !s.Percent.HasValue) return ControllerGeometry.Gray;
            if (s.Percent < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (s.Percent < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }

        /// <summary>O anel nunca salta entre percentuais: a troca leva 180ms.</summary>
        private void AnimarAnel(BatteryState s)
        {
            // no cabo o anel fica cheio e cinza, dizendo "estou ligado, sem numero"
            double alvo = s.Mode == LinkMode.Cable ? 100 : (s.Percent ?? 0);
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
            // a borda visivel comeca 16px dentro da janela, por causa da sombra
            Left = area.Right - Width + 16 - Folga;
            Top = area.Bottom - Height + 16 - Folga;

            Show();
            Activate();

            Opacity = 0;
            BeginAnimation(OpacityProperty,
                new DoubleAnimation(1, TimeSpan.FromMilliseconds(240))
                {
                    EasingFunction = new CubicEase { EasingMode = EasingMode.EaseOut }
                });
        }

        public void Toggle()
        {
            if (IsVisible) Hide();
            else ShowNearTray();
        }
    }
}
