import { useEffect, useState } from "react";

import { BarraDeTitulo } from "../componentes/BarraDeTitulo";
import { useConfig } from "../estado";
import { Configuracoes } from "./Configuracoes";
import { Passos } from "./Passos";
import { Resumo } from "./Resumo";
import "./principal.css";

type Pagina = "resumo" | "config";

const PAGINAS: { id: Pagina; rotulo: string; icone: React.ReactNode }[] = [
  {
    id: "resumo",
    rotulo: "Resumo",
    icone: (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" strokeWidth="1.4" />
        <path
          d="M8 4.4 V8 L10.4 9.6"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinecap="round"
        />
      </svg>
    ),
  },
  {
    id: "config",
    rotulo: "Configurações",
    icone: (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="8" cy="8" r="2.2" fill="none" stroke="currentColor" strokeWidth="1.4" />
        <path
          d="M8 1.4 V3 M8 13 V14.6 M1.4 8 H3 M13 8 H14.6 M3.3 3.3 L4.5 4.5 M11.5 11.5 L12.7 12.7 M12.7 3.3 L11.5 4.5 M4.5 11.5 L3.3 12.7"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinecap="round"
        />
      </svg>
    ),
  },
];

export function Principal() {
  const cfg = useConfig();
  const [pagina, setPagina] = useState<Pagina>("resumo");
  const [passos, setPassos] = useState<boolean | null>(null);

  useEffect(() => {
    if (passos === null && cfg) setPassos(!cfg.FirstRunDone);
  }, [cfg, passos]);

  if (passos) {
    return (
      <div className="app">
        <BarraDeTitulo />
        <Passos aoTerminar={() => setPassos(false)} />
      </div>
    );
  }

  return (
    <div className="app">
      <BarraDeTitulo />
      <div className="corpo">
        <nav className="trilho">
          {PAGINAS.map((p) => (
            <button
              key={p.id}
              className={`aba${pagina === p.id ? " ativa" : ""}`}
              aria-current={pagina === p.id}
              onClick={() => setPagina(p.id)}
            >
              {p.icone}
              <span>{p.rotulo}</span>
            </button>
          ))}
        </nav>
        <main className="pagina">
          {pagina === "resumo" ? <Resumo /> : <Configuracoes aoRever={() => setPassos(true)} />}
        </main>
      </div>
    </div>
  );
}
