using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Windows.Devices.Enumeration;
using Windows.Devices.HumanInterfaceDevice;
using Windows.Storage;

namespace Kontro
{
    /// <summary>Quanto vale a leitura que se conseguiu.</summary>
    public enum Precisao
    {
        Nenhuma = 0,
        /// <summary>Quatro degraus, sem numero. E o que o XInput sabe.</summary>
        Aproximada = 1,
        /// <summary>Percentual real, de 0 a 100.</summary>
        Exata = 2
    }

    public readonly record struct Leitura(int Valor, Precisao Precisao)
    {
        public static readonly Leitura Vazia = new(0, Precisao.Nenhuma);
        public bool Tem => Precisao != Precisao.Nenhuma;
    }

    /// <summary>
    /// Vias de leitura de carga que nao passam pelo Bluetooth.
    ///
    /// O caminho principal do app e o GATT do Bluetooth LE, que da percentual exato e
    /// avisa sozinho quando muda. Controle ligado por dongle de radio nao e dispositivo
    /// Bluetooth e nunca aparece por la, entao precisa destas alternativas -- todas
    /// piores, cada uma de um jeito, e por isso a leitura carrega junto o quanto vale.
    /// </summary>
    internal static class BatteryReaders
    {
        private const ushort PaginaControlesGenericos = 0x0006;
        private const ushort UsageCargaDaBateria = 0x0020;
        private const string ChavePnpNivel = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
        private const string ChavePnpInstancia = "System.Devices.DeviceInstanceId";

        /// <summary>
        /// Le a carga pelo proprio HID, que e por onde boa parte dos controles de dongle
        /// informa. O valor vem numa escala declarada pelo dispositivo, e nao em
        /// porcentagem, entao precisa ser convertido pelos limites que ele mesmo diz.
        /// </summary>
        internal static async Task<Leitura> LerHidAsync(string idHid, CancellationToken ct = default)
        {
            if (string.IsNullOrEmpty(idHid)) return Leitura.Vazia;

            HidDevice hid = null;
            try
            {
                hid = await HidDevice.FromIdAsync(idHid, FileAccessMode.Read);
                if (hid == null) return Leitura.Vazia;   // outro processo com acesso exclusivo

                // relatorio de recurso primeiro: ele pode ser pedido a qualquer momento,
                // enquanto o de entrada so chega quando o controle manda algo
                var recurso = hid.GetNumericControlDescriptions(
                    HidReportType.Feature, PaginaControlesGenericos, UsageCargaDaBateria);
                foreach (var desc in recurso)
                {
                    try
                    {
                        var rel = await hid.GetFeatureReportAsync(desc.ReportId).AsTask(ct);
                        var ctrl = rel?.GetNumericControl(PaginaControlesGenericos, UsageCargaDaBateria);
                        if (ctrl != null) return Escalar(ctrl.Value, desc.LogicalMinimum, desc.LogicalMaximum);
                    }
                    catch { }
                }

                var entrada = hid.GetNumericControlDescriptions(
                    HidReportType.Input, PaginaControlesGenericos, UsageCargaDaBateria);
                if (entrada.Count > 0)
                {
                    // o relatorio de entrada so chega quando o controle envia algo; sem
                    // limite de espera, um controle parado deixaria a leitura pendurada
                    var leitura = hid.GetInputReportAsync(entrada[0].ReportId).AsTask(ct);
                    var tempo = Task.Delay(TimeSpan.FromSeconds(2), ct);
                    if (await Task.WhenAny(leitura, tempo) == leitura)
                    {
                        var ctrl = (await leitura)?.GetNumericControl(
                            PaginaControlesGenericos, UsageCargaDaBateria);
                        if (ctrl != null)
                            return Escalar(ctrl.Value, entrada[0].LogicalMinimum, entrada[0].LogicalMaximum);
                    }
                }
            }
            catch { }
            finally { hid?.Dispose(); }

            return Leitura.Vazia;
        }

        private static Leitura Escalar(long valor, long minimo, long maximo)
        {
            if (maximo <= minimo) return Leitura.Vazia;
            double fracao = (valor - minimo) / (double)(maximo - minimo);
            int pct = (int)Math.Round(Math.Clamp(fracao, 0, 1) * 100);
            return new Leitura(pct, Precisao.Exata);
        }

        /// <summary>
        /// Carga que o Windows mantem para o dispositivo. Funciona para varios
        /// perifericos ligados por dongle, nao so Bluetooth, desde que o driver informe.
        /// </summary>
        internal static async Task<Leitura> LerPropriedadeDoWindowsAsync(string idInstancia)
        {
            if (string.IsNullOrEmpty(idInstancia)) return Leitura.Vazia;
            try
            {
                var props = new[] { ChavePnpNivel, ChavePnpInstancia };
                var nos = await DeviceInformation.FindAllAsync("", props, DeviceInformationKind.Device);
                foreach (var no in nos)
                {
                    if (!no.Properties.TryGetValue(ChavePnpNivel, out var bruto) || bruto == null) continue;
                    if (!no.Properties.TryGetValue(ChavePnpInstancia, out var id)) continue;
                    if (id is not string texto) continue;
                    if (texto.IndexOf(idInstancia, StringComparison.OrdinalIgnoreCase) < 0) continue;

                    int pct = Convert.ToInt32(bruto);
                    if (pct is >= 0 and <= 100) return new Leitura(pct, Precisao.Exata);
                }
            }
            catch { }
            return Leitura.Vazia;
        }

        /// <summary>
        /// Ultimo recurso: os quatro degraus do XInput. Nao e percentual, e transforma-lo
        /// em um seria inventar precisao que o dado nao tem -- por isso volta marcado
        /// como aproximado, para a interface dizer "carga baixa" em vez de "25%".
        /// </summary>
        internal static Leitura LerXInput()
        {
            foreach (var (_, tipo, nivel, _) in Native.BateriasXInput())
            {
                // tipo 0 e desconectado, 1 e com fio: nenhum dos dois tem carga a informar
                if (tipo is 0 or 1) continue;
                return new Leitura(Math.Clamp((int)nivel, 0, 3), Precisao.Aproximada);
            }
            return Leitura.Vazia;
        }

        /// <summary>Carga de um slot especifico do XInput.</summary>
        internal static Leitura LerXInputDoSlot(int slot)
        {
            var info = Native.BateriaDoSlot(slot);
            if (info == null) return Leitura.Vazia;
            if (info.Value.Tipo is 0 or 1) return Leitura.Vazia;
            return new Leitura(Math.Clamp((int)info.Value.Nivel, 0, 3), Precisao.Aproximada);
        }

        /// <summary>
        /// Ultimo recurso para controle que nao e HID: procurar, entre os dispositivos
        /// cuja carga o Windows conhece, algum que se pareca com controle pelo nome.
        ///
        /// E frouxo de proposito. Sem id de instancia -- que um controle so-XInput nao
        /// nos da -- nao ha como casar com precisao, entao isto so entra depois que toda
        /// via direta falhou, e ainda assim mede o nome antes de acreditar.
        /// </summary>
        internal static async Task<Leitura> ProcurarCargaDeControleAsync()
        {
            string[] pistas =
            {
                "control", "gamepad", "joystick", "xbox", "gamesir",
                "dualsense", "dualshock", "wireless receiver"
            };
            try
            {
                var props = new[] { ChavePnpNivel, ChavePnpInstancia };
                var nos = await DeviceInformation.FindAllAsync("", props, DeviceInformationKind.Device);
                foreach (var no in nos)
                {
                    if (!no.Properties.TryGetValue(ChavePnpNivel, out var bruto) || bruto == null) continue;

                    // Dispositivo Bluetooth fica de fora, e essa e a trava que impede o
                    // erro pior possivel aqui: o Windows guarda a ultima carga de um
                    // controle pareado mesmo desligado, e sem esta linha um controle de
                    // dongle mudo herdaria a porcentagem de outro controle da casa.
                    no.Properties.TryGetValue(ChavePnpInstancia, out var id);
                    var instancia = (id as string ?? string.Empty).ToUpperInvariant();
                    if (instancia.StartsWith("BTHLE", StringComparison.Ordinal)
                        || instancia.StartsWith("BTHENUM", StringComparison.Ordinal)) continue;

                    var nome = (no.Name ?? string.Empty).ToLowerInvariant();
                    bool parece = false;
                    foreach (var pista in pistas)
                        if (nome.Contains(pista)) { parece = true; break; }
                    if (!parece) continue;

                    int pct = Convert.ToInt32(bruto);
                    if (pct is >= 0 and <= 100) return new Leitura(pct, Precisao.Exata);
                }
            }
            catch { }
            return Leitura.Vazia;
        }

        /// <summary>Texto para uma leitura aproximada, que nao tem numero para mostrar.</summary>
        internal static string DescreverNivel(int nivel) => nivel switch
        {
            0 => "quase acabando",
            1 => "carga baixa",
            2 => "carga média",
            _ => "carga cheia"
        };

        /// <summary>Quanto do anel preencher quando so ha degrau, e nao percentual.</summary>
        internal static int PreenchimentoDoNivel(int nivel) => nivel switch
        {
            0 => 10,
            1 => 35,
            2 => 65,
            _ => 100
        };
    }
}
