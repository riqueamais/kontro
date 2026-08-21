using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Windows.Devices.Bluetooth;
using Windows.Devices.Enumeration;
using Windows.Devices.HumanInterfaceDevice;

namespace Kontro
{
    public sealed class ControllerInfo
    {
        /// <summary>Endereco Bluetooth. Zero quando o controle so aparece por cabo.</summary>
        public ulong Address { get; set; }
        public string Name { get; set; }
        public DateTime LastSeen { get; set; }

        /// <summary>Interface HID do controle, usada para ler a carga sem Bluetooth.</summary>
        public string HidId { get; set; }

        /// <summary>Id do dispositivo no Windows, para consultar a carga que ele mantem.</summary>
        public string InstanceId { get; set; }

        /// <summary>
        /// Slot do XInput, quando o controle so existe por ali. Vale -1 para quem foi
        /// encontrado como dispositivo HID.
        /// </summary>
        public int XInputSlot { get; set; } = -1;

        /// <summary>
        /// Container do aparelho fisico. E o que amarra as varias interfaces do mesmo
        /// controle -- HID, XUSB, no do dispositivo -- e por isso serve para achar a
        /// carga mesmo quando ela nao esta no no que o app abriu.
        /// </summary>
        public string ContainerId { get; set; }

        /// <summary>
        /// Identidade estavel do controle. Com Bluetooth e o endereco; sem ele, o
        /// proprio id da interface, que ja carrega fabricante, produto e uma parte
        /// especifica daquela conexao.
        /// </summary>
        public string Key =>
            Address != 0 ? Address.ToString("x12")
            : XInputSlot >= 0 ? "xinput:" + XInputSlot
            : "hid:" + ChaveDoHid(HidId);

        private static string ChaveDoHid(string id)
        {
            if (string.IsNullOrEmpty(id)) return "desconhecido";
            var limpo = new System.Text.StringBuilder(id.Length);
            foreach (var c in id)
                if (char.IsLetterOrDigit(c)) limpo.Append(char.ToLowerInvariant(c));
            var texto = limpo.ToString();
            return texto.Length <= 40 ? texto : texto[^40..];
        }

        public string PrettyAddress => Address == 0
            ? null
            : string.Join(":", Enumerable.Range(0, 6)
                .Select(i => ((Address >> ((5 - i) * 8)) & 0xFF).ToString("X2")));
    }

    /// <summary>
    /// Descobre controles sem depender de marca ou modelo: pega os dispositivos HID cuja
    /// usage e gamepad ou joystick, extrai o endereco Bluetooth do id da interface e cruza
    /// com os dispositivos Bluetooth pareados. O nome exibido vem do proprio sistema.
    /// </summary>
    internal static class Discovery
    {
        private const ushort UsagePageGenericDesktop = 0x01;
        private const ushort UsageJoystick = 0x04;
        private const ushort UsageGamepad = 0x05;
        private const ushort UsageMultiAxis = 0x08;

        // um bloco de 12 hex delimitado por & ou _ dentro do id da interface HID
        private static readonly Regex AddressPattern =
            new(@"[&_]([0-9A-Fa-f]{12})[&_]", RegexOptions.Compiled);

        internal static async Task<List<ControllerInfo>> DiscoverAsync()
        {
            // Sem HID ainda pode haver controle: o XUSB e um mundo a parte, conferido
            // mais abaixo. Desistir aqui era justamente o que escondia esses controles.
            var encontrados = await GamepadsHidAsync();

            // Nomes bons vem do Bluetooth: o HID costuma devolver rotulos genericos
            // como "Controlador de jogo compativel com HID".
            var porEndereco = new Dictionary<ulong, string>();
            try
            {
                var pareados = await DeviceInformation.FindAllAsync(
                    BluetoothLEDevice.GetDeviceSelectorFromPairingState(true));
                foreach (var info in pareados)
                {
                    BluetoothLEDevice dev = null;
                    try { dev = await BluetoothLEDevice.FromIdAsync(info.Id); }
                    catch { }
                    if (dev == null) continue;
                    if (!string.IsNullOrWhiteSpace(dev.Name)) porEndereco[dev.BluetoothAddress] = dev.Name;
                    dev.Dispose();
                }
            }
            catch { }

            var resultado = new List<ControllerInfo>();
            var containersHid = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (var g in encontrados)
            {
                // sem endereco significa cabo ou dongle de radio: continua sendo um
                // controle, so nao tem leitura por Bluetooth
                string nome = g.Endereco != 0 && porEndereco.TryGetValue(g.Endereco, out var bom)
                    ? bom
                    : (string.IsNullOrWhiteSpace(g.Nome) ? "Controle" : g.Nome);

                resultado.Add(new ControllerInfo
                {
                    Address = g.Endereco,
                    Name = nome,
                    HidId = g.IdHid,
                    InstanceId = g.IdInstancia,
                    ContainerId = g.Container,
                    LastSeen = DateTime.Now
                });
                containersHid.Add(g.Container);
            }

            await AcrescentarSomenteXInputAsync(resultado, containersHid);
            return resultado;
        }

        /// <summary>Interface que o Windows publica para todo controle atendido pelo XUSB.</summary>
        private const string GuidXusb = "{EC87F1E3-C13B-4100-B5F7-8B84D54260CB}";
        private const string PropContainer = "System.Devices.ContainerId";
        private const string PropInstancia = "System.Devices.DeviceInstanceId";

        /// <summary>
        /// Acrescenta controles que existem apenas para o XInput.
        ///
        /// Quem usa o driver do Xbox 360 -- o caso dos dongles que emulam esse controle,
        /// e do que o Windows lista como "Xbox 360 for Windows" -- nao e dispositivo HID.
        /// A busca por HID passa direto por ele, e sem isto esse controle simplesmente
        /// nao existe para o app.
        ///
        /// A comparacao e por container, que e o aparelho fisico: um controle atendido
        /// por HID e por XUSB publica as duas interfaces sob o mesmo container. Container
        /// de XUSB que nao apareceu no HID e, por definicao, controle que so o XInput
        /// enxerga -- criterio que nao depende de contar dispositivos nem de adivinhar
        /// quem e quem.
        /// </summary>
        private static async Task AcrescentarSomenteXInputAsync(
            List<ControllerInfo> encontrados, HashSet<string> containersHid)
        {
            List<int> slots;
            try { slots = Native.SlotsXInputConectados(); }
            catch { return; }
            if (slots.Count == 0) return;

            var xusb = await DispositivosXusbAsync();

            List<(string Nome, string Instancia, string Container)> somente;
            if (xusb.Count == 0)
            {
                // Sem lista de XUSB nao ha container para comparar. Ainda assim, se o
                // XInput ve controle e o HID nao viu nenhum, nao existe com o que
                // confundir e os slots sao todos de controles invisiveis ao HID.
                if (encontrados.Count > 0) return;
                somente = slots.Select(_ => ((string)null, (string)null, (string)null)).ToList();
            }
            else
            {
                somente = xusb
                    .Where(x => string.IsNullOrEmpty(x.Container) || !containersHid.Contains(x.Container))
                    .Select(x => (x.Nome, x.Instancia, x.Container))
                    .ToList();
            }
            if (somente.Count == 0 || somente.Count > slots.Count) return;

            // na pratica os slots ocupados por controle HID vem primeiro na contagem do
            // XInput, entao o que sobra na ponta pertence a estes
            var meus = slots.Skip(slots.Count - somente.Count).ToList();
            var reserva = NomesDisponiveis(encontrados);

            for (int i = 0; i < somente.Count; i++)
            {
                encontrados.Add(new ControllerInfo
                {
                    Address = 0,
                    XInputSlot = meus[i],
                    InstanceId = somente[i].Instancia,
                    ContainerId = somente[i].Container,
                    Name = EscolherNome(somente[i].Nome, reserva, meus[i]),
                    LastSeen = DateTime.Now
                });
            }
        }

        /// <summary>Controles publicados pela interface do XUSB, com container e id.</summary>
        private static async Task<List<(string Container, string Nome, string Instancia)>>
            DispositivosXusbAsync()
        {
            var lista = new List<(string, string, string)>();
            try
            {
                var props = new[] { PropContainer, PropInstancia };
                var seletor = "System.Devices.InterfaceClassGuid:=\"" + GuidXusb + "\""
                            + " AND System.Devices.InterfaceEnabled:=System.StructuredQueryType.Boolean#True";
                var achados = await DeviceInformation.FindAllAsync(seletor, props);
                foreach (var d in achados)
                {
                    d.Properties.TryGetValue(PropContainer, out var container);
                    d.Properties.TryGetValue(PropInstancia, out var instancia);
                    lista.Add((Texto(container), d.Name, instancia as string));
                }
            }
            catch { }
            return lista;
        }

        /// <summary>O container vem como Guid, nao como texto, dependendo da consulta.</summary>
        private static string Texto(object valor) => valor switch
        {
            null => string.Empty,
            Guid g => g.ToString("B"),
            _ => valor.ToString()
        };

        /// <summary>
        /// O XInput nao informa nome nenhum, e o nome da interface do XUSB costuma ser
        /// generico. O Gaming.Input sabe o nome comercial, mas lista tambem os controles
        /// ja achados pelo HID -- por isso os repetidos saem antes.
        /// </summary>
        private static List<string> NomesDisponiveis(List<ControllerInfo> jaEncontrados)
        {
            var nomes = new List<string>();
            try
            {
                var usados = new HashSet<string>(
                    jaEncontrados.Select(c => c.Name ?? string.Empty), StringComparer.OrdinalIgnoreCase);
                foreach (var raw in Windows.Gaming.Input.RawGameController.RawGameControllers)
                {
                    var nome = raw.DisplayName;
                    if (string.IsNullOrWhiteSpace(nome) || !usados.Add(nome)) continue;
                    nomes.Add(nome);
                }
            }
            catch { }
            return nomes;
        }

        private static string EscolherNome(string doXusb, List<string> reserva, int slot)
        {
            if (reserva.Count > 0)
            {
                var nome = reserva[0];
                reserva.RemoveAt(0);
                return nome;
            }
            if (!string.IsNullOrWhiteSpace(doXusb)) return doXusb;
            return $"Controle {slot + 1}";
        }

        private readonly record struct GamepadHid(
            string IdHid, string IdInstancia, string Nome, ulong Endereco, string Container);

        /// <summary>Todo dispositivo HID que se declara controle, com ou sem Bluetooth.</summary>
        private static async Task<List<GamepadHid>> GamepadsHidAsync()
        {
            var lista = new List<GamepadHid>();
            var vistos = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            ushort[] usages = { UsageGamepad, UsageJoystick, UsageMultiAxis };
            var props = new[] { PropInstancia, PropContainer };

            foreach (var usage in usages)
            {
                IReadOnlyList<DeviceInformation> achados;
                try
                {
                    var seletor = HidDevice.GetDeviceSelector(UsagePageGenericDesktop, usage);
                    achados = await DeviceInformation.FindAllAsync(seletor, props);
                }
                catch { continue; }

                foreach (var d in achados)
                {
                    if (!vistos.Add(d.Id)) continue;
                    d.Properties.TryGetValue(PropInstancia, out var instancia);
                    d.Properties.TryGetValue(PropContainer, out var container);
                    lista.Add(new GamepadHid(d.Id, instancia as string, d.Name,
                        ExtractAddress(d.Id), Texto(container)));
                }
            }
            return lista;
        }

        private static async Task<HashSet<ulong>> GamepadAddressesAsync()
        {
            var set = new HashSet<ulong>();
            ushort[] usages = { UsageGamepad, UsageJoystick, UsageMultiAxis };

            foreach (var usage in usages)
            {
                string selector;
                try { selector = HidDevice.GetDeviceSelector(UsagePageGenericDesktop, usage); }
                catch { continue; }

                IReadOnlyList<DeviceInformation> found;
                try { found = await DeviceInformation.FindAllAsync(selector); }
                catch { continue; }

                foreach (var d in found)
                {
                    ulong a = ExtractAddress(d.Id);
                    if (a != 0) set.Add(a);
                }
            }
            return set;
        }

        internal static ulong ExtractAddress(string interfaceId)
        {
            if (string.IsNullOrEmpty(interfaceId)) return 0;
            foreach (Match m in AddressPattern.Matches(interfaceId))
            {
                if (!ulong.TryParse(m.Groups[1].Value,
                        System.Globalization.NumberStyles.HexNumber,
                        System.Globalization.CultureInfo.InvariantCulture, out var v))
                    continue;
                if (v != 0) return v;
            }
            return 0;
        }
    }

    /// <summary>
    /// Lembra os controles ja vistos para que continuem na lista mesmo desligados,
    /// quando nao existe nenhum HID para descobrir.
    /// </summary>
    public sealed class KnownControllers
    {
        private static readonly string Dir = AppPaths.DataDir;
        private static readonly string FilePath = AppPaths.File("controllers.json");

        private readonly List<ControllerInfo> _items = new();

        public IReadOnlyList<ControllerInfo> Items => _items;

        public void Load()
        {
            try
            {
                if (!File.Exists(FilePath)) return;
                var loaded = JsonSerializer.Deserialize<List<ControllerInfo>>(File.ReadAllText(FilePath));
                if (loaded != null) _items.AddRange(loaded.Where(i => i != null));
            }
            catch { }
        }

        /// <summary>Funde a descoberta com o que ja era conhecido. True se algo mudou.</summary>
        public bool Merge(IEnumerable<ControllerInfo> discovered)
        {
            bool dirty = false;
            foreach (var d in discovered)
            {
                var existing = _items.FirstOrDefault(i => i.Key == d.Key);
                if (existing == null)
                {
                    _items.Add(d);
                    dirty = true;
                }
                else
                {
                    if (existing.Name != d.Name) { existing.Name = d.Name; dirty = true; }
                    existing.HidId = d.HidId;
                    existing.InstanceId = d.InstanceId;
                    existing.XInputSlot = d.XInputSlot;
                    existing.ContainerId = d.ContainerId;
                    existing.LastSeen = d.LastSeen;
                }
            }
            if (dirty) Save();
            return dirty;
        }

        public void Touch(ulong address)
        {
            var it = _items.FirstOrDefault(i => i.Address == address);
            if (it != null) it.LastSeen = DateTime.Now;
        }

        public void Save()
        {
            try
            {
                Directory.CreateDirectory(Dir);
                File.WriteAllText(FilePath, JsonSerializer.Serialize(_items));
            }
            catch { }
        }
    }
}


