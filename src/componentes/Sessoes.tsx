import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { Sessao } from "../estado";
import { duracao, momentoRelativo } from "../formato";
import { Glifo } from "./Glifo";
import "./sessoes.css";

const QUANTAS_MOSTRAR = 5;
const MINUTOS_PARA_TAXA = 20;

export function Sessoes({
  sessoes,
  escolhida,
  aoEscolher,
}: {
  sessoes: Sessao[];
  escolhida?: number | null;
  aoEscolher?: (s: Sessao) => void;
}) {
  if (sessoes.length === 0) return null;

  return (
    <div className="sessoes">
      <div className="sessoes-titulo">Últimas sessões</div>
      {sessoes.slice(0, QUANTAS_MOSTRAR).map((s) => (
        <button
          className={`sessao${escolhida === s.inicio ? " escolhida" : ""}`}
          key={s.inicio}
          title="Ver esta sessão no gráfico"
          onClick={() => aoEscolher?.(s)}
        >
          <Icone jogo={s.jogo} />

          <span className="sessao-texto">
            <span className="sessao-titulo">{s.jogo ?? momentoRelativo(s.inicio)}</span>
            <span className="sessao-quando">
              {s.jogo && `${momentoRelativo(s.inicio)} · `}
              {duracao(minutos(s))}
            </span>
          </span>

          <span className="sessao-numeros">
            <span className={`sessao-carga${s.ate < s.de ? " gastou" : ""}`}>
              {gasto(s)}
            </span>
            <span className="sessao-taxa">{taxa(s)}</span>
          </span>
        </button>
      ))}
    </div>
  );
}

function Icone({ jogo }: { jogo: string | null }) {
  const [uri, setUri] = useState<string | null>(null);

  useEffect(() => {
    if (!jogo) {
      setUri(null);
      return;
    }
    let vivo = true;
    invoke<string | null>("icone_do_jogo", { nome: jogo })
      .then((achado) => {
        if (vivo) setUri(achado);
      })
      .catch(() => {});
    return () => {
      vivo = false;
    };
  }, [jogo]);

  return (
    <span className="sessao-icone">
      {uri ? (
        <img src={uri} alt="" width={24} height={24} />
      ) : (
        <Glifo tamanho={20} cor="var(--text-tertiary)" />
      )}
    </span>
  );
}

function minutos(s: Sessao): number {
  return Math.round((s.fim - s.inicio) / 60_000);
}

function gasto(s: Sessao): string {
  const delta = s.ate - s.de;
  if (delta === 0) return "0%";
  return `${delta > 0 ? "+" : "−"}${Math.abs(delta)}%`;
}

function taxa(s: Sessao): string {
  const gastou = s.de - s.ate;
  const min = minutos(s);
  if (gastou <= 0 || min < MINUTOS_PARA_TAXA) return "";
  const porHora = (gastou * 60) / min;
  return `${porHora.toFixed(1).replace(".", ",")} %/h`;
}

export function rotuloDaSessao(s: Sessao): string {
  const quando = momentoRelativo(s.inicio);
  const carga = `${s.de}% a ${s.ate}%`;
  return s.jogo ? `${s.jogo} · ${quando} · ${carga}` : `${quando} · ${carga}`;
}
