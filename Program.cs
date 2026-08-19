using System;
using Velopack;

namespace Kontro
{
    internal static class Program
    {
        /// <summary>
        /// Ponto de entrada proprio, no lugar do que o WPF geraria sozinho.
        ///
        /// O Velopack precisa rodar antes de qualquer janela existir: durante a instalacao
        /// e a atualizacao o proprio executavel e chamado com argumentos especiais, e nesses
        /// casos ele executa o hook e encerra o processo. Se a UI subisse antes, o usuario
        /// veria janelas piscando no meio da instalacao.
        /// </summary>
        [STAThread]
        public static void Main(string[] args)
        {
            VelopackApp.Build()
                .SetArgs(args)
                .Run();

            var app = new App();
            app.InitializeComponent();
            app.Run();
        }
    }
}


