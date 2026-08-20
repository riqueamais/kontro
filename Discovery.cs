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
        /// Identidade estavel do controle. Com Bluetooth e o endereco; sem ele, o
        /// proprio id da interface, que ja carrega fabricante, produto e uma parte
        /// especifica daquela conexao.
        /// </summary>
        public string Key => Address != 0
            ? Address.ToString("x12")
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
            var encontrados = await GamepadsHidAsync();
            if (encontrados.Count == 0) return new List<ControllerInfo>();

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
                    LastSeen = DateTime.Now
                });
            }
            return resultado;
        }

        private readonly record struct GamepadHid(
            string IdHid, string IdInstancia, string Nome, ulong Endereco);

        /// <summary>Todo dispositivo HID que se declara controle, com ou sem Bluetooth.</summary>
        private static async Task<List<GamepadHid>> GamepadsHidAsync()
        {
            var lista = new List<GamepadHid>();
            var vistos = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            ushort[] usages = { UsageGamepad, UsageJoystick, UsageMultiAxis };
            var props = new[] { "System.Devices.DeviceInstanceId" };

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
                    d.Properties.TryGetValue(props[0], out var instancia);
                    lista.Add(new GamepadHid(d.Id, instancia as string, d.Name, ExtractAddress(d.Id)));
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


