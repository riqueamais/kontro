using System.Drawing;
using System.Windows.Forms;

namespace Kontro
{
    /// <summary>
    /// O menu da bandeja e WinForms, entao o tema em XAML nao o alcanca. Este desenhista
    /// aplica os mesmos tokens a mao: sem ele o menu apareceria cinza-claro do sistema,
    /// destoando de todo o resto do app.
    /// </summary>
    internal sealed class TrayMenuRenderer : ToolStripProfessionalRenderer
    {
        internal static readonly Color SurfaceAlt = Color.FromArgb(0x15, 0x1A, 0x20);
        internal static readonly Color Stroke = Color.FromArgb(0x1E, 0x25, 0x2C);
        internal static readonly Color StrokeStrong = Color.FromArgb(0x2A, 0x33, 0x3B);
        internal static readonly Color TextPrimary = Color.FromArgb(0xE8, 0xEC, 0xEF);
        internal static readonly Color TextTertiary = Color.FromArgb(0x6B, 0x75, 0x7D);

        internal TrayMenuRenderer() : base(new Cores()) { }

        private sealed class Cores : ProfessionalColorTable
        {
            public Cores() { UseSystemColors = false; }

            public override Color ToolStripDropDownBackground => SurfaceAlt;
            public override Color ImageMarginGradientBegin => SurfaceAlt;
            public override Color ImageMarginGradientMiddle => SurfaceAlt;
            public override Color ImageMarginGradientEnd => SurfaceAlt;
            public override Color MenuBorder => Stroke;
            public override Color MenuItemBorder => StrokeStrong;
            public override Color MenuItemSelected => StrokeStrong;
            public override Color MenuItemSelectedGradientBegin => StrokeStrong;
            public override Color MenuItemSelectedGradientEnd => StrokeStrong;
            public override Color MenuItemPressedGradientBegin => SurfaceAlt;
            public override Color MenuItemPressedGradientEnd => SurfaceAlt;
            public override Color SeparatorDark => Stroke;
            public override Color SeparatorLight => Stroke;
        }

        protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
        {
            // o primeiro item e o estado atual: fica desabilitado de proposito, como
            // cabecalho, e por isso precisa de cor propria em vez do cinza do sistema
            e.TextColor = e.Item.Enabled ? TextPrimary : TextTertiary;
            base.OnRenderItemText(e);
        }

        protected override void OnRenderSeparator(ToolStripSeparatorRenderEventArgs e)
        {
            using var caneta = new Pen(Stroke);
            int y = e.Item.Height / 2;
            e.Graphics.DrawLine(caneta, 8, y, e.Item.Width - 8, y);
        }
    }
}
