import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { useConfig } from "../estado";
import { Marca } from "./Marca";

const ESPERA_DO_SNAP_MS = 400;

export function BarraDeTitulo() {
  const janela = getCurrentWindow();
  const cfg = useConfig();
  const [maximizada, setMaximizada] = useState(false);
  const snap = useRef<number | undefined>(undefined);

  useEffect(() => {
    const conferir = () => {
      void janela.isMaximized().then(setMaximizada);
    };
    conferir();
    const redimensionou = janela.onResized(conferir);
    const foco = janela.onFocusChanged(({ payload }) => {
      document.body.dataset.foco = payload ? "sim" : "nao";
    });
    return () => {
      window.clearTimeout(snap.current);
      void redimensionou.then((f) => f());
      void foco.then((f) => f());
    };
  }, [janela]);

  const fica = cfg?.CloseAction !== "Exit";
  const rotuloDoFechar = fica ? "Fechar · o Kontro continua na bandeja" : "Fechar";

  return (
    <header className="barra" data-tauri-drag-region="deep">
      <div className="marca">
        <Marca tamanho={16} />
        <span>Kontro</span>
      </div>
      <div className="botoes-da-janela">
        <button
          className="botao-janela"
          title="Minimizar"
          aria-label="Minimizar"
          onClick={() => void janela.minimize()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5 H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </button>
        <button
          className="botao-janela"
          title={maximizada ? "Restaurar" : "Maximizar"}
          aria-label={maximizada ? "Restaurar" : "Maximizar"}
          onClick={() => {
            window.clearTimeout(snap.current);
            void janela.toggleMaximize();
          }}
          onMouseEnter={() => {
            window.clearTimeout(snap.current);
            snap.current = window.setTimeout(
              () => void invoke("mostrar_snap_layouts"),
              ESPERA_DO_SNAP_MS,
            );
          }}
          onMouseLeave={() => window.clearTimeout(snap.current)}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            {maximizada ? (
              <path
                d="M2.5 2.5 V0.5 H9.5 V7.5 H7.5 M0.5 2.5 H7.5 V9.5 H0.5 Z"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
              />
            ) : (
              <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" />
            )}
          </svg>
        </button>
        <button
          className="botao-janela fechar"
          title={rotuloDoFechar}
          aria-label={rotuloDoFechar}
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
