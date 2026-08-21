using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Windows.Devices.Bluetooth;
using Windows.Devices.Enumeration;
using Windows.Devices.HumanInterfaceDevice;
using Windows.Storage;

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
            await FontesDeBateriaAsync(sb);

            sb.AppendLine();
            sb.AppendLine("=== Resultado final da descoberta (o que o app mostra) ===");
            var controllers = await Discovery.DiscoverAsync();
            if (controllers.Count == 0) sb.AppendLine("   (nenhum controle ligado agora)");
            foreach (var c in controllers)
            {
                string via = c.Address != 0 ? "Bluetooth"
                    : c.XInputSlot >= 0 ? $"somente XInput, slot {c.XInputSlot}"
                    : "HID";
                sb.AppendLine($"   {c.Name}   [{c.PrettyAddress ?? via}]");
                sb.AppendLine($"      chave={c.Key}   via={via}");
                if (!string.IsNullOrEmpty(c.ContainerId)) sb.AppendLine($"      container={c.ContainerId}");
                if (!string.IsNullOrEmpty(c.HidId)) sb.AppendLine($"      hid={c.HidId}");
            }

            File.WriteAllText(path, sb.ToString(), Encoding.UTF8);
        }

        /// <summary>
        /// Lista todas as fontes de carga que este computador oferece para os controles
        /// conectados, seja qual for a forma de conexao.
        ///
        /// Existe porque a leitura principal do app depende do Bluetooth, e controle
        /// ligado por dongle de radio nao aparece como dispositivo Bluetooth. Saber o
        /// que cada caminho devolve para um controle especifico e o unico jeito de
        /// decidir se da para suporta-lo, em vez de supor.
        /// </summary>
        private static async Task FontesDeBateriaAsync(StringBuilder sb)
        {
            sb.AppendLine("=== Fontes de carga disponiveis ===");

            sb.AppendLine();
            sb.AppendLine("-- XInput (funciona em cabo, adaptador sem fio e dongle) --");
            bool achouXInput = false;
            foreach (var (slot, tipo, nivel, desc) in Native.BateriasXInput())
            {
                achouXInput = true;
                sb.AppendLine($"   slot {slot}: {desc}   (tipo={tipo} nivel={nivel} de 3)");
            }
            if (!achouXInput) sb.AppendLine("   (nenhum controle visivel ao XInput)");

            sb.AppendLine();
            sb.AppendLine("-- Propriedade de bateria do Windows, por dispositivo --");
            try
            {
                var props = new[]
                {
                    "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2",
                    "System.Devices.DeviceInstanceId"
                };
                var nos = await DeviceInformation.FindAllAsync("", props, DeviceInformationKind.Device);
                int achados = 0;
                foreach (var no in nos)
                {
                    if (!no.Properties.TryGetValue(props[0], out var val) || val == null) continue;
                    no.Properties.TryGetValue(props[1], out var id);
                    achados++;
                    sb.AppendLine($"   {no.Name}: {val}%");
                    sb.AppendLine($"      {id}");
                }
                if (achados == 0) sb.AppendLine("   (nenhum dispositivo expoe carga por esta via)");
            }
            catch (Exception ex) { sb.AppendLine("   erro: " + ex.Message); }

            sb.AppendLine();
            sb.AppendLine("-- HID: usage de carga de bateria (pagina 0x06, usage 0x20) --");
            sb.AppendLine("   E por aqui que muitos controles de dongle informam a carga.");
            ushort[] usos = { 0x05, 0x04, 0x08 };
            bool algum = false;
            foreach (var uso in usos)
            {
                IReadOnlyList<DeviceInformation> achadosHid;
                try { achadosHid = await DeviceInformation.FindAllAsync(HidDevice.GetDeviceSelector(0x01, uso)); }
                catch { continue; }

                foreach (var d in achadosHid)
                {
                    algum = true;
                    sb.AppendLine($"   {d.Name}");
                    HidDevice hid = null;
                    try { hid = await HidDevice.FromIdAsync(d.Id, FileAccessMode.Read); }
                    catch (Exception ex) { sb.AppendLine($"      nao abriu: {ex.Message}"); continue; }

                    if (hid == null)
                    {
                        // acontece quando outro processo detem acesso exclusivo
                        sb.AppendLine("      nao abriu (sem acesso)");
                        continue;
                    }

                    using (hid)
                    {
                        foreach (var (rotulo, tipo) in new[]
                        {
                            ("entrada", HidReportType.Input),
                            ("recurso", HidReportType.Feature)
                        })
                        {
                            var controles = hid.GetNumericControlDescriptions(tipo, 0x0006, 0x0020);
                            sb.AppendLine(controles.Count > 0
                                ? $"      {rotulo}: {controles.Count} controle(s) de carga -- " +
                                  string.Join(", ", controles.Select(c => $"id={c.ReportId} {c.LogicalMinimum}..{c.LogicalMaximum}"))
                                : $"      {rotulo}: sem controle de carga");
                        }
                    }
                }
            }
            if (!algum) sb.AppendLine("   (nenhum controle HID presente)");
        }
    }
}


