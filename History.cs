using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;

namespace Kontro
{
    public sealed class Sample
    {
        public DateTime T { get; set; }
        public int P { get; set; }
    }

    /// <summary>
    /// Historico persistido por controle (chaveado pelo endereco) mais a estimativa de
    /// autonomia. So gravamos quando o valor muda ou a cada 10 min, senao o arquivo cresce a toa.
    /// </summary>
    public sealed class History
    {
        private static readonly string Dir = AppPaths.DataDir;
        private static readonly string FilePath = AppPaths.File("history.json");

        private readonly Dictionary<string, List<Sample>> _byDevice = new(StringComparer.OrdinalIgnoreCase);
        private DateTime _lastSave = DateTime.MinValue;

        private List<Sample> Series(string key)
        {
            if (key == null) key = "wired";
            if (!_byDevice.TryGetValue(key, out var list))
            {
                list = new List<Sample>();
                _byDevice[key] = list;
            }
            return list;
        }

        public void Load()
        {
            try
            {
                if (!File.Exists(FilePath)) return;
                var loaded = JsonSerializer.Deserialize<Dictionary<string, List<Sample>>>(File.ReadAllText(FilePath));
                if (loaded == null) return;

                var cutoff = DateTime.Now.AddDays(-30);
                foreach (var kv in loaded)
                {
                    if (kv.Value == null) continue;
                    _byDevice[kv.Key] = kv.Value.Where(s => s.T >= cutoff).OrderBy(s => s.T).ToList();
                }
            }
            catch { /* historico corrompido nao pode derrubar o app */ }
        }

        public Sample Last(string key)
        {
            var s = Series(key);
            return s.Count > 0 ? s[^1] : null;
        }

        public void Add(string key, int percent, DateTime when)
        {
            var series = Series(key);
            var last = series.Count > 0 ? series[^1] : null;

            if (last != null && when <= last.T && last.P == percent) return;

            bool changed = last == null || last.P != percent;
            bool overdue = last != null && (when - last.T) > TimeSpan.FromMinutes(10);
            if (!changed && !overdue) return;

            series.Add(new Sample { T = when, P = percent });
            series.Sort((a, b) => a.T.CompareTo(b.T));

            if ((DateTime.Now - _lastSave) > TimeSpan.FromSeconds(20)) Save();
        }

        public void Save()
        {
            try
            {
                Directory.CreateDirectory(Dir);
                File.WriteAllText(FilePath, JsonSerializer.Serialize(_byDevice));
                _lastSave = DateTime.Now;
            }
            catch { }
        }

        public IReadOnlyList<Sample> Recent(string key, TimeSpan window)
        {
            var cutoff = DateTime.Now - window;
            return Series(key).Where(s => s.T >= cutoff).ToList();
        }

        /// <summary>
        /// Amostras desde a ultima troca de pilha ou recarga. Uma subida de mais de 5 pontos
        /// entre leituras consecutivas significa carga nova: o que veio antes nao serve
        /// para estimar o consumo atual.
        /// </summary>
        private List<Sample> CurrentSession(string key)
        {
            var all = Series(key);
            if (all.Count == 0) return new List<Sample>();

            int start = 0;
            for (int i = all.Count - 1; i > 0; i--)
            {
                if (all[i].P - all[i - 1].P > 5) { start = i; break; }
                // um buraco longo tambem quebra a sessao: pode ter trocado a pilha desligado
                if (all[i].T - all[i - 1].T > TimeSpan.FromDays(2)) { start = i; break; }
            }
            return all.Skip(start).ToList();
        }

        /// <summary>
        /// Regressao linear sobre a sessao atual (ultimas 12 h) para achar o consumo em %/h
        /// e projetar quanto falta. Null enquanto nao houver sinal suficiente.
        /// </summary>
        public TimeSpan? EstimateRemaining(string key, int currentPercent)
        {
            var session = CurrentSession(key).Where(s => s.T >= DateTime.Now.AddHours(-12)).ToList();
            if (session.Count < 3) return null;
            if ((session[^1].T - session[0].T) < TimeSpan.FromMinutes(20)) return null;
            if (session[0].P == session[^1].P) return null;

            double t0 = session[0].T.ToOADate();
            double sx = 0, sy = 0, sxx = 0, sxy = 0;
            int n = session.Count;
            foreach (var s in session)
            {
                double x = (s.T.ToOADate() - t0) * 24.0; // horas
                double y = s.P;
                sx += x; sy += y; sxx += x * x; sxy += x * y;
            }
            double denom = n * sxx - sx * sx;
            if (Math.Abs(denom) < 1e-9) return null;

            double slope = (n * sxy - sx * sy) / denom; // %/hora
            if (slope >= -0.15) return null;            // consumo baixo demais para projetar

            double hours = currentPercent / -slope;
            if (hours <= 0 || hours > 400) return null;
            return TimeSpan.FromHours(hours);
        }

        public double? DrainPerHour(string key)
        {
            var session = CurrentSession(key).Where(s => s.T >= DateTime.Now.AddHours(-12)).ToList();
            if (session.Count < 3) return null;
            double span = (session[^1].T - session[0].T).TotalHours;
            if (span < 0.34) return null;
            double drop = session[0].P - session[^1].P;
            if (drop <= 0) return null;
            return drop / span;
        }
    }
}


