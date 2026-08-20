using System;
using System.Globalization;
using System.Linq;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Animation;

namespace Kontro
{
    public partial class MainWindow : Window
    {
        private static readonly CultureInfo Br = CultureInfo.GetCultureInfo("pt-BR");

        private const int AmbarAbaixoDe = 60;
        private const int VermelhoAbaixoDe = 30;

        private readonly Settings _settings;
        private readonly History _history;
        private bool _carregando = true;

        public event Action QuitRequested;
        public event Action SettingsChanged;

        public MainWindow(Settings settings, History history)
        {
            InitializeComponent();
            _settings = settings;
            _history = history;

            SourceInitialized += (_, _) =>
                Native.UseDarkTitleBar(new WindowInteropHelper(this).Handle, 0x0B0E11, 0xE8ECEF);

            CarregarDasConfiguracoes();
            LigarEventos();
            AplicarPrimeiraExecucao();

            _carregando = false;
        }

        // ---------- estado da bateria ----------

        public void Apply(BatteryState s)
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
                StateText.Text = s.Charging
                    ? "Carregando pelo cabo"
                    : "No cabo · o controle não expõe percentual";
            }
            else
            {
                PercentText.Visibility = Visibility.Visible;
                PercentText.Text = s.Percent.HasValue ? s.Percent.Value + "%" : "--";
                StateText.Text = s.Mode == LinkMode.Offline
                    ? "Desconectado"
                    : Autonomia(s);
            }

            Ring.BeginAnimation(RingControl.ValueProperty,
                new DoubleAnimation(s.Mode == LinkMode.Cable ? 100 : (s.Percent ?? 0),
                                    TimeSpan.FromMilliseconds(180))
                {
                    EasingFunction = new CubicEase { EasingMode = EasingMode.EaseOut }
                });

            UpdatedText.Text = s.ReadAt.HasValue
                ? (s.Stale ? "lido às " : "atualizado às ") + s.ReadAt.Value.ToString("HH:mm", Br)
                : "sem leitura ainda";

            AplicarHistorico(s, pincel);
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

        private void AplicarHistorico(BatteryState s, Brush pincel)
        {
            var pontos = _history.Recent(s.Key, TimeSpan.FromHours(24));
            // o design pede a linha em verde a 40%: e contexto, nao protagonista
            var linha = new SolidColorBrush(ControllerGeometry.Green) { Opacity = 0.4 };
            linha.Freeze();
            Spark.Stroke = linha;
            Spark.Points = pontos;

            HistoryRange.Text = pontos.Count >= 2
                ? $"{pontos.Min(p => p.P)}% – {pontos.Max(p => p.P)}%"
                : "sem dados ainda";
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
            if (s.Mode != LinkMode.Bluetooth || !s.Percent.HasValue) return ControllerGeometry.Gray;
            if (s.Percent < VermelhoAbaixoDe) return ControllerGeometry.Red;
            if (s.Percent < AmbarAbaixoDe) return ControllerGeometry.Amber;
            return ControllerGeometry.Green;
        }

        // ---------- configuracoes ----------

        private void CarregarDasConfiguracoes()
        {
            SwStartup.IsChecked = _settings.StartWithWindows;
            SwStartMinimized.IsChecked = _settings.StartMinimized;
            SwNotify.IsChecked = _settings.NotificationsEnabled;
            SwConnectToast.IsChecked = _settings.ConnectToastEnabled;
            SwAutoUpdate.IsChecked = _settings.AutoCheckUpdates;

            SlWarn.Value = _settings.WarnThreshold;
            SlCrit.Value = _settings.CriticalThreshold;
            AtualizarRotulos();

            VersionText.Text = "Versão " + Updater.CurrentVersion;
            UpdateStatus.Text = _settings.LastUpdateCheck.HasValue
                ? "última verificação em " + _settings.LastUpdateCheck.Value.ToString("dd/MM 'às' HH:mm", Br)
                : "nunca verificado";
        }

        private void LigarEventos()
        {
            SwStartup.Checked += (_, _) => AlternarAutostart(true);
            SwStartup.Unchecked += (_, _) => AlternarAutostart(false);

            SwStartMinimized.Checked += (_, _) => Gravar(() => _settings.StartMinimized = true);
            SwStartMinimized.Unchecked += (_, _) => Gravar(() => _settings.StartMinimized = false);

            SwConnectToast.Checked += (_, _) => Gravar(() => _settings.ConnectToastEnabled = true);
            SwConnectToast.Unchecked += (_, _) => Gravar(() => _settings.ConnectToastEnabled = false);

            SwNotify.Checked += (_, _) => Gravar(() => _settings.NotificationsEnabled = true);
            SwNotify.Unchecked += (_, _) => Gravar(() => _settings.NotificationsEnabled = false);

            SwAutoUpdate.Checked += (_, _) => Gravar(() => _settings.AutoCheckUpdates = true);
            SwAutoUpdate.Unchecked += (_, _) => Gravar(() => _settings.AutoCheckUpdates = false);

            SlWarn.ValueChanged += (_, _) =>
            {
                if (_carregando) return;
                // o critico nunca pode encostar no aviso, senao um engole o outro
                if (SlCrit.Value >= SlWarn.Value) SlCrit.Value = Math.Max(SlCrit.Minimum, SlWarn.Value - 5);
                Gravar(() => _settings.WarnThreshold = (int)SlWarn.Value);
                AtualizarRotulos();
            };
            SlCrit.ValueChanged += (_, _) =>
            {
                if (_carregando) return;
                if (SlCrit.Value >= SlWarn.Value) SlCrit.Value = Math.Max(SlCrit.Minimum, SlWarn.Value - 5);
                Gravar(() => _settings.CriticalThreshold = (int)SlCrit.Value);
                AtualizarRotulos();
            };

            // O pacote de design nao traz estilo de lista suspensa, entao a escolha
            // cicla no proprio botao -- mesmo padrao ja usado na acao do X.
            BtnOverlayMode.Click += (_, _) =>
            {
                _settings.OverlayMode = _settings.OverlayMode switch
                {
                    OverlayMode.Desligada => OverlayMode.EmJogo,
                    OverlayMode.EmJogo => OverlayMode.Sempre,
                    _ => OverlayMode.Desligada
                };
                Gravar(() => { });
                AtualizarRotulos();
            };

            BtnOverlayCorner.Click += (_, _) =>
            {
                _settings.OverlayCorner = _settings.OverlayCorner switch
                {
                    OverlayCorner.SuperiorEsquerdo => OverlayCorner.SuperiorDireito,
                    OverlayCorner.SuperiorDireito => OverlayCorner.InferiorDireito,
                    OverlayCorner.InferiorDireito => OverlayCorner.InferiorEsquerdo,
                    _ => OverlayCorner.SuperiorEsquerdo
                };
                Gravar(() => { });
                AtualizarRotulos();
            };

            BtnCloseAction.Click += (_, _) =>
            {
                _settings.CloseAction = _settings.CloseAction == CloseAction.MinimizeToTray
                    ? CloseAction.Exit
                    : CloseAction.MinimizeToTray;
                Gravar(() => { });
                AtualizarRotulos();
            };

            BtnCheckUpdate.Click += async (_, _) => await ProcurarAtualizacaoAsync();
            BtnDone.Click += (_, _) => ConcluirPrimeiraExecucao();
            BtnQuit.Click += (_, _) => QuitRequested?.Invoke();

            Closing += AoFechar;
        }

        private void AlternarAutostart(bool ligar)
        {
            if (_carregando) return;
            Autostart.Set(ligar);
            // se o registro recusar, o interruptor nao pode mentir para o usuario
            bool real = Autostart.IsEnabled();
            if (real != ligar)
            {
                _carregando = true;
                SwStartup.IsChecked = real;
                _carregando = false;
            }
            Gravar(() => _settings.StartWithWindows = real);
        }

        private void AtualizarRotulos()
        {
            WarnValue.Text = $"{(int)SlWarn.Value}%";
            CritValue.Text = $"{(int)SlCrit.Value}%";
            BtnCloseAction.Content = _settings.CloseAction == CloseAction.MinimizeToTray
                ? "Minimizar para a bandeja"
                : "Fechar o app";

            BtnOverlayMode.Content = _settings.OverlayMode switch
            {
                OverlayMode.Desligada => "Desligada",
                OverlayMode.EmJogo => "Só em jogo",
                _ => "Sempre visível"
            };

            BtnOverlayCorner.Content = _settings.OverlayCorner switch
            {
                OverlayCorner.SuperiorEsquerdo => "Superior esquerdo",
                OverlayCorner.SuperiorDireito => "Superior direito",
                OverlayCorner.InferiorDireito => "Inferior direito",
                _ => "Inferior esquerdo"
            };
        }

        private void Gravar(Action alterar)
        {
            if (_carregando) return;
            alterar();
            _settings.Save();
            SettingsChanged?.Invoke();
        }

        // ---------- primeira execucao ----------

        private void AplicarPrimeiraExecucao()
        {
            bool primeira = _settings.IsFirstRun;
            FooterHint.Text = primeira ? "O app fica na bandeja, ao lado do relógio." : "";
            BtnDone.Content = primeira ? "Concluir" : "Fechar";
            BtnQuit.Visibility = primeira ? Visibility.Collapsed : Visibility.Visible;
        }

        private void ConcluirPrimeiraExecucao()
        {
            if (_settings.IsFirstRun)
            {
                _settings.FirstRunDone = true;
                _settings.Save();
                AplicarPrimeiraExecucao();
                SettingsChanged?.Invoke();
            }
            Hide();
        }

        // ---------- atualizacoes ----------

        private async System.Threading.Tasks.Task ProcurarAtualizacaoAsync()
        {
            BtnCheckUpdate.IsEnabled = false;
            UpdateStatus.Text = "procurando...";
            try
            {
                var r = await Updater.CheckAsync();
                _settings.LastUpdateCheck = DateTime.Now;
                _settings.Save();
                UpdateStatus.Text = r.Message;

                if (r.HasUpdate)
                {
                    var resposta = MessageBox.Show(this,
                        $"A versão {r.Version} está disponível.\n\nVocê está na {Updater.CurrentVersion}.\n\nBaixar e instalar agora? O app reinicia ao terminar.",
                        "Atualização disponível", MessageBoxButton.YesNo, MessageBoxImage.Information);

                    if (resposta == MessageBoxResult.Yes)
                    {
                        UpdateStatus.Text = "baixando atualização...";
                        await Updater.ApplyAsync(r);
                        UpdateStatus.Text = "não foi possível aplicar a atualização";
                    }
                }
            }
            catch (Exception ex)
            {
                UpdateStatus.Text = "falha ao verificar: " + ex.Message;
            }
            finally { BtnCheckUpdate.IsEnabled = true; }
        }

        // ---------- janela ----------

        private void AoFechar(object sender, System.ComponentModel.CancelEventArgs e)
        {
            if (_settings.CloseAction == CloseAction.MinimizeToTray)
            {
                e.Cancel = true;
                Hide();
                return;
            }
            QuitRequested?.Invoke();
        }

        public void ShowAndFocus()
        {
            AplicarPrimeiraExecucao();
            CarregarDasConfiguracoes();
            Show();
            if (WindowState == WindowState.Minimized) WindowState = WindowState.Normal;
            Activate();
            Topmost = true;
            Topmost = false;
            Focus();
        }
    }
}
