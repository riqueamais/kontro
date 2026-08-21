using System;
using System.Collections.Generic;
using System.Threading;
using System.Windows;
using System.Windows.Threading;
using Microsoft.Win32;
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
        private ToastWindow _toast;
        private OverlayWindow _overlay;
        private bool _avisouTelaCheiaExclusiva;
        private WF.NotifyIcon _tray;
        private DispatcherTimer _timer;
        private DispatcherTimer _updateTimer;
        private Mutex _instanceMutex;
        private EventWaitHandle _showEvent;

        // limiares ja avisados nesta carga; zerados quando a bateria sobe (carga nova)
        private readonly HashSet<int> _notified = new();
        private int _lastPercentSeen = -1;
        private LinkMode _modoAnterior = LinkMode.Offline;

        /// <summary>Conexao que ainda espera uma leitura para virar aviso.</summary>
        private DateTime? _conexaoAAvisar;

        /// <summary>
        /// Quanto esperar pela carga antes de avisar assim mesmo.
        ///
        /// A leitura real chega alguns segundos depois da conexao, e o aviso fica muito
        /// melhor com o numero pronto. Mas ha controle que nunca informa carga: passado
        /// este tempo, avisar sem numero e melhor que nao avisar.
        /// </summary>
        private static readonly TimeSpan EsperaDoAviso = TimeSpan.FromSeconds(10);
        private DateTime _inicio = DateTime.Now;
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
            _flyout.SettingsRequested += OpenSettings;
            _flyout.RefreshRequested += async () =>
            {
                // leitura sob demanda: o usuario pediu agora, nao no proximo ciclo
                await _monitor.PollAsync();
                _flyout.Apply(_monitor.Current);
            };

            _toast = new ToastWindow();

            _overlay = new OverlayWindow();
            _overlay.AplicarPreferencias(_settings);

            _main = new MainWindow(_settings, _history);
            _main.QuitRequested += Quit;
            _main.SettingsChanged += OnSettingsChanged;
            // aparecer ou sumir muda a previa da sobreposicao, mas nao as preferencias
            _main.VisibilityChanged += () =>
            {
                if (_monitor != null) AtualizarSobreposicao(_monitor.Current);
            };

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

            SystemEvents.UserPreferenceChanged += OnUserPreferenceChanged;

            StartShowListener();

            _monitor.Changed += OnStateChanged;
            OnStateChanged(_monitor.Current);

            _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(2) };
            _timer.Tick += async (_, _) =>
            {
                await _monitor.PollAsync();
                // mantem o "atualizado ha X" vivo enquanto o painel esta aberto
                if (_flyout.IsVisible) _flyout.Apply(_monitor.Current);

                // A visibilidade da sobreposicao depende do que ocupa a tela, e nao do
                // estado da bateria. Reavaliar so quando a carga muda faria entrar e
                // sair de um jogo nao ter efeito ate a proxima variacao de percentual,
                // o que pode demorar muitos minutos.
                AtualizarSobreposicao(_monitor.Current);

                // controle que nunca informa carga nao gera mudanca de estado nenhuma:
                // sem passar por aqui, o aviso marcado ficaria esperando para sempre
                TalvezAvisar(_monitor.Current);
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

            int checkIndex = Array.IndexOf(args, "--check-update");
            if (checkIndex >= 0)
            {
                string saida = checkIndex + 1 < args.Length && !args[checkIndex + 1].StartsWith("--")
                    ? args[checkIndex + 1]
                    : null;
                bool aplicar = Array.IndexOf(args, "--apply") >= 0;
                _ = System.Threading.Tasks.Task.Run(async () =>
                {
                    string texto;
                    try
                    {
                        var r = await Updater.CheckAsync();
                        texto = $"versao atual : {Updater.CurrentVersion}\n" +
                                $"instalado    : {Updater.IsInstalled}\n" +
                                $"tem update   : {r.HasUpdate}\n" +
                                $"versao nova  : {r.Version ?? "-"}\n" +
                                $"aplicavel    : {r.CanApply}\n" +
                                $"mensagem     : {r.Message}\n";

                        if (aplicar && r.HasUpdate && r.CanApply)
                        {
                            texto += "\naplicando...\n";
                            if (saida != null) System.IO.File.WriteAllText(saida, texto);
                            await Updater.ApplyAsync(r);   // reinicia o processo, nao retorna
                            texto += "a atualizacao nao foi aplicada\n";
                        }
                    }
                    catch (Exception ex) { texto = "FALHOU: " + ex; }

                    if (saida != null) System.IO.File.WriteAllText(saida, texto);
                    Dispatcher.Invoke(Shutdown);
                });
                return true;
            }

            // mostra o aviso com um estado inventado, para conferir o desenho sem
            // depender de ligar e desligar o controle
            int toastIndex = Array.IndexOf(args, "--toast-demo");
            if (toastIndex >= 0)
            {
                int pct = toastIndex + 1 < args.Length && int.TryParse(args[toastIndex + 1], out var v) ? v : 72;
                var janela = new ToastWindow();
                janela.Mostrar(new BatteryState
                {
                    Mode = LinkMode.Bluetooth,
                    Percent = pct,
                    Precisao = Precisao.Exata,
                    ReadAt = DateTime.Now,
                    DeviceName = "Controle sem fio",
                    Key = "demo"
                });
                var relogio = new DispatcherTimer { Interval = TimeSpan.FromSeconds(9) };
                relogio.Tick += (_, _) => Shutdown();
                relogio.Start();
                return true;
            }

            // mesma ideia para a caixa de dialogo: conferir o desenho sem precisar de
            // uma release nova esperando do outro lado
            if (Array.IndexOf(args, "--dialog-demo") >= 0)
            {
                DialogWindow.Perguntar(null, "Atualização disponível",
                    "A versão 1.5.0 está disponível. Você está na 1.4.0.\n\n" +
                    "Baixar e instalar agora? O app reinicia sozinho ao terminar.",
                    "Atualizar agora");
                Shutdown();
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

        private WF.ToolStripMenuItem _estadoItem;

        private WF.ContextMenuStrip BuildMenu()
        {
            var menu = new WF.ContextMenuStrip
            {
                Renderer = new TrayMenuRenderer(),
                BackColor = TrayMenuRenderer.SurfaceAlt,
                ForeColor = TrayMenuRenderer.TextPrimary,
                ShowImageMargin = false,
                Font = new D.Font("Segoe UI", 9f)
            };
            menu.MinimumSize = new D.Size(200, 0);

            // primeiro item e o estado, desabilitado: serve de cabecalho, nao de acao
            _estadoItem = new WF.ToolStripMenuItem("Sem leitura") { Enabled = false };

            var painel = new WF.ToolStripMenuItem("Ver bateria");
            painel.Click += (_, _) => _flyout.ShowNearTray();

            var config = new WF.ToolStripMenuItem("Configurações...");
            config.Click += (_, _) => OpenSettings();

            var update = new WF.ToolStripMenuItem("Procurar atualizações");
            update.Click += (_, _) => OpenSettings();

            var quit = new WF.ToolStripMenuItem("Sair");
            quit.Click += (_, _) => Quit();

            menu.Items.Add(_estadoItem);
            menu.Items.Add(new WF.ToolStripSeparator());
            menu.Items.Add(painel);
            menu.Items.Add(config);
            menu.Items.Add(new WF.ToolStripSeparator());
            menu.Items.Add(update);
            menu.Items.Add(quit);

            foreach (WF.ToolStripItem item in menu.Items)
                if (item is WF.ToolStripMenuItem mi) mi.Padding = new WF.Padding(12, 8, 12, 8);

            return menu;
        }

        private void OpenSettings()
        {
            _flyout.Hide();
            _main.ShowAndFocus();
            if (_monitor != null) AtualizarSobreposicao(_monitor.Current);
        }

        /// <summary>
        /// A barra de tarefas pode trocar de claro para escuro a qualquer momento.
        /// Quando isso acontece o icone precisa ser redesenhado, senao o glifo branco
        /// some numa barra clara.
        /// </summary>
        private void OnUserPreferenceChanged(object sender, UserPreferenceChangedEventArgs e)
        {
            if (e.Category != UserPreferenceCategory.General &&
                e.Category != UserPreferenceCategory.Color &&
                e.Category != UserPreferenceCategory.VisualStyle) return;

            Dispatcher.BeginInvoke(new Action(() =>
            {
                // o tema decide qual pasta de icones usar, entao o cache inteiro caduca
                TrayRenderer.LimparCache();
                if (_monitor != null) UpdateTray(_monitor.Current);
            }));
        }

        private void OnSettingsChanged()
        {
            // limiares novos merecem uma nova chance de avisar
            _notified.Clear();
            _overlay?.AplicarPreferencias(_settings);
            if (_monitor != null) AtualizarSobreposicao(_monitor.Current);
        }

        // ---------- estado da bateria ----------

        private void OnStateChanged(BatteryState s)
        {
            Dispatcher.Invoke(() =>
            {
                UpdateTray(s);
                _flyout.Apply(s);
                _main.Apply(s);
                CheckThresholds(s);
                AvisarConexao(s);
                TalvezAvisar(s);
                _toast?.Atualizar(s);
                AtualizarSobreposicao(s);
            });
        }

        /// <summary>
        /// Mostra o aviso quando o controle sai de desconectado para conectado.
        ///
        /// Os primeiros segundos sao ignorados de proposito: ao abrir o app o estado
        /// parte de desconectado e logo encontra o controle, o que dispararia um aviso
        /// em toda inicializacao sem nada ter acontecido de fato.
        /// </summary>
        private void AvisarConexao(BatteryState s)
        {
            var anterior = _modoAnterior;
            _modoAnterior = s.Mode;

            if (s.Mode == LinkMode.Offline) { _conexaoAAvisar = null; return; }
            if (_settings == null || !_settings.ConnectToastEnabled) return;
            if (anterior != LinkMode.Offline) return;
            if ((DateTime.Now - _inicio) < TimeSpan.FromSeconds(6)) return;

            // Conectar e um evento; ter a carga e outro, alguns segundos depois. O aviso
            // fica marcado aqui e sai quando o numero existe -- assim ele nasce pronto,
            // em vez de aparecer dizendo que esta lendo.
            _conexaoAAvisar = DateTime.Now;
        }

        /// <summary>Solta o aviso marcado quando a carga chega, ou quando a espera acaba.</summary>
        private void TalvezAvisar(BatteryState s)
        {
            if (_conexaoAAvisar == null || _toast == null) return;
            if (s.Mode == LinkMode.Offline) { _conexaoAAvisar = null; return; }

            bool temCarga = !s.Stale && s.Preenchimento.HasValue;
            bool cansou = (DateTime.Now - _conexaoAAvisar.Value) > EsperaDoAviso;
            if (!temCarga && !cansou) return;

            _conexaoAAvisar = null;
            _toast.Mostrar(s);
        }

        /// <summary>
        /// Decide se a sobreposicao aparece agora, e onde.
        ///
        /// O modo "em jogo" se apoia no estado que o Windows relata sobre o que ocupa a
        /// tela, em vez de tentar reconhecer jogos por nome de processo -- lista que
        /// nunca estaria completa nem atualizada.
        /// </summary>
        private void AtualizarSobreposicao(BatteryState s)
        {
            if (_overlay == null || _settings == null) return;

            bool ligada = _settings.OverlayMode != OverlayMode.Desligada;
            bool temLeitura = s.Mode != LinkMode.Offline;
            bool emMomentoDeJogo = _settings.OverlayMode == OverlayMode.Sempre || Native.EmTelaCheia();

            // Enquanto as configuracoes estao abertas a sobreposicao fica visivel de
            // qualquer jeito, funcionando como previa. Sem isso, escolher o canto seria
            // as cegas: ao clicar no botao a janela de configuracoes vira o primeiro
            // plano, o app conclui que saiu do jogo e esconde justamente o que o usuario
            // esta tentando posicionar.
            bool ajustando = _main != null && _main.IsVisible;

            if (ligada && (ajustando || (temLeitura && emMomentoDeJogo)))
            {
                _overlay.Aplicar(s);
                if (!_overlay.IsVisible) _overlay.Show();
                _overlay.Reposicionar();   // o jogo pode ter mudado de monitor
                AvisarTelaCheiaExclusiva();
            }
            else if (_overlay.IsVisible)
            {
                _overlay.Hide();
            }
        }

        /// <summary>
        /// Explica, uma unica vez, por que a sobreposicao pode nao aparecer em certos
        /// jogos.
        ///
        /// A redacao e condicional de proposito. O estado que o Windows relata nao
        /// separa tela cheia exclusiva de tela cheia em janela: uma janela sem bordas
        /// ocupando o monitor tambem e relatada como exclusiva. Afirmar que nao vai
        /// aparecer seria falso justamente nos jogos em janela, onde ela aparece.
        /// </summary>
        private void AvisarTelaCheiaExclusiva()
        {
            if (_avisouTelaCheiaExclusiva) return;
            if (_settings == null || _settings.OverlayTipShown) return;
            if (!Native.EmTelaCheia()) return;

            _avisouTelaCheiaExclusiva = true;
            _settings.OverlayTipShown = true;
            _settings.Save();

            _tray?.ShowBalloonTip(
                10000,
                "Sobreposição ativa",
                "Se ela não aparecer dentro de algum jogo, é porque ele está em tela cheia exclusiva. " +
                "Trocar para \"tela cheia em janela\" nas opções de vídeo resolve.",
                WF.ToolTipIcon.Info);
        }

        private void UpdateTray(BatteryState s)
        {
            int size = WF.SystemInformation.SmallIconSize.Width;
            var icon = TrayRenderer.Render(s, size, _settings);
            // nao descartamos o icone: quem e dono dele e o cache do TrayRenderer
            if (icon != null) _tray.Icon = icon;

            string mode = s.TextoDaLigacao;
            string pct = s.Preenchimento.HasValue ? s.TextoDaCarga : "sem leitura";
            string suffix = s.Stale && s.Preenchimento.HasValue ? " (última leitura)" : "";

            string name = string.IsNullOrWhiteSpace(s.DeviceName) ? "Controle" : s.DeviceName;
            var text = $"{name} — {pct} · {mode}{suffix}";
            // o tooltip da bandeja trunca em 63 caracteres
            _tray.Text = text.Length > 62 ? text[..62] : text;
            if (_estadoItem != null) _estadoItem.Text = $"{pct} · {mode}";
        }

        private void CheckThresholds(BatteryState s)
        {
            if (_settings == null || !_settings.NotificationsEnabled) return;
            // limiar em porcentagem so faz sentido com leitura exata; com quatro
            // degraus nao da para dizer se passou de 20%
            if (!s.TemNumero || s.Mode == LinkMode.Offline) return;

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
                SystemEvents.UserPreferenceChanged -= OnUserPreferenceChanged;
                _timer?.Stop();
                _updateTimer?.Stop();
                _monitor?.Dispose();
                _history?.Save();
                _known?.Save();
                _settings?.Save();
                if (_tray != null) { _tray.Visible = false; _tray.Dispose(); }
                TrayRenderer.LimparCache();
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


