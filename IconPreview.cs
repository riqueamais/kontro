using System;
using System.Collections.Generic;
using D = System.Drawing;

namespace Kontro
{
    /// <summary>
    /// Gera uma tira com o icone da bandeja em varios estados e tamanhos. Existe so para
    /// conferir o desenho, que na bandeja fica pequeno demais para avaliar a olho.
    /// </summary>
    internal static class IconPreview
    {
        internal static void Write(string path)
        {
            var states = new List<(string Label, BatteryState State)>
            {
                ("100", New(100, LinkMode.Bluetooth)),
                ("77",  New(77,  LinkMode.Bluetooth)),
                ("45",  New(45,  LinkMode.Bluetooth)),
                ("20",  New(20,  LinkMode.Bluetooth)),
                ("8",   New(8,   LinkMode.Bluetooth)),
                ("cabo", New(77, LinkMode.Cable)),
                ("off", New(null, LinkMode.Offline)),
            };

            int[] sizes = { 16, 20, 24, 32, 64 };
            const int cell = 78, headerH = 24;

            int w = cell * states.Count;
            int h = headerH + sizes.Length * cell;

            using var sheet = new D.Bitmap(w, h);
            using var g = D.Graphics.FromImage(sheet);
            g.Clear(D.Color.FromArgb(0x1E, 0x21, 0x26));

            using var font = new D.Font("Segoe UI", 11, D.FontStyle.Bold);
            using var small = new D.Font("Segoe UI", 9);
            using var white = new D.SolidBrush(D.Color.White);
            using var grey = new D.SolidBrush(D.Color.FromArgb(0x6B, 0x75, 0x80));

            for (int c = 0; c < states.Count; c++)
                g.DrawString(states[c].Label, font, white, c * cell + 8, 4);

            for (int r = 0; r < sizes.Length; r++)
            {
                int size = sizes[r];
                int y = headerH + r * cell;
                g.DrawString(size + "px", small, grey, 2, y + cell - 16);

                for (int c = 0; c < states.Count; c++)
                {
                    using var icon = TrayRenderer.Render(states[c].State, size);
                    using var bmp = icon.ToBitmap();
                    int x = c * cell + (cell - size) / 2;
                    g.DrawImageUnscaled(bmp, x, y + (cell - size) / 2 - 6);
                }
            }

            sheet.Save(path, D.Imaging.ImageFormat.Png);
        }

        private static BatteryState New(int? percent, LinkMode mode) => new()
        {
            Mode = mode,
            Percent = percent,
            ReadAt = DateTime.Now,
            DeviceName = "Controle",
            Key = "preview"
        };
    }
}


