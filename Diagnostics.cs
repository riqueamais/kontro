using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using Windows.Devices.Bluetooth;
using Windows.Devices.Enumeration;
using Windows.Devices.HumanInterfaceDevice;

namespace Kontro
{
    /// <summary>
    /// Despeja o que a descoberta enxerga: quais dispositivos HID existem por usage,
    /// quais Bluetooth estao pareados e o que sobra depois do filtro. Serve para conferir
    /// que teclado, mouse e fone nao entram na lista de controles.
    /// </summary>
    internal static class Diagnostics
    {
        private static readonly (ushort Usage, string Label)[] Usages =
        {
            (0x02, "Mouse"),
            (0x04, "Joystick"),
            (0x05, "Gamepad"),
            (0x06, "Teclado"),
            (0x08, "Multi-axis"),
        };

        internal static async Task WriteAsync(string path)
        {
            var sb = new StringBuilder();
            sb.AppendLine("=== HID por usage (pagina 0x01, Generic Desktop) ===");
            sb.AppendLine("O filtro do app aceita SOMENTE Joystick, Gamepad e Multi-axis.");
            sb.AppendLine();

            foreach (var (usage, label) in Usages)
            {
                bool accepted = usage is 0x04 or 0x05 or 0x08;
                sb.AppendLine($"-- usage 0x{usage:X2} {label}  [{(accepted ? "ACEITO" : "IGNORADO")}]");

                IReadOnlyList<DeviceInformation> found;
                try
                {
                    var selector = HidDevice.GetDeviceSelector(0x01, usage);
                    found = await DeviceInformation.FindAllAsync(selector);
                }
                catch (Exception ex)
                {
                    sb.AppendLine($"   erro: {ex.Message}");
                    continue;
                }

                if (found.Count == 0) sb.AppendLine("   (nenhum)");
                foreach (var d in found)
                {
                    ulong addr = Discovery.ExtractAddress(d.Id);
                    string mac = addr == 0 ? "sem endereco BT" : addr.ToString("x12");
                    sb.AppendLine($"   {d.Name}   [{mac}]");
                }
                sb.AppendLine();
            }

            sb.AppendLine("=== Bluetooth LE pareados ===");
            try
            {
                var paired = await DeviceInformation.FindAllAsync(
                    BluetoothLEDevice.GetDeviceSelectorFromPairingState(true));
                foreach (var info in paired)
                {
                    BluetoothLEDevice dev = null;
                    try { dev = await BluetoothLEDevice.FromIdAsync(info.Id); } catch { }
                    if (dev == null) { sb.AppendLine($"   {info.Name}  (nao abriu)"); continue; }
                    sb.AppendLine($"   {dev.Name}   [{dev.BluetoothAddress:x12}]   {dev.ConnectionStatus}");
                    dev.Dispose();
                }
            }
            catch (Exception ex) { sb.AppendLine("   erro: " + ex.Message); }

            sb.AppendLine();
            sb.AppendLine("=== Resultado final da descoberta (o que o app mostra) ===");
            var controllers = await Discovery.DiscoverAsync();
            if (controllers.Count == 0) sb.AppendLine("   (nenhum controle ligado agora)");
            foreach (var c in controllers)
                sb.AppendLine($"   {c.Name}   [{c.PrettyAddress}]");

            File.WriteAllText(path, sb.ToString(), Encoding.UTF8);
        }
    }
}


