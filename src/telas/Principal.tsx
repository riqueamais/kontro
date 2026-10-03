import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { BarraDeTitulo } from "../componentes/BarraDeTitulo";
import { useConfig } from "../estado";
import { Configuracoes } from "./Configuracoes";
import { Diario } from "./Diario";
import { Passos } from "./Passos";
import { Resumo } from "./Resumo";
import "./principal.css";

type Pagina = "resumo" | "diario" | "config";

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
    id: "diario",
    rotulo: "Diário",
    icone: (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <rect x="2" y="3.4" width="12" height="10.2" rx="1.8" fill="none" stroke="currentColor" strokeWidth="1.4" />
        <path d="M2 6.6 H14" stroke="currentColor" strokeWidth="1.4" />
        <path d="M5.4 2.2 V4.4 M10.6 2.2 V4.4" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
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

const ALTURA_DO_INDICADOR = 16;

export function Principal() {
  const cfg = useConfig();
  const [pagina, setPagina] = useState<Pagina>("resumo");
  const [passos, setPassos] = useState<boolean | null>(null);
  const [indicador, setIndicador] = useState<number | null>(null);
  const folhas = useRef<Partial<Record<Pagina, HTMLElement | null>>>({});
  const abas = useRef<Partial<Record<Pagina, HTMLButtonElement | null>>>({});
  const rolagens = useRef<Partial<Record<Pagina, number>>>({});

  const irPara = (proxima: Pagina, focar = false) => {
    const atual = folhas.current[pagina];
    if (atual) rolagens.current[pagina] = atual.scrollTop;
    setPagina(proxima);
    if (focar) abas.current[proxima]?.focus();
  };

  const irParaAgora = useRef(irPara);
  irParaAgora.current = irPara;

  useEffect(() => {
    let vivo = true;
    const parar = listen<Pagina>("kontro://abrir-aba", ({ payload }) => {
      if (vivo) irParaAgora.current(payload);
    });
    return () => {
      vivo = false;
      void parar.then((f) => f());
    };
  }, []);

  const vizinha = (passo: number) => {
    const i = PAGINAS.findIndex((p) => p.id === pagina);
    return PAGINAS[(i + passo + PAGINAS.length) % PAGINAS.length].id;
  };

  useLayoutEffect(() => {
    const folha = folhas.current[pagina];
    if (folha) folha.scrollTop = rolagens.current[pagina] ?? 0;
    const aba = abas.current[pagina];
    if (aba) setIndicador(aba.offsetTop + (aba.offsetHeight - ALTURA_DO_INDICADOR) / 2);
  }, [pagina, passos]);

  useEffect(() => {
    if (passos !== false) return;
    const aoTeclar = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.key !== "Tab") return;
      e.preventDefault();
      const foco = document.activeElement;
      const focoNaTroca =
        foco instanceof HTMLElement &&
        (foco.getAttribute("role") === "tab" || !!folhas.current[pagina]?.contains(foco));
      irPara(vizinha(e.shiftKey ? -1 : 1), focoNaTroca);
    };
    window.addEventListener("keydown", aoTeclar);
    return () => window.removeEventListener("keydown", aoTeclar);
  });

  useEffect(() => {
    if (passos === null && cfg) setPassos(!cfg.FirstRunDone);
  }, [cfg, passos]);

  const decidido = passos !== null;
  useEffect(() => {
    if (decidido) void invoke("janela_pronta", { rotulo: "principal" });
  }, [decidido]);

  if (passos === null) {
    return (
      <div className="app">
        <BarraDeTitulo />
      </div>
    );
  }

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
        <div
          className="trilho"
          role="tablist"
          aria-orientation="vertical"
          aria-label="Páginas"
          onKeyDown={(e) => {
            const destino =
              e.key === "ArrowDown"
                ? vizinha(1)
                : e.key === "ArrowUp"
                  ? vizinha(-1)
                  : e.key === "Home"
                    ? PAGINAS[0].id
                    : e.key === "End"
                      ? PAGINAS[PAGINAS.length - 1].id
                      : null;
            if (!destino) return;
            e.preventDefault();
            irPara(destino, true);
          }}
        >
          {indicador !== null && (
            <span
              className="indicador"
              aria-hidden="true"
              style={{ transform: `translateY(${indicador}px)` }}
            />
          )}
          {PAGINAS.map((p) => {
            const ativa = pagina === p.id;
            return (
              <button
                key={p.id}
                ref={(el) => {
                  abas.current[p.id] = el;
                }}
                id={`aba-${p.id}`}
                role="tab"
                aria-selected={ativa}
                aria-controls={`painel-${p.id}`}
                tabIndex={ativa ? 0 : -1}
                className={`aba${ativa ? " ativa" : ""}`}
                onClick={() => irPara(p.id)}
              >
                {p.icone}
                <span>{p.rotulo}</span>
              </button>
            );
          })}
        </div>
        <main className="pagina">
          {PAGINAS.map((p) => (
            <section
              key={p.id}
              ref={(el) => {
                folhas.current[p.id] = el;
              }}
              className="folha"
              role="tabpanel"
              id={`painel-${p.id}`}
              aria-labelledby={`aba-${p.id}`}
              hidden={pagina !== p.id}
            >
              {p.id === "resumo" && <Resumo />}
              {p.id === "diario" && <Diario ativa={pagina === "diario"} />}
              {p.id === "config" && (
                <Configuracoes ativa={pagina === "config"} aoRever={() => setPassos(true)} />
              )}
            </section>
          ))}
        </main>
      </div>
    </div>
  );
}
