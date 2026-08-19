using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Windows.Devices.Enumeration;

namespace Kontro
{
    /// <summary>
    /// O Windows guarda a ultima leitura de bateria de cada dispositivo Bluetooth numa
    /// propriedade do no PnP. Isso da um valor util ja no primeiro arranque, antes de
    /// existir historico proprio. A propriedade vive no no do dispositivo (BTHLE\...),
    /// nao no endpoint Bluetooth, por isso a enumeracao usa DeviceInformationKind.Device.
    /// </summary>
    internal static class WindowsCache
    {
        private const string KeyLevel = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
        private const string KeyStamp = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 7";
        private const string KeyInstanceId = "System.Devices.DeviceInstanceId";

        internal readonly record struct Reading(ulong Address, int Percent, DateTime When);

        internal static async Task<List<Reading>> ReadAsync(IEnumerable<ulong> addresses)
        {
            var wanted = new Dictionary<string, ulong>(StringComparer.OrdinalIgnoreCase);
            foreach (var a in addresses)
                if (a != 0) wanted[a.ToString("x12")] = a;

            var result = new List<Reading>();
            if (wanted.Count == 0) return result;

            try
            {
                var props = new[] { KeyLevel, KeyStamp, KeyInstanceId };
                var nodes = await DeviceInformation.FindAllAsync("", props, DeviceInformationKind.Device);

                foreach (var node in nodes)
                {
                    if (!node.Properties.TryGetValue(KeyLevel, out var raw) || raw == null) continue;
                    if (!node.Properties.TryGetValue(KeyInstanceId, out var idObj)) continue;

                    string instanceId = idObj as string;
                    if (string.IsNullOrEmpty(instanceId)) continue;

                    ulong match = 0;
                    foreach (var kv in wanted)
                    {
                        if (instanceId.IndexOf(kv.Key, StringComparison.OrdinalIgnoreCase) >= 0)
                        {
                            match = kv.Value;
                            break;
                        }
                    }
                    if (match == 0) continue;

                    int percent;
                    try { percent = Convert.ToInt32(raw); }
                    catch { continue; }
                    if (percent < 0 || percent > 100) continue;

                    var when = DateTime.Now;
                    if (node.Properties.TryGetValue(KeyStamp, out var stamp) && stamp is DateTimeOffset dto)
                        when = dto.LocalDateTime;

                    result.Add(new Reading(match, percent, when));
                }
            }
            catch { }

            return result;
        }
    }
}


