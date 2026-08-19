using System;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Reflection;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Velopack;
using Velopack.Sources;

namespace Kontro
{
    public sealed class UpdateCheckResult
    {
        public bool HasUpdate { get; init; }
        public string Version { get; init; }
        public string Message { get; init; }
        public string ReleaseUrl { get; init; }
        /// <summary>Preenchido apenas quando o app roda instalado e a atualizacao pode ser aplicada.</summary>
        internal UpdateInfo Info { get; init; }
        public bool CanApply => Info != null;
    }

    /// <summary>
    /// Verificacao de versao contra as releases do GitHub.
    ///
    /// Instalado pelo Setup, o Velopack cuida de tudo: baixa o delta, troca os arquivos
    /// com seguranca e reinicia. Rodando solto (build local ou copia portatil) nao existe
    /// estrutura de update, entao consultamos a API do GitHub so para avisar que saiu
    /// versao nova, sem prometer instalar.
    /// </summary>
    internal static class Updater
    {
        internal const string Owner = "riqueamais";
        internal const string Repo = "kontro";
        internal const string RepoUrl = "https://github.com/" + Owner + "/" + Repo;

        private static readonly HttpClient Http = CreateClient();

        private static HttpClient CreateClient()
        {
            var c = new HttpClient { Timeout = TimeSpan.FromSeconds(25) };
            c.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue("Kontro", "1.0"));
            c.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));
            return c;
        }

        /// <summary>Versao do assembly, sem o sufixo de metadados que o build anexa.</summary>
        internal static string CurrentVersion
        {
            get
            {
                var raw = Assembly.GetExecutingAssembly()
                    .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
                    ?? Assembly.GetExecutingAssembly().GetName().Version?.ToString()
                    ?? "0.0.0";
                int plus = raw.IndexOf('+');
                return plus > 0 ? raw[..plus] : raw;
            }
        }

        private static UpdateManager TryCreateManager()
        {
            try
            {
                var mgr = new UpdateManager(new GithubSource(RepoUrl, null, false));
                return mgr.IsInstalled ? mgr : null;
            }
            catch { return null; }
        }

        internal static bool IsInstalled => TryCreateManager() != null;

        internal static async Task<UpdateCheckResult> CheckAsync(CancellationToken ct = default)
        {
            var mgr = TryCreateManager();
            if (mgr != null)
            {
                var info = await mgr.CheckForUpdatesAsync().ConfigureAwait(false);
                if (info == null)
                {
                    return new UpdateCheckResult
                    {
                        HasUpdate = false,
                        Message = $"Você está na versão mais recente ({CurrentVersion})."
                    };
                }

                string v = info.TargetFullRelease.Version.ToString();
                return new UpdateCheckResult
                {
                    HasUpdate = true,
                    Version = v,
                    Info = info,
                    ReleaseUrl = $"{RepoUrl}/releases/tag/v{v}",
                    Message = $"Versão {v} disponível."
                };
            }

            return await CheckViaGithubApiAsync(ct).ConfigureAwait(false);
        }

        private static async Task<UpdateCheckResult> CheckViaGithubApiAsync(CancellationToken ct)
        {
            string url = $"https://api.github.com/repos/{Owner}/{Repo}/releases/latest";
            using var resp = await Http.GetAsync(url, ct).ConfigureAwait(false);

            if (resp.StatusCode == System.Net.HttpStatusCode.NotFound)
            {
                return new UpdateCheckResult
                {
                    HasUpdate = false,
                    Message = "Nenhuma release publicada ainda."
                };
            }
            resp.EnsureSuccessStatusCode();

            using var doc = JsonDocument.Parse(await resp.Content.ReadAsStringAsync(ct).ConfigureAwait(false));
            var root = doc.RootElement;

            string tag = root.TryGetProperty("tag_name", out var t) ? t.GetString() : null;
            string html = root.TryGetProperty("html_url", out var h) ? h.GetString() : RepoUrl;
            if (string.IsNullOrWhiteSpace(tag))
                return new UpdateCheckResult { HasUpdate = false, Message = "Não foi possível ler a última release." };

            string remote = tag.TrimStart('v', 'V');
            bool newer = IsNewer(remote, CurrentVersion);

            return new UpdateCheckResult
            {
                HasUpdate = newer,
                Version = remote,
                ReleaseUrl = html,
                Message = newer
                    ? $"Versão {remote} disponível no GitHub. Esta cópia não se atualiza sozinha."
                    : $"Você está na versão mais recente ({CurrentVersion})."
            };
        }

        /// <summary>Compara so a parte numerica; um sufixo de pre-lancamento perde do lancamento final.</summary>
        internal static bool IsNewer(string candidate, string current)
        {
            static (Version V, bool Pre) Parse(string s)
            {
                if (string.IsNullOrWhiteSpace(s)) return (new Version(0, 0, 0), false);
                int dash = s.IndexOfAny(new[] { '-', '+' });
                bool pre = dash > 0;
                string numeric = pre ? s[..dash] : s;
                var parts = numeric.Split('.').Where(p => int.TryParse(p, out _)).Select(int.Parse).ToArray();
                return (new Version(
                    parts.Length > 0 ? parts[0] : 0,
                    parts.Length > 1 ? parts[1] : 0,
                    parts.Length > 2 ? parts[2] : 0), pre);
            }

            var a = Parse(candidate);
            var b = Parse(current);
            int cmp = a.V.CompareTo(b.V);
            if (cmp != 0) return cmp > 0;
            // mesma versao numerica: o final ganha do pre-lancamento
            return b.Pre && !a.Pre;
        }

        /// <summary>Baixa e aplica. Se der certo o processo reinicia e este metodo nao retorna.</summary>
        internal static async Task ApplyAsync(UpdateCheckResult result, Action<int> progress = null)
        {
            if (result?.Info == null) return;
            var mgr = TryCreateManager();
            if (mgr == null) return;

            await mgr.DownloadUpdatesAsync(result.Info, progress).ConfigureAwait(false);
            mgr.ApplyUpdatesAndRestart(result.Info.TargetFullRelease);
        }
    }
}


