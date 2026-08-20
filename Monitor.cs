using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Windows.Devices.Bluetooth;
using Windows.Devices.Bluetooth.GenericAttributeProfile;
using Windows.Gaming.Input;
using Windows.Storage.Streams;

namespace Kontro
{
    /// <summary>
    /// Como o controle esta ligado. Sem fio nao e sinonimo de Bluetooth: dongle de radio
    /// e adaptador sem fio ligam sem cabo e sem Bluetooth nenhum, e a diferenca importa
    /// porque so o Bluetooth entrega percentual exato e avisa sozinho quando muda.
    /// </summary>
    public enum LinkMode { Offline, Bluetooth, Cable, Wireless }

    public sealed class BatteryState
    {
        public LinkMode Mode { get; init; }
        public int? Percent { get; init; }

        /// <summary>
        /// Quanto vale o numero acima. Nem toda via de leitura entrega percentual: o
        /// XInput so sabe quatro degraus, e transformar isso em porcentagem seria
        /// inventar precisao. Quando e aproximada, Percent fica nulo e Nivel manda.
        /// </summary>
        public Precisao Precisao { get; init; }
        public int? Nivel { get; init; }
        public DateTime? ReadAt { get; init; }
        public bool Charging { get; init; }
        /// <summary>True quando o percentual e a ultima leitura conhecida, nao um valor ao vivo.</summary>
        public bool Stale { get; init; }
        public string DeviceName { get; init; }
        public string Address { get; init; }
        public string Key { get; init; }
        public int KnownCount { get; init; }

        /// <summary>Quanto do anel preencher. Nulo quando nao ha leitura alguma.</summary>
        public int? Preenchimento => Precisao switch
        {
            Precisao.Exata => Percent,
            Precisao.Aproximada when Nivel.HasValue => BatteryReaders.PreenchimentoDoNivel(Nivel.Value),
            _ => null
        };

        /// <summary>
        /// O que escrever sobre a carga. Com leitura aproximada devolve texto em vez de
        /// numero: mostrar "35%" para um dado que so sabe dizer "baixa" seria inventar
        /// uma precisao que ninguem mediu.
        /// </summary>
        public string TextoDaCarga => Precisao switch
        {
            Precisao.Exata when Percent.HasValue => Percent.Value + "%",
            Precisao.Aproximada when Nivel.HasValue => BatteryReaders.DescreverNivel(Nivel.Value),
            _ => "--"
        };

        /// <summary>Verdadeiro quando ha numero para mostrar dentro do anel.</summary>
        public bool TemNumero => Precisao == Precisao.Exata && Percent.HasValue;

        /// <summary>Como o controle esta ligado, em uma palavra.</summary>
        public string TextoDaLigacao => Mode switch
        {
            LinkMode.Bluetooth => "Bluetooth",
            LinkMode.Wireless => "sem fio",
            LinkMode.Cable => Charging ? "carregando" : "no cabo",
            _ => "desconectado"
        };

        /// <summary>
        /// Controle ligado do qual nao se conseguiu carga alguma. Acontece com controle
        /// que so existe para o XInput e nao reporta bateria: dizer isso e mais util que
        /// mostrar um tracinho e deixar o usuario achando que o app travou.
        /// </summary>
        public bool ConectadoSemCarga => Mode != LinkMode.Offline && Preenchimento == null;

        public bool SameAs(BatteryState o) =>
            o != null && o.Mode == Mode && o.Percent == Percent && o.Charging == Charging
            && o.Stale == Stale && o.Key == Key && o.DeviceName == DeviceName
            && o.KnownCount == KnownCount && o.Precisao == Precisao && o.Nivel == Nivel;
    }

    /// <summary>
    /// Fonte da verdade: o GATT Battery Service do controle no Bluetooth LE, que da
    /// percentual exato e ainda empurra atualizacao por Notify. No cabo esse servico
    /// desaparece (o controle troca de protocolo) e nao existe percentual nenhum, entao
    /// exibimos a ultima leitura conhecida marcada como stale.
    /// </summary>
    public sealed class BatteryMonitor : IDisposable
    {
        private sealed class Reading
        {
            public int? Percent;
            public int? Nivel;
            public Precisao Precisao;
            public DateTime? At;

            /// <summary>
            /// Quando tentamos ler pela ultima vez, tendo dado certo ou nao. Separado de
            /// At porque uma tentativa frustrada nao pode se passar por leitura nova nem
            /// apagar a ultima carga conhecida -- so serve para nao repetir a tentativa
            /// a cada ciclo.
            /// </summary>
            public DateTime? Tentativa;

            /// <summary>
            /// Valor ainda nao confirmado, vindo da primeira leitura apos conectar.
            /// Enquanto estiver assim ele nao entra no historico e conta como leitura
            /// velha, para o app nao afirmar um numero que pode ser desmentido.
            /// </summary>
            public bool Provisorio;
        }

        private readonly History _history;
        private readonly KnownControllers _known;
        private readonly SemaphoreSlim _gate = new(1, 1);

        private readonly Dictionary<ulong, BluetoothLEDevice> _devices = new();
        private readonly Dictionary<string, Reading> _readings = new(StringComparer.OrdinalIgnoreCase);

        private ControllerInfo _active;
        private ulong _gattAddress;
        private GattCharacteristic _char;
        private GattDeviceService _service;

        private DateTime _lastDiscovery = DateTime.MinValue;

        /// <summary>Chaves vistas na ultima descoberta, ou seja, ligadas agora.</summary>
        private HashSet<string> _presentes = new(StringComparer.OrdinalIgnoreCase);

        /// <summary>Leitura de conexao esperando confirmacao, e desde quando.</summary>
        private (string Chave, int Percent, DateTime Quando)? _emObservacao;

        /// <summary>
        /// Quanto esperar antes de aceitar a primeira leitura de uma conexao.
        ///
        /// O controle responde a leitura inicial do GATT com um valor de espera -- 50%,
        /// no Xbox Wireless Controller -- e so manda a medida real alguns segundos
        /// depois, pelo Notify. Aceitar a primeira na hora sujava o historico e fazia o
        /// aviso de conexao mostrar um numero que o resto do app contradizia em seguida.
        /// </summary>
        private static readonly TimeSpan EsperaDeConfirmacao = TimeSpan.FromSeconds(12);

        /// <summary>Espacamento entre leituras que exigem perguntar ao dispositivo.</summary>
        private static readonly TimeSpan IntervaloSemBluetooth = TimeSpan.FromSeconds(20);
        private BatteryState _last;

        public event Action<BatteryState> Changed;

        public BatteryMonitor(History history, KnownControllers known)
        {
            _history = history;
            _known = known;

            foreach (var c in _known.Items)
            {
                var last = _history.Last(c.Key);
                if (last != null)
                    _readings[c.Key] = new Reading
                    {
                        Percent = last.P, At = last.T, Precisao = Precisao.Exata
                    };
            }
            _active = _known.Items.OrderByDescending(i => i.LastSeen).FirstOrDefault();
        }

        public BatteryState Current => _last ?? Build(LinkMode.Offline);

        // ---------- semeadura ----------

        /// <summary>Puxa do Windows a ultima leitura cacheada, util no primeiro arranque.</summary>
        public async Task SeedFromWindowsAsync()
        {
            var addresses = _known.Items.Select(i => i.Address).ToList();
            if (addresses.Count == 0) return;

            foreach (var r in await WindowsCache.ReadAsync(addresses))
            {
                var info = _known.Items.FirstOrDefault(i => i.Address == r.Address);
                if (info == null) continue;

                var existing = Get(info.Key);
                if (existing.At.HasValue && existing.At.Value >= r.When) continue;

                existing.Percent = r.Percent;
                existing.At = r.When;
                existing.Precisao = Precisao.Exata;
                _history.Add(info.Key, r.Percent, r.When);
            }
            Emit(Build(_last?.Mode ?? LinkMode.Offline));
        }

        private Reading Get(string key)
        {
            if (!_readings.TryGetValue(key, out var r))
            {
                r = new Reading();
                _readings[key] = r;
            }
            return r;
        }

        // ---------- ciclo ----------

        public async Task PollAsync()
        {
            if (!await _gate.WaitAsync(0)) return;
            try
            {
                if ((DateTime.Now - _lastDiscovery) > TimeSpan.FromSeconds(30))
                {
                    _lastDiscovery = DateTime.Now;
                    var found = await Discovery.DiscoverAsync();
                    _presentes = new HashSet<string>(
                        found.Select(f => f.Key), StringComparer.OrdinalIgnoreCase);
                    if (found.Count > 0) _known.Merge(found);
                    await EnsureDeviceObjectsAsync();
                }

                var connected = _devices.FirstOrDefault(kv =>
                {
                    try { return kv.Value.ConnectionStatus == BluetoothConnectionStatus.Connected; }
                    catch { return false; }
                });

                LinkMode mode;
                if (connected.Value != null)
                {
                    var info = _known.Items.FirstOrDefault(i => i.Address == connected.Key);
                    if (info != null) { _active = info; _known.Touch(info.Address); }
                    await EnsureCharacteristicAsync(connected.Key);
                    ConfirmarLeituraEmObservacao();
                    mode = LinkMode.Bluetooth;
                }
                else
                {
                    DropGatt();
                    _emObservacao = null;
                    bool wired = IsWired();

                    // Sem cabo e sem Bluetooth, o XInput ainda enxergar um controle so
                    // pode significar ligacao sem fio propria -- dongle ou adaptador.
                    // Antes isto virava "desconectado" e o app nem tentava ler a carga.
                    mode = wired ? LinkMode.Cable
                         : Native.AnyControllerPresent() ? LinkMode.Wireless
                         : LinkMode.Offline;

                    // quem esta na tomada agora vem antes de quem so foi visto um dia
                    var best = _known.Items
                        .OrderByDescending(i => _presentes.Contains(i.Key))
                        .ThenByDescending(i => i.LastSeen)
                        .FirstOrDefault();
                    if (best != null) _active = best;
                    else if (wired) _active = SyntheticWired();

                    // sem Bluetooth ainda pode haver leitura: dongle e cabo tem
                    // caminhos proprios, so que piores
                    if (_active != null && mode != LinkMode.Offline)
                        await LerSemBluetoothAsync(_active);
                    // controle so-cabo que foi desplugado: sem endereco nao ha o que lembrar,
                    // entao limpamos para nao continuar exibindo o nome de quem ja saiu
                    else if (_active != null && _active.Address == 0) _active = null;
                }

                Emit(Build(mode));
            }
            catch { /* nunca deixar o timer morrer */ }
            finally { _gate.Release(); }
        }

        private async Task EnsureDeviceObjectsAsync()
        {
            foreach (var info in _known.Items.ToList())
            {
                if (info.Address == 0 || _devices.ContainsKey(info.Address)) continue;
                try
                {
                    var dev = await BluetoothLEDevice.FromBluetoothAddressAsync(info.Address);
                    if (dev == null) continue;
                    dev.ConnectionStatusChanged += OnConnectionStatusChanged;
                    _devices[info.Address] = dev;
                }
                catch { }
            }
        }

        private void OnConnectionStatusChanged(BluetoothLEDevice sender, object args)
        {
            if (sender.ConnectionStatus == BluetoothConnectionStatus.Disconnected) DropGatt();
            _ = PollAsync();
        }

        /// <summary>
        /// XInput distingue ligacao com fio da ligacao a bateria, entao ele decide primeiro.
        /// Para controles que o XInput nao enxerga, cai no Gaming.Input: um controle visivel
        /// ali que nao esta reportando bateria so pode estar no cabo.
        /// </summary>
        private static bool IsWired()
        {
            if (Native.AnyControllerWired()) return true;
            if (Native.AnyControllerBatteryPowered()) return false;
            try { return RawGameController.RawGameControllers.Count > 0; }
            catch { return false; }
        }

        /// <summary>
        /// Controle no cabo que nunca apareceu por Bluetooth: nao temos endereco nem historico,
        /// mas o Gaming.Input ainda sabe o nome dele, o que ja e melhor que "desconhecido".
        /// </summary>
        private static ControllerInfo SyntheticWired()
        {
            string name = null;
            try
            {
                foreach (var raw in RawGameController.RawGameControllers)
                {
                    if (string.IsNullOrWhiteSpace(raw.DisplayName)) continue;
                    name = raw.DisplayName;
                    break;
                }
            }
            catch { }

            return new ControllerInfo
            {
                Address = 0,
                Name = string.IsNullOrWhiteSpace(name) ? "Controle com fio" : name,
                LastSeen = DateTime.Now
            };
        }

        private static bool DetectCharging()
        {
            try
            {
                foreach (var raw in RawGameController.RawGameControllers)
                {
                    if (raw is not IGameControllerBatteryInfo info) continue;
                    var report = info.TryGetBatteryReport();
                    if (report == null) continue;
                    if (report.Status == Windows.System.Power.BatteryStatus.Charging) return true;
                    if (report.ChargeRateInMilliwatts is int rate && rate > 0) return true;
                }
            }
            catch { }
            return false;
        }

        // ---------- GATT ----------

        private async Task EnsureCharacteristicAsync(ulong address)
        {
            if (_char != null && _gattAddress == address) return;
            if (_char != null) DropGatt();
            if (!_devices.TryGetValue(address, out var dev)) return;

            GattDeviceServicesResult svc;
            try { svc = await dev.GetGattServicesForUuidAsync(GattServiceUuids.Battery, BluetoothCacheMode.Uncached); }
            catch { return; }
            if (svc.Status != GattCommunicationStatus.Success || svc.Services.Count == 0) return;

            _service = svc.Services[0];
            GattCharacteristicsResult chars;
            try
            {
                chars = await _service.GetCharacteristicsForUuidAsync(
                    GattCharacteristicUuids.BatteryLevel, BluetoothCacheMode.Uncached);
            }
            catch { return; }
            if (chars.Status != GattCommunicationStatus.Success || chars.Characteristics.Count == 0) return;

            var ch = chars.Characteristics[0];

            try
            {
                var read = await ch.ReadValueAsync(BluetoothCacheMode.Uncached);
                if (read.Status == GattCommunicationStatus.Success)
                    Record(address, ReadByte(read.Value), provisorio: true);
            }
            catch { }

            if (ch.CharacteristicProperties.HasFlag(GattCharacteristicProperties.Notify))
            {
                ch.ValueChanged += OnValueChanged;
                try
                {
                    await ch.WriteClientCharacteristicConfigurationDescriptorAsync(
                        GattClientCharacteristicConfigurationDescriptorValue.Notify);
                }
                catch { }
            }

            _char = ch;
            _gattAddress = address;
        }

        private void OnValueChanged(GattCharacteristic sender, GattValueChangedEventArgs args)
        {
            Record(_gattAddress, ReadByte(args.CharacteristicValue));
            Emit(Build(LinkMode.Bluetooth));
        }

        private static byte ReadByte(IBuffer buf) => DataReader.FromBuffer(buf).ReadByte();

        /// <summary>
        /// Tenta as vias que nao dependem de Bluetooth, da melhor para a pior: o proprio
        /// HID do controle, a carga que o Windows mantem para o dispositivo e, por
        /// ultimo, os quatro degraus do XInput.
        ///
        /// E espacado no tempo de proposito. Ao contrario do GATT, que avisa sozinho
        /// quando muda, estas vias exigem perguntar -- e abrir o dispositivo HID a cada
        /// dois segundos seria trabalho constante para um numero que muda devagar.
        /// </summary>
        private async Task LerSemBluetoothAsync(ControllerInfo info)
        {
            var r = Get(info.Key);
            if (r.Tentativa.HasValue && (DateTime.Now - r.Tentativa.Value) < IntervaloSemBluetooth) return;
            r.Tentativa = DateTime.Now;

            // da via mais precisa para a menos: percentual do proprio HID, percentual
            // que o Windows guarda, degrau do slot certo do XInput, degrau de qualquer
            // slot, e por fim adivinhar pelo nome do dispositivo
            var leitura = await BatteryReaders.LerHidAsync(info.HidId);
            if (!leitura.Tem) leitura = await BatteryReaders.LerPropriedadeDoWindowsAsync(info.InstanceId);
            if (!leitura.Tem && info.XInputSlot >= 0) leitura = BatteryReaders.LerXInputDoSlot(info.XInputSlot);
            if (!leitura.Tem) leitura = BatteryReaders.LerXInput();
            if (!leitura.Tem) leitura = await BatteryReaders.ProcurarCargaDeControleAsync();

            if (!leitura.Tem) return;

            // Controle de Bluetooth tem fonte melhor que qualquer uma daqui. Num
            // intervalo em que o GATT nao respondeu, aceitar o degrau do XInput no lugar
            // trocaria "69%" por "carga cheia" -- perder precisao que ja se tinha.
            if (leitura.Precisao == Precisao.Aproximada
                && r.Precisao == Precisao.Exata && info.Address != 0) return;

            r.At = DateTime.Now;
            r.Precisao = leitura.Precisao;
            if (leitura.Precisao == Precisao.Exata)
            {
                r.Percent = leitura.Valor;
                r.Nivel = null;
                _history.Add(info.Key, leitura.Valor, r.At.Value);
            }
            else
            {
                // degrau nao vira historico: a serie ficaria com saltos artificiais
                r.Nivel = leitura.Valor;
                r.Percent = null;
            }
        }

        private void Record(ulong address, int percent, bool provisorio = false)
        {
            if (percent < 0 || percent > 100) return;
            var info = _known.Items.FirstOrDefault(i => i.Address == address);
            string key = info?.Key ?? address.ToString("x12");

            var r = Get(key);

            if (provisorio)
            {
                _emObservacao = (key, percent, DateTime.Now);

                // Havendo leitura anterior confiavel, ela continua no ar: trocar por um
                // valor que pode ser desmentido em segundos e o que fazia o numero
                // piscar. Sem nada anterior, mostrar o provisorio ainda e melhor que um
                // tracinho -- so que marcado, para o app nao afirmar o que nao sabe.
                if (r.Precisao == Precisao.Exata && r.Percent.HasValue) return;

                r.Percent = percent;
                r.Nivel = null;
                r.Precisao = Precisao.Exata;
                r.At = DateTime.Now;
                r.Provisorio = true;
                return;
            }

            _emObservacao = null;
            r.Percent = percent;
            r.Nivel = null;
            r.Precisao = Precisao.Exata;
            r.At = DateTime.Now;
            r.Provisorio = false;
            _history.Add(key, percent, r.At.Value);
        }

        /// <summary>
        /// Aceita a leitura de conexao que ninguem desmentiu dentro da janela de espera.
        /// Sem isto, controle que nao usa Notify nunca teria carga registrada.
        /// </summary>
        private void ConfirmarLeituraEmObservacao()
        {
            if (_emObservacao == null) return;
            var (chave, percent, quando) = _emObservacao.Value;
            if ((DateTime.Now - quando) < EsperaDeConfirmacao) return;

            _emObservacao = null;
            var r = Get(chave);
            r.Percent = percent;
            r.Nivel = null;
            r.Precisao = Precisao.Exata;
            r.At = DateTime.Now;
            r.Provisorio = false;
            _history.Add(chave, percent, r.At.Value);
        }

        private void DropGatt()
        {
            try { if (_char != null) _char.ValueChanged -= OnValueChanged; } catch { }
            _char = null;
            _gattAddress = 0;
            try { _service?.Dispose(); } catch { }
            _service = null;
        }

        // ---------- estado ----------

        private BatteryState Build(LinkMode mode)
        {
            var info = _active;
            string key = info?.Key ?? "wired";
            var r = Get(key);

            // leitura recente por qualquer via conta como ao vivo, nao so o GATT
            bool live = r.At.HasValue && !r.Provisorio &&
                        (mode == LinkMode.Bluetooth ||
                         (DateTime.Now - r.At.Value) < IntervaloSemBluetooth * 2);

            return new BatteryState
            {
                Mode = mode,
                Percent = r.Percent,
                Precisao = r.Precisao,
                Nivel = r.Nivel,
                ReadAt = r.At,
                Charging = mode == LinkMode.Cable && DetectCharging(),
                Stale = !live,
                DeviceName = info?.Name ?? "Nenhum controle pareado",
                Address = info?.PrettyAddress,
                Key = key,
                KnownCount = _known.Items.Count
            };
        }

        private void Emit(BatteryState s)
        {
            if (s.SameAs(_last)) { _last = s; return; }
            _last = s;
            Changed?.Invoke(s);
        }

        public void Dispose()
        {
            DropGatt();
            foreach (var d in _devices.Values)
            {
                try { d.ConnectionStatusChanged -= OnConnectionStatusChanged; d.Dispose(); } catch { }
            }
            _devices.Clear();
            _history.Save();
            _known.Save();
        }
    }
}


