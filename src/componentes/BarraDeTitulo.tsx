import { getCurrentWindow } from "@tauri-apps/api/window";

import { Marca } from "./Marca";

/**
 * A moldura da janela, desenhada pelo app.
 *
 * A area de arrasto e declarada no HTML (`data-tauri-drag-region`) e nao em JavaScript:
 * o arrasto tem de comecar no mesmo quadro do clique, e uma ida e volta ate o Rust
 * atrasa o suficiente para a janela escapar do cursor.
 */
export function BarraDeTitulo() {
  const janela = getCurrentWindow();

  return (
    <header className="barra" data-tauri-drag-region>
      <div className="marca" data-tauri-drag-region>
        <Marca tamanho={15} />
        <span data-tauri-drag-region>Kontro</span>
      </div>

      <div className="botoes-da-janela">
        <button className="botao-janela" title="Minimizar" onClick={() => void janela.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5 H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </button>
        <button
          className="botao-janela fechar"
          title="Fechar"
          onClick={() => void janela.close()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </button>
      </div>
    </header>
  );
}
