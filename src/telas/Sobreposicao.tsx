import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useLayoutEffect, useRef } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import {
  Estado,
  corDoAnel,
  useConfig,
  useControles,
  useEstado,
  useLimiares,
  usePilulaSolta,
} from "../estado";
import "./sobreposicao.css";

export function Sobreposicao() {
  const estado = useEstado();
  const todos = useControles();
  const cfg = useConfig();
  const limiares = useLimiares();
  const solta = usePilulaSolta();
  const raiz = useRef<HTMLDivElement>(null);
  const medida = useRef("");

  const medir = useCallback(() => {
    const alvo = raiz.current;
    if (!alvo) return;

    const caixa = alvo.getBoundingClientRect();
    const largura = Math.ceil(caixa.width);
    const altura = Math.ceil(caixa.height);
    if (largura < 40 || altura < 20) return;

    const assinatura = `${largura}x${altura}`;
    if (assinatura === medida.current) return;
    medida.current = assinatura;

    void invoke("ajustar_tamanho_da_sobreposicao", { largura, altura });
  }, []);

  useLayoutEffect(medir);

  useEffect(() => {
    const alvo = raiz.current;
    if (!alvo) return;

    const observador = new ResizeObserver(medir);
    observador.observe(alvo);
    return () => observador.disconnect();
  }, [medir]);

  if (!estado) return null;
  const escala = cfg?.OverlayScale ?? 1;
  const opacidade = cfg?.OverlayOpacity ?? 0.9;
  const acompanhantes = todos.filter((c) => c.via !== "Desligado" && c.chave !== estado.chave);
  return (
    <div
      ref={raiz}
      className={classes(solta, (cfg?.OverlayX ?? 1) > 0.5)}
      style={{ transform: `scale(${escala})`, transformOrigin: "top left" }}
      onMouseDown={(evento) => {
        if (solta && evento.button === 0) void getCurrentWindow().startDragging();
      }}
    >
      <div className="pilula" style={{ opacity: solta ? 1 : opacidade }}>
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado, limiares)}
          espessura={60}
          tamanho={30}
          girando={estado.girando}
        >
          <Glifo tamanho={15} cor="var(--text-primary)" />
        </Anel>
        <span className="valor">{resumir(estado)}</span>
        {acompanhantes.length > 0 && (
          <div className="acompanhantes">
            {acompanhantes.map((c) => (
              <div className="acompanhante" key={c.chave} title={c.nome}>
                <Anel
                  valor={c.preenchimento}
                  cor={corDoAnel(c, limiares)}
                  espessura={70}
                  tamanho={20}
                  girando={c.girando}
                >
                  <Glifo tamanho={10} cor="var(--text-secondary)" />
                </Anel>
                <span className="valor menor">{resumir(c)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
      {solta && (
        <button
          className="prender"
          title="Prender a pílula aqui"
          onMouseDown={(evento) => evento.stopPropagation()}
          onClick={() => void invoke("soltar_a_pilula", { solta: false })}
        >
          <Cadeado />
        </button>
      )}
    </div>
  );
}
function classes(solta: boolean, aDireita: boolean): string {
  return ["sobreposicao", solta && "solta", aDireita && "espelhada"].filter(Boolean).join(" ");
}

function Cadeado() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">
      <rect x="3.4" y="7" width="9.2" height="6.2" rx="1.6" fill="currentColor" />
      <path
        d="M5.8 7V5.3a2.2 2.2 0 0 1 4.4 0V7"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}

function resumir(estado: Estado): string {
  if (estado.girando) return "cabo";
  if (estado.precisao === "Aproximada" && estado.nivel !== null) {
    return ["baixa", "baixa", "média", "cheia"][Math.min(Math.max(estado.nivel, 0), 3)];
  }
  return estado.textoDaCarga;
}
