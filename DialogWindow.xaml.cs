using System;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media.Animation;

namespace Kontro
{
    /// <summary>
    /// Caixa de dialogo do app.
    ///
    /// Existe porque o MessageBox do Windows chega com barra de titulo clara, fonte e
    /// botoes do sistema: no meio de uma janela escura ele parece de outro programa. As
    /// perguntas que o app faz -- atualizar agora, por exemplo -- sao momentos em que a
    /// confianca importa, e nao e hora de o app parecer outro.
    /// </summary>
    public partial class DialogWindow : Window
    {
        private DialogWindow()
        {
            InitializeComponent();
            // O icone do app tem os analogicos recortados; o recurso do XAML e so o
            // corpo. Usar a mesma geometria da bandeja mantem a marca igual em todo lugar.
            Glifo.Data = ControllerGeometry.PadWithHollowSticks();

            // sem barra de titulo, arrastar so funciona se a propria janela ouvir
            Painel.MouseLeftButtonDown += (_, e) =>
            {
                if (e.ButtonState == MouseButtonState.Pressed) DragMove();
            };

            BtnPrimario.Click += (_, _) => { DialogResult = true; };
            BtnSecundario.Click += (_, _) => { DialogResult = false; };
        }

        /// <summary>Pergunta de duas saidas. True quando o usuario escolhe a acao principal.</summary>
        public static bool Perguntar(Window dono, string titulo, string texto,
                                     string acao, string cancelar = "Agora não")
        {
            var d = Montar(dono, titulo, texto);
            d.BtnPrimario.Content = acao;
            d.BtnSecundario.Content = cancelar;
            return d.ShowDialog() == true;
        }

        /// <summary>Aviso de uma saida so.</summary>
        public static void Avisar(Window dono, string titulo, string texto, string acao = "Entendi")
        {
            var d = Montar(dono, titulo, texto);
            d.BtnPrimario.Content = acao;
            d.BtnSecundario.Visibility = Visibility.Collapsed;
            d.ShowDialog();
        }

        private static DialogWindow Montar(Window dono, string titulo, string texto)
        {
            var d = new DialogWindow { TitleValue = titulo, BodyValue = texto };

            // Sem dono a janela nasceria no centro da tela principal, longe de onde o
            // usuario esta olhando, e ainda poderia sumir atras da janela que a abriu.
            if (dono != null && dono.IsVisible)
            {
                d.Owner = dono;
                d.WindowStartupLocation = WindowStartupLocation.CenterOwner;
            }
            else
            {
                d.WindowStartupLocation = WindowStartupLocation.CenterScreen;
                d.Topmost = true;
            }
            return d;
        }

        private string TitleValue { set => TitleText.Text = value; }
        private string BodyValue { set => BodyText.Text = value; }

        /// <summary>Entra crescendo de leve, no tempo de abertura do sistema de design.</summary>
        protected override void OnSourceInitialized(EventArgs e)
        {
            base.OnSourceInitialized(e);

            Opacity = 0;
            Escala.ScaleX = Escala.ScaleY = 0.96;

            var suave = new CubicEase { EasingMode = EasingMode.EaseOut };
            var t = TimeSpan.FromMilliseconds(180);

            BeginAnimation(OpacityProperty, new DoubleAnimation(1, t) { EasingFunction = suave });
            Escala.BeginAnimation(System.Windows.Media.ScaleTransform.ScaleXProperty,
                new DoubleAnimation(1, t) { EasingFunction = suave });
            Escala.BeginAnimation(System.Windows.Media.ScaleTransform.ScaleYProperty,
                new DoubleAnimation(1, t) { EasingFunction = suave });
        }
    }
}
