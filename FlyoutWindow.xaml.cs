using System;
using System.Globalization;
using System.Linq;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Animation;

namespace Kontro
{
    public partial class FlyoutWindow : Window
    {
        private static readonly CultureInfo Br = CultureInfo.GetCultureInfo("pt-BR");

        private readonly History _history;
        private BatteryState _state;

        public event Action QuitRequested;

        /// <summary>Impede o auto-ocultar ao perder foco. Usado so para inspecionar a UI.</summary>
        public bool Pinned { get; set; }

        public FlyoutWindow(History history)
        {
            InitializeComponent();
            _history = history;

            QuitButton.Click += (_, _) => QuitRequested?.Invoke();
            Deactivated += (_, _) => { if (!Pinned) Hide(); };
        }

        // ---------- apresentacao ----------

        public void Apply(BatteryState s)
        {
            _state = s;

            var color = AccentFor(s);
            var brush = new SolidColorBrush(color);
            brush.Freeze();

            TitleText.Text = string.IsNullOrWhiteSpace(s.DeviceName) ? "Controle" : s.DeviceName;
            SubtitleText.Text = BuildSubtitle(s);

            if (s.Percent.HasValue)
            {
                PercentText.Text = s.Percent.Value.ToString(CultureInfo.InvariantCulture);
                PercentSign.Visibility = Visibility.Visible;
            }
            else
            {
                PercentText.Text = "--";
                PercentSign.Visibility = Visibility.Collapsed;
            }

            double target = s.Percent ?? 0;
            Ring.RingBrush = brush;
            Ring.BeginAnimation(RingControl.ValueProperty,
                new DoubleAnimation(target, TimeSpan.FromMilliseconds(650))
                {
                    EasingFunction = new QuadraticEase { EasingMode = EasingMode.EaseOut }
                });

            Glow.Fill = new RadialGradientBrush(
                Color.FromArgb(60, color.R, color.G, color.B),
                Color.FromArgb(0, color.R, color.G, color.B));
            Glow.Opacity = s.Mode == LinkMode.Offline ? 0.18 : 0.55;

            ApplyModePill(s);
            ApplyEstimate(s);
            ApplySpark(s, brush);
            ApplyUpdated(s);
        }

        private static string BuildSubtitle(BatteryState s)
        {
            if (!string.IsNullOrEmpty(s.Address))
            {
                return s.KnownCount > 1
                    ? $"{s.Address} · {s.KnownCount} controles conhecidos"
                    : s.Address;
            }
            if (s.Mode == LinkMode.Cable) return "conectado por cabo";
            return s.KnownCount == 0 ? "nenhum controle pareado ainda" : "";
        }

        private void ApplyModePill(BatteryState s)
        {
            string label;
            Color pill;

            switch (s.Mode)
            {
                case LinkMode.Bluetooth:
                    label = "Bluetooth";
                    pill = Color.FromRgb(0xC9, 0xCE, 0xD4);
                    StateLabel.Text = "ao vivo";
                    break;
                case LinkMode.Cable when s.Charging:
                    label = "Carregando";
                    pill = Color.FromRgb(0xFF, 0xFF, 0xFF);
                    StateLabel.Text = s.Percent.HasValue ? "última leitura sem fio" : "";
                    break;
                case LinkMode.Cable:
                    label = "No cabo";
                    pill = Color.FromRgb(0x8B, 0x91, 0x98);
                    StateLabel.Text = s.Percent.HasValue ? "última leitura sem fio" : "sem leitura anterior";
                    break;
                default:
                    label = "Desconectado";
                    pill = Color.FromRgb(0x8A, 0x90, 0x97);
                    StateLabel.Text = s.Percent.HasValue ? "última leitura conhecida" : "";
                    break;
            }

            ModeText.Text = label;
            var fg = new SolidColorBrush(pill);
            fg.Freeze();
            ModeText.Foreground = fg;
            ModeDot.Fill = fg;
            ModePill.Background = new SolidColorBrush(Color.FromArgb(0x1F, pill.R, pill.G, pill.B));
        }

        private void ApplyEstimate(BatteryState s)
        {
            if (s.KnownCount == 0 && s.Mode == LinkMode.Offline)
            {
                EstimateText.Text = "Nenhum controle encontrado";
                DrainText.Text = "ligue um controle por Bluetooth para começar";
                return;
            }

            if (s.Mode == LinkMode.Offline)
            {
                EstimateText.Text = "Controle desligado";
                DrainText.Text = s.Percent.HasValue ? "mostrando a última leitura conhecida" : "";
                return;
            }

            if (s.Mode == LinkMode.Cable)
            {
                EstimateText.Text = s.Charging ? "Carregando pelo cabo" : "Alimentado pelo cabo";
                DrainText.Text = "no cabo o controle não expõe percentual";
                return;
            }

            var est = s.Percent.HasValue ? _history.EstimateRemaining(s.Key, s.Percent.Value) : null;
            if (est.HasValue)
            {
                EstimateText.Text = "≈ " + FormatSpan(est.Value) + " restantes";
                var drain = _history.DrainPerHour(s.Key);
                DrainText.Text = drain.HasValue
                    ? string.Format(Br, "consumo de {0:0.0} %/h", drain.Value)
                    : "";
            }
            else
            {
                EstimateText.Text = "Medindo o consumo";
                DrainText.Text = "a estimativa aparece após alguns minutos de uso";
            }
        }

        private void ApplySpark(BatteryState s, SolidColorBrush brush)
        {
            var pts = _history.Recent(s.Key, TimeSpan.FromHours(24));
            Spark.Stroke = brush;
            Spark.Points = pts;

            if (pts.Count >= 2)
            {
                int lo = pts.Min(p => p.P), hi = pts.Max(p => p.P);
                SparkHint.Text = lo == hi ? $"{hi}%" : $"{lo}–{hi}%";
            }
            else SparkHint.Text = "sem dados ainda";
        }

        private void ApplyUpdated(BatteryState s)
        {
            if (!s.ReadAt.HasValue)
            {
                UpdatedText.Text = s.KnownCount == 0 ? "aguardando um controle" : "nenhuma leitura ainda";
                return;
            }

            string rel = Relative(s.ReadAt.Value);
            UpdatedText.Text = s.Stale ? $"Lido {rel} · sem fio" : $"Atualizado {rel}";
        }

        private static string Relative(DateTime t)
        {
            var d = DateTime.Now - t;
            if (d < TimeSpan.FromSeconds(45)) return "agora";
            if (d < TimeSpan.FromMinutes(60)) return $"há {(int)d.TotalMinutes} min";
            if (d < TimeSpan.FromHours(24)) return $"há {(int)d.TotalHours} h";
            return "em " + t.ToString("dd/MM 'às' HH:mm", Br);
        }

        private static string FormatSpan(TimeSpan t)
        {
            if (t.TotalHours >= 1)
            {
                int h = (int)t.TotalHours;
                int m = t.Minutes;
                return m > 0 ? $"{h} h {m} min" : $"{h} h";
            }
            return $"{Math.Max(1, (int)t.TotalMinutes)} min";
        }

        private static Color AccentFor(BatteryState s)
        {
            if (!s.Percent.HasValue || s.Mode == LinkMode.Offline) return Color.FromRgb(0x8A, 0x90, 0x97);
            if (s.Mode == LinkMode.Cable && s.Charging) return Color.FromRgb(0xFF, 0xFF, 0xFF);
            if (s.Percent <= 10) return Color.FromRgb(0xE5, 0x54, 0x4B);
            if (s.Percent <= 25) return Color.FromRgb(0xE3, 0xA9, 0x3C);
            return Color.FromRgb(0xFF, 0xFF, 0xFF);
        }

        // ---------- janela ----------

        public void ShowNearTray()
        {
            if (_state != null) Apply(_state);

            var wa = SystemParameters.WorkArea;
            Left = wa.Right - Width + 6;
            Top = wa.Bottom - Height + 6;

            Show();
            Activate();

            Opacity = 0;
            BeginAnimation(OpacityProperty, new DoubleAnimation(1, TimeSpan.FromMilliseconds(160)));
        }

        public void Toggle()
        {
            if (IsVisible) Hide();
            else ShowNearTray();
        }
    }
}


