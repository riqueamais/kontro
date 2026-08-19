using System;
using Microsoft.Win32;

namespace Kontro
{
    /// <summary>
    /// Inicio automatico pela chave Run do usuario atual. Nao exige privilegio de
    /// administrador, ao contrario da chave equivalente da maquina.
    ///
    /// Quando o app roda instalado pelo Velopack, o executavel fica numa pasta com o
    /// numero da versao no caminho, que muda a cada atualizacao. Por isso apontamos para
    /// o atalho estavel criado na instalacao sempre que ele existir.
    /// </summary>
    internal static class Autostart
    {
        private const string RunKey = @"Software\Microsoft\Windows\CurrentVersion\Run";
        private const string ValueName = "Kontro";

        internal static bool IsEnabled()
        {
            try
            {
                using var key = Registry.CurrentUser.OpenSubKey(RunKey);
                return key?.GetValue(ValueName) != null;
            }
            catch { return false; }
        }

        internal static void Set(bool enabled)
        {
            try
            {
                using var key = Registry.CurrentUser.CreateSubKey(RunKey);
                if (key == null) return;

                if (!enabled)
                {
                    key.DeleteValue(ValueName, false);
                    return;
                }

                string target = ResolveLaunchTarget();
                if (string.IsNullOrEmpty(target)) return;
                key.SetValue(ValueName, target);
            }
            catch { }
        }

        private static string ResolveLaunchTarget()
        {
            string exe = Environment.ProcessPath;
            if (string.IsNullOrEmpty(exe)) return null;

            // atalho do menu iniciar criado pelo instalador: sobrevive as atualizacoes
            string shortcut = System.IO.Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                @"Microsoft\Windows\Start Menu\Programs", "Kontro.lnk");

            if (System.IO.File.Exists(shortcut)) return $"\"{shortcut}\"";
            return $"\"{exe}\"";
        }
    }
}


