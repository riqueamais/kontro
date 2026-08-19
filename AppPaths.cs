using System;
using System.IO;

namespace Kontro
{
    /// <summary>
    /// Onde ficam os dados do usuario.
    ///
    /// Nao pode ser %LOCALAPPDATA%\Kontro: e exatamente ali que o instalador coloca o
    /// aplicativo. Guardar dados dentro da pasta de instalacao daria dois problemas — o
    /// instalador enxergaria a pasta ocupada e recusaria instalar, e uma atualizacao
    /// sobrescreveria a arvore levando junto configuracao e historico.
    ///
    /// Por isso os dados moram em %APPDATA% (perfil movel), separados do binario.
    /// </summary>
    internal static class AppPaths
    {
        internal static string DataDir { get; } = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Kontro");

        /// <summary>Local antigo, que colidia com a instalacao.</summary>
        private static string LegacyDir { get; } = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Kontro");

        internal static string File(string name) => Path.Combine(DataDir, name);

        /// <summary>
        /// Traz os arquivos do local antigo uma unica vez, para quem ja usava o app antes
        /// da mudanca nao perder historico. Copia em vez de mover: se a pasta antiga for a
        /// instalacao em si, apagar dela seria destrutivo.
        /// </summary>
        internal static void MigrateLegacy()
        {
            try
            {
                if (!Directory.Exists(LegacyDir)) return;
                if (string.Equals(LegacyDir, DataDir, StringComparison.OrdinalIgnoreCase)) return;

                Directory.CreateDirectory(DataDir);
                foreach (var nome in new[] { "settings.json", "history.json", "controllers.json" })
                {
                    string origem = Path.Combine(LegacyDir, nome);
                    string destino = Path.Combine(DataDir, nome);
                    if (System.IO.File.Exists(origem) && !System.IO.File.Exists(destino))
                        System.IO.File.Copy(origem, destino);
                }
            }
            catch { /* migracao e conveniencia, nunca motivo para o app nao subir */ }
        }
    }
}
