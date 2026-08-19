using System;
using System.Collections.Generic;
using System.Threading;
using System.Windows;
using System.Windows.Threading;
using WF = System.Windows.Forms;
using D = System.Drawing;

namespace Kontro
{
    public partial class App : Application
    {
        private const string MutexName = @"Local\Kontro.SingleInstance";
        private const string ShowEventName = @"Local\Kontro.Show";

        private Settings _settings;
        private History _history;
        private KnownControllers _known;
        private BatteryMonitor _monitor;
        private FlyoutWindow _flyout;
        private MainWindow _main;
        private WF.NotifyIcon _tray;
        private DispatcherTimer _timer;
        private DispatcherTimer _updateTimer;
        private D.Icon _currentIcon;
        private Mutex _instanceMutex;
        private EventWaitHandle _showEvent;

        // limiares ja avisados nesta carga; zerados quando a bateria sobe (carga nova)
        private readonly HashSet<int> _notified = new();
        private int _lastPercentSeen = -1;
        private string _pendingUpdateVersion;

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            if (HandleToolFlags(e.Args)) return;

            // instancia unica: o segundo processo apenas acorda o primeiro e sai,
            // senao cada clique no atalho deixaria mais um icone na bandeja
            _instanceMutex = new Mutex(true, MutexName, out bool isFirstInstance);
            if (!isFirstInstance)
            {
                try { EventWaitHandle.OpenExisting(ShowEventName).Set(); }
                catch { /* a outra instancia pode estar encerrando */ }
                Shutdown();
                return;
            }

            AppPaths.MigrateLegacy();

            _settings = Settings.Load();
            // o registro manda: o usuario pode ter mexido no autostart por fora
            _settings.StartWithWindows = Autostart.IsEnabled();

            _history = new History();
            _history.Load();

            _known = new KnownControllers();
            _known.Load();

            _monitor = new BatteryMonitor(_history, _known);

            _flyout = new FlyoutWindow(_history);
            _flyout.QuitRequested += Quit;

            _main = new MainWindow(_settings);
            _main.QuitRequested += Quit;
            _main.SettingsChanged += OnSettingsChanged;

            _tray = new WF.NotifyIcon
            {
                Visible = true,
                Text = "Kontro",
                ContextMenuStrip = BuildMenu()
            };
            _tray.MouseClick += (_, args) =>
            {
                if (args.Button == WF.MouseButtons.Left) _flyout.Toggle();
            };
            _tray.MouseDoubleClick += (_, args) =>
            {
                if (args.Button == WF.MouseButtons.Left) OpenSettings();
            };
            _tray.BalloonTipClicked += OnBalloonClicked;

            StartShowListener();

            _monitor.Changed += OnStateChanged;
            OnStateChanged(_monitor.Current);

            _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(2) };
            _timer.Tick += async (_, _) =>
            {
                await _monitor.PollAsync();
                // mantem o "atualizado ha X" vivo enquanto o painel esta aberto
                if (_flyout.IsVisible) _flyout.Apply(_monitor.Current);
            };
            _timer.Start();

            _updateTimer = new DispatcherTimer { Interval = TimeSpan.FromHours(6) };
            _updateTimer.Tick += async (_, _) => await MaybeCheckUpdatesAsync();
            _updateTimer.Start();

            _ = StartupSequenceAsync(Array.Exists(e.Args, a => a == "--show"));
        }

        /// <summary>Modos utilitarios que nao sobem a interface. Retorna true se tratou.</summary>
        private bool HandleToolFlags(string[] args)
        {
            int diagIndex = Array.IndexOf(args, "--diagnose");
            if (diagIndex >= 0 && diagIndex + 1 < args.Length)
            {
                string outPath = args[diagIndex + 1];
                _ = System.Threading.Tasks.Task.Run(async () =>
                {
                    try { await Diagnostics.WriteAsync(outPath); }
                    catch (Exception ex) { System.IO.File.WriteAllText(outPath, "FALHOU:\n" + ex); }
                    finally { Dispatcher.Invoke(Shutdown); }
                });
                return true;
            }

            int makeIndex = Array.IndexOf(args, "--make-icon");
            if (makeIndex >= 0 && makeIndex + 1 < args.Length)
            {
                string ico = args[makeIndex + 1];
                string preview = makeIndex + 2 < args.Length ? args[makeIndex + 2] : null;
                IconBuilder.Write(ico, preview);
                Shutdown();
                return true;
            }

            int previewIndex = Array.IndexOf(args, "--icon-preview");
            if (previewIndex >= 0 && previewIndex + 1 < args.Length)
            {
                IconPreview.Write(args[previewIndex + 1]);
                Shutdown();
                return true;
            }

            return false;
        }

        private async System.Threading.Tasks.Task StartupSequenceAsync(bool forceShowFlyout)
        {
            await _monitor.PollAsync();            // descobre controles
            await _monitor.SeedFromWindowsAsync(); // usa o cache do Windows enquanto nao ha leitura ao vivo

            if (_settings.IsFirstRun)
            {
                // primeira execucao: a janela de configuracao e a apresentacao do app
                _main.ShowAndFocus();
            }
            else if (!_settings.StartMinimized && !forceShowFlyout)
            {
                _main.ShowAndFocus();
            }

            if (forceShowFlyout)
            {
                _flyout.Pinned = true;
                _flyout.ShowNearTray();
            }

            await MaybeCheckUpdatesAsync();
        }

        /// <summary>
        /// Fica ouvindo o sinal das instancias seguintes. Quando alguem tenta abrir o app
        /// de novo, em vez de subir outro processo nos trazemos a janela para a frente.
        /// </summary>
        private void StartShowListener()
        {
            _showEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ShowEventName);
            var listener = new Thread(() =>
            {
                while (true)
                {
                    try
                    {
                        _showEvent.WaitOne();
                        Dispatcher.Invoke(OpenSettings);
                    }
                    catch { return; }
                }
            })
            {
                IsBackground = true,
                Name = "ShowListener"
            };
            listener.Start();
        }

        private WF.ContextMenuStrip BuildMenu()
        {
            var menu = new WF.ContextMenuStrip();

            var panel = new WF.ToolStripMenuItem("Ver bateria");
            panel.Click += (_, _) => _flyout.ShowNearTray();

            var config = new WF.ToolStripMenuItem("Configurações...");
            config.Click += (_, _) => OpenSettings();

            var update = new WF.ToolStripMenuItem("Procurar atualizações");
            update.Click += async (_, _) => await CheckUpdatesInteractiveAsync();

            var quit = new WF.ToolStripMenuItem("Sair");
            quit.Click += (_, _) => Quit();

            menu.Items.Add(panel);
            menu.Items.Add(config);
            menu.Items.Add(new WF.ToolStripSeparator());
            menu.Items.Add(update);
            menu.Items.Add(new WF.ToolStripSeparator());
            menu.Items.Add(quit);
            return menu;
        }

        private void OpenSettings()
        {
            _flyout.Hide();
            _main.ShowAndFocus();
        }

        private void OnSettingsChanged()
        {
            // limiares novos merecem uma nova chance de avisar
            _notified.Clear();
        }

        // ---------- estado da bateria ----------

        private void OnStateChanged(BatteryState s)
        {
            Dispatcher.Invoke(() =>
            {
                UpdateTray(s);
                _flyout.Apply(s);
                CheckThresholds(s);
            });
        }

        private void UpdateTray(BatteryState s)
        {
            int size = WF.SystemInformation.SmallIconSize.Width;
            var icon = TrayRenderer.Render(s, size);

            var old = _currentIcon;
            _currentIcon = icon;
            _tray.Icon = icon;
            old?.Dispose();

            string mode = s.Mode switch
            {
                LinkMode.Bluetooth => "Bluetooth",
                LinkMode.Cable => s.Charging ? "carregando" : "no cabo",
                _ => "desconectado"
            };
            string pct = s.Percent.HasValue ? $"{s.Percent}%" : "sem leitura";
            string suffix = s.Stale && s.Percent.HasValue ? " (última leitura)" : "";

            string name = string.IsNullOrWhiteSpace(s.DeviceName) ? "Controle" : s.DeviceName;
            var text = $"{name} — {pct} · {mode}{suffix}";
            // o tooltip da bandeja trunca em 63 caracteres
            _tray.Text = text.Length > 62 ? text[..62] : text;
        }

        private void CheckThresholds(BatteryState s)
        {
            if (_settings == null || !_settings.NotificationsEnabled) return;
            if (!s.Percent.HasValue || s.Mode != LinkMode.Bluetooth) return;

            int p = s.Percent.Value;

            // subiu de forma relevante: carga nova, pode avisar de novo mais tarde
            if (_lastPercentSeen >= 0 && p - _lastPercentSeen > 5) _notified.Clear();
            _lastPercentSeen = p;

            string name = string.IsNullOrWhiteSpace(s.DeviceName) ? "O controle" : s.DeviceName;

            foreach (int limit in new[] { _settings.CriticalThreshold, _settings.WarnThreshold })
            {
                if (p > limit || _notified.Contains(limit)) continue;
                _notified.Add(limit);

                bool critical = limit == _settings.CriticalThreshold;
                _tray.ShowBalloonTip(
                    8000,
                    critical ? "Controle quase acabando" : "Bateria baixa no controle",
                    $"{name} está com {p}%. Vale trocar a pilha ou plugar o cabo.",
                    critical ? WF.ToolTipIcon.Error : WF.ToolTipIcon.Warning);
                break;
            }
        }

        // ---------- atualizacoes ----------

        private async System.Threading.Tasks.Task MaybeCheckUpdatesAsync()
        {
            if (_settings == null || !_settings.AutoCheckUpdates) return;
            if (_settings.LastUpdateCheck.HasValue &&
                (DateTime.Now - _settings.LastUpdateCheck.Value) < TimeSpan.FromHours(24)) return;

            try
            {
                var result = await Updater.CheckAsync();
                _settings.LastUpdateCheck = DateTime.Now;
                _settings.Save();

                if (!result.HasUpdate) return;
                if (result.Version == _settings.SkippedVersion) return;

                _pendingUpdateVersion = result.Version;
                _tray.ShowBalloonTip(
                    10000,
                    "Atualização disponível",
                    $"A versão {result.Version} saiu. Clique para ver os detalhes.",
                    WF.ToolTipIcon.Info);
            }
            catch { /* sem rede, tenta de novo no proximo ciclo */ }
        }

        private void OnBalloonClicked(object sender, EventArgs e)
        {
            if (_pendingUpdateVersion != null)
            {
                _pendingUpdateVersion = null;
                OpenSettings();
                return;
            }
            _flyout.ShowNearTray();
        }

        private async System.Threading.Tasks.Task CheckUpdatesInteractiveAsync()
        {
            OpenSettings();
            await System.Threading.Tasks.Task.CompletedTask;
        }

        // ---------- encerramento ----------

        private void Quit()
        {
            try
            {
                _timer?.Stop();
                _updateTimer?.Stop();
                _monitor?.Dispose();
                _history?.Save();
                _known?.Save();
                _settings?.Save();
                if (_tray != null) { _tray.Visible = false; _tray.Dispose(); }
                _currentIcon?.Dispose();
                _showEvent?.Dispose();
                if (_instanceMutex != null)
                {
                    try { _instanceMutex.ReleaseMutex(); } catch { }
                    _instanceMutex.Dispose();
                }
            }
            catch { }
            Shutdown();
        }
    }
}


