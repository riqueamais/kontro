import { useState } from "react";

import { Sessao } from "../estado";
import { duracao, quando, taxa as porHoraEmTexto } from "../formato";
import { IconeDoJogo } from "./IconeDoJogo";
import "./sessoes.css";

const QUANTAS_DE_CADA_VEZ = 5;
const MINUTOS_PARA_TAXA = 20;
const MINUTOS_PARA_VIRAR_SESSAO = 10;

export function Sessoes({
  sessoes,
  escolhida,
  aoEscolher,
}: {
  sessoes: Sessao[];
  escolhida?: number | null;
  aoEscolher?: (s: Sessao) => void;
}) {
  const [quantas, setQuantas] = useState(QUANTAS_DE_CADA_VEZ);

  return (
    <div className="sessoes">
      <div className="sessoes-titulo">
        <span>Últimas sessões</span>
        {sessoes.length > 0 && (
          <span className="sessoes-contagem">
            {Math.min(quantas, sessoes.length)} de {sessoes.length}
          </span>
        )}
      </div>

      {sessoes.length === 0 && (
        <div className="sessoes-vazio">
          A primeira sessão aparece depois de {MINUTOS_PARA_VIRAR_SESSAO} min com o controle
          ligado.
        </div>
      )}

      {sessoes.slice(0, quantas).map((s) => {
        const porHora = taxa(s);
        const aberta = escolhida === s.inicio;
        return (
          <button
            className={`sessao${aberta ? " escolhida" : ""}`}
            key={s.inicio}
            aria-pressed={aberta}
            onClick={() => aoEscolher?.(s)}
          >
            <IconeDoJogo jogo={s.jogo} />

            <span className="sessao-texto">
              <span className="sessao-titulo">{s.jogo ?? quando(s.inicio)}</span>
              <span className="sessao-quando">
                {s.jogo && `${quando(s.inicio)} · `}
                {duracao(minutos(s))}
              </span>
            </span>

            <span className="sessao-numeros">
              <span className={`sessao-carga${s.ate < s.de ? " gastou" : ""}`}>{gasto(s)}</span>
              {porHora && <span className="sessao-taxa">{porHora}</span>}
            </span>
          </button>
        );
      })}

      {sessoes.length > quantas && (
        <button className="mais" onClick={() => setQuantas((n) => n + QUANTAS_DE_CADA_VEZ)}>
          mais {Math.min(QUANTAS_DE_CADA_VEZ, sessoes.length - quantas)}
        </button>
      )}
    </div>
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
  return porHoraEmTexto(porHora);
}

export function rotuloDaSessao(s: Sessao): string {
  return s.jogo ?? quando(s.inicio);
}
