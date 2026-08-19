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

        public string Key => Address != 0 ? Address.ToString("x12") : "wired";

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
            var addresses = await GamepadAddressesAsync();
            var result = new List<ControllerInfo>();
            if (addresses.Count == 0) return result;

            IReadOnlyList<DeviceInformation> paired;
            try
            {
                paired = await DeviceInformation.FindAllAsync(
                    BluetoothLEDevice.GetDeviceSelectorFromPairingState(true));
            }
            catch { return result; }

            foreach (var info in paired)
            {
                BluetoothLEDevice dev = null;
                try { dev = await BluetoothLEDevice.FromIdAsync(info.Id); }
                catch { }
                if (dev == null) continue;

                ulong addr = dev.BluetoothAddress;
                string name = !string.IsNullOrWhiteSpace(dev.Name) ? dev.Name : info.Name;
                dev.Dispose();

                if (!addresses.Contains(addr)) continue;

                result.Add(new ControllerInfo
                {
                    Address = addr,
                    Name = string.IsNullOrWhiteSpace(name) ? "Controle" : name,
                    LastSeen = DateTime.Now
                });
            }

            return result;
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
        private static readonly string Dir =
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Kontro");
        private static readonly string FilePath = Path.Combine(Dir, "controllers.json");

        private readonly List<ControllerInfo> _items = new();

        public IReadOnlyList<ControllerInfo> Items => _items;

        public void Load()
        {
            try
            {
                if (!File.Exists(FilePath)) return;
                var loaded = JsonSerializer.Deserialize<List<ControllerInfo>>(File.ReadAllText(FilePath));
                if (loaded != null) _items.AddRange(loaded.Where(i => i != null && i.Address != 0));
            }
            catch { }
        }

        /// <summary>Funde a descoberta com o que ja era conhecido. True se algo mudou.</summary>
        public bool Merge(IEnumerable<ControllerInfo> discovered)
        {
            bool dirty = false;
            foreach (var d in discovered)
            {
                var existing = _items.FirstOrDefault(i => i.Address == d.Address);
                if (existing == null)
                {
                    _items.Add(d);
                    dirty = true;
                }
                else
                {
                    if (existing.Name != d.Name) { existing.Name = d.Name; dirty = true; }
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


