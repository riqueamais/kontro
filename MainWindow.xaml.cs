using System;
using System.Windows;
using System.Windows.Interop;

namespace Kontro
{
    public partial class MainWindow : Window
    {
        private readonly Settings _settings;
        private bool _loading = true;

        /// <summary>Disparado quando o usuario pede para encerrar o app de vez.</summary>
        public event Action QuitRequested;

        /// <summary>Disparado quando qualquer preferencia muda, para o app reagir na hora.</summary>
        public event Action SettingsChanged;

        public MainWindow(Settings settings)
        {
            InitializeComponent();
            _settings = settings;

            SourceInitialized += (_, _) =>
            {
                var handle = new WindowInteropHelper(this).Handle;
                Native.UseDarkTitleBar(handle);
            };

            LoadFromSettings();
            WireEvents();
            ApplyFirstRunChrome();

            _loading = false;
        }

        // ---------- estado ----------

        private void LoadFromSettings()
        {
            SwStartup.IsChecked = _settings.StartWithWindows;
            SwStartMinimized.IsChecked = _settings.StartMinimized;
            RbMinimize.IsChecked = _settings.CloseAction == CloseAction.MinimizeToTray;
            RbExit.IsChecked = _settings.CloseAction == CloseAction.Exit;

            SwNotify.IsChecked = _settings.NotificationsEnabled;
            SlWarn.Value = _settings.WarnThreshold;
            SlCrit.Value = _settings.CriticalThreshold;
            UpdateThresholdLabels();

            SwAutoUpdate.IsChecked = _settings.AutoCheckUpdates;
            VersionText.Text = "Versão " + Updater.CurrentVersion;
            UpdateStatus.Text = _settings.LastUpdateCheck.HasValue
                ? "Última verificação em " + _settings.LastUpdateCheck.Value.ToString("dd/MM 'às' HH:mm")
                : "Nunca verificado.";
        }

        private void WireEvents()
        {
            SwStartup.Checked += (_, _) => OnStartupToggled(true);
            SwStartup.Unchecked += (_, _) => OnStartupToggled(false);

            SwStartMinimized.Checked += (_, _) => Persist(() => _settings.StartMinimized = true);
            SwStartMinimized.Unchecked += (_, _) => Persist(() => _settings.StartMinimized = false);

            RbMinimize.Checked += (_, _) => Persist(() => _settings.CloseAction = CloseAction.MinimizeToTray);
            RbExit.Checked += (_, _) => Persist(() => _settings.CloseAction = CloseAction.Exit);

            SwNotify.Checked += (_, _) => Persist(() => _settings.NotificationsEnabled = true);
            SwNotify.Unchecked += (_, _) => Persist(() => _settings.NotificationsEnabled = false);

            SlWarn.ValueChanged += (_, _) =>
            {
                if (_loading) return;
                // o critico nunca pode encostar no aviso, senao um engole o outro
                if (SlCrit.Value >= SlWarn.Value) SlCrit.Value = Math.Max(SlCrit.Minimum, SlWarn.Value - 5);
                Persist(() => _settings.WarnThreshold = (int)SlWarn.Value);
                UpdateThresholdLabels();
            };
            SlCrit.ValueChanged += (_, _) =>
            {
                if (_loading) return;
                if (SlCrit.Value >= SlWarn.Value) SlCrit.Value = Math.Max(SlCrit.Minimum, SlWarn.Value - 5);
                Persist(() => _settings.CriticalThreshold = (int)SlCrit.Value);
                UpdateThresholdLabels();
            };

            SwAutoUpdate.Checked += (_, _) => Persist(() => _settings.AutoCheckUpdates = true);
            SwAutoUpdate.Unchecked += (_, _) => Persist(() => _settings.AutoCheckUpdates = false);

            BtnCheckUpdate.Click += async (_, _) => await CheckUpdatesAsync();
            BtnDone.Click += (_, _) => CompleteFirstRun();
            BtnQuit.Click += (_, _) => QuitRequested?.Invoke();

            Closing += OnClosing;
        }

        private void OnStartupToggled(bool on)
        {
            if (_loading) return;
            Autostart.Set(on);
            // se o registro recusar, o interruptor nao pode mentir para o usuario
            bool actual = Autostart.IsEnabled();
            if (actual != on)
            {
                _loading = true;
                SwStartup.IsChecked = actual;
                _loading = false;
            }
            Persist(() => _settings.StartWithWindows = actual);
        }

        private void UpdateThresholdLabels()
        {
            WarnValue.Text = $"{(int)SlWarn.Value}%";
            CritValue.Text = $"{(int)SlCrit.Value}%";
        }

        private void Persist(Action mutate)
        {
            if (_loading) return;
            mutate();
            _settings.Save();
            SettingsChanged?.Invoke();
        }

        // ---------- primeira execucao ----------

        private void ApplyFirstRunChrome()
        {
            if (_settings.IsFirstRun)
            {
                HeaderTitle.Text = "Tudo pronto";
                HeaderSubtitle.Text = "Ajuste como o app deve se comportar. Dá para mudar depois pelo ícone da bandeja.";
                FooterHint.Text = "O app fica na bandeja, ao lado do relógio.";
                BtnDone.Content = "Concluir";
                BtnQuit.Visibility = Visibility.Collapsed;
            }
            else
            {
                HeaderTitle.Text = "Kontro";
                HeaderSubtitle.Text = "Configure como o app se comporta.";
                FooterHint.Text = "";
                BtnDone.Content = "Fechar";
                BtnQuit.Visibility = Visibility.Visible;
            }
        }

        private void CompleteFirstRun()
        {
            if (_settings.IsFirstRun)
            {
                _settings.FirstRunDone = true;
                _settings.Save();
                ApplyFirstRunChrome();
                SettingsChanged?.Invoke();
            }
            Hide();
        }

        // ---------- atualizacoes ----------

        private async System.Threading.Tasks.Task CheckUpdatesAsync()
        {
            BtnCheckUpdate.IsEnabled = false;
            UpdateStatus.Text = "Procurando...";
            try
            {
                var result = await Updater.CheckAsync();
                _settings.LastUpdateCheck = DateTime.Now;
                _settings.Save();

                UpdateStatus.Text = result.Message;

                if (result.HasUpdate)
                {
                    var answer = MessageBox.Show(
                        this,
                        $"A versão {result.Version} está disponível.\n\nVocê está na {Updater.CurrentVersion}.\n\nBaixar e instalar agora? O app reinicia ao terminar.",
                        "Atualização disponível",
                        MessageBoxButton.YesNo, MessageBoxImage.Information);

                    if (answer == MessageBoxResult.Yes)
                    {
                        UpdateStatus.Text = "Baixando atualização...";
                        await Updater.ApplyAsync(result);
                        // se chegou aqui, o reinicio nao aconteceu
                        UpdateStatus.Text = "Não foi possível aplicar a atualização.";
                    }
                }
            }
            catch (Exception ex)
            {
                UpdateStatus.Text = "Falha ao verificar: " + ex.Message;
            }
            finally
            {
                BtnCheckUpdate.IsEnabled = true;
            }
        }

        // ---------- janela ----------

        private void OnClosing(object sender, System.ComponentModel.CancelEventArgs e)
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
            ApplyFirstRunChrome();
            LoadFromSettings();
            Show();
            if (WindowState == WindowState.Minimized) WindowState = WindowState.Normal;
            Activate();
            Topmost = true;
            Topmost = false;
            Focus();
        }
    }
}


