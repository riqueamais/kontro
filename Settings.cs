using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Kontro
{
    public enum CloseAction
    {
        /// <summary>O X esconde a janela e o app segue vivo na bandeja.</summary>
        MinimizeToTray = 0,
        /// <summary>O X encerra o app de vez.</summary>
        Exit = 1
    }

    /// <summary>
    /// Preferencias do usuario, gravadas ao lado do historico. Toda escrita e imediata:
    /// sao poucos bytes e perder ajuste de configuracao irrita mais que o custo de I/O.
    /// </summary>
    public sealed class Settings
    {
        private static readonly string Dir = AppPaths.DataDir;
        private static readonly string FilePath = AppPaths.File("settings.json");

        private static readonly JsonSerializerOptions JsonOpts = new()
        {
            WriteIndented = true,
            Converters = { new JsonStringEnumConverter() }
        };

        public bool StartWithWindows { get; set; }
        public bool StartMinimized { get; set; } = true;
        public CloseAction CloseAction { get; set; } = CloseAction.MinimizeToTray;

        public bool NotificationsEnabled { get; set; } = true;
        public int WarnThreshold { get; set; } = 20;
        public int CriticalThreshold { get; set; } = 10;

        /// <summary>Aviso passageiro no topo da tela quando o controle conecta.</summary>
        public bool ConnectToastEnabled { get; set; } = true;

        public bool AutoCheckUpdates { get; set; } = true;
        public DateTime? LastUpdateCheck { get; set; }
        /// <summary>Versao que o usuario mandou ignorar, para nao insistir no aviso.</summary>
        public string SkippedVersion { get; set; }

        /// <summary>Falso ate a janela de boas-vindas ser concluida uma vez.</summary>
        public bool FirstRunDone { get; set; }

        [JsonIgnore]
        public bool IsFirstRun => !FirstRunDone;

        public static Settings Load()
        {
            try
            {
                if (File.Exists(FilePath))
                {
                    var s = JsonSerializer.Deserialize<Settings>(File.ReadAllText(FilePath), JsonOpts);
                    if (s != null) return s.Sanitized();
                }
            }
            catch { /* config corrompida volta ao padrao, nao derruba o app */ }
            return new Settings();
        }

        public void Save()
        {
            try
            {
                Directory.CreateDirectory(Dir);
                File.WriteAllText(FilePath, JsonSerializer.Serialize(Sanitized(), JsonOpts));
            }
            catch { }
        }

        internal static string SettingsPath => FilePath;

        private Settings Sanitized()
        {
            // limiares invertidos ou fora de faixa quebrariam a logica de aviso
            WarnThreshold = Math.Clamp(WarnThreshold, 5, 90);
            CriticalThreshold = Math.Clamp(CriticalThreshold, 1, 50);
            if (CriticalThreshold >= WarnThreshold) CriticalThreshold = Math.Max(1, WarnThreshold - 5);
            return this;
        }
    }
}


