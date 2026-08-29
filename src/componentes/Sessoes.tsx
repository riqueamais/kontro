import { Sessao } from "../estado";
import { duracao, momentoRelativo } from "../formato";
import "./sessoes.css";

const QUANTAS_MOSTRAR = 5;

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
          <span className="sessao-quando">{momentoRelativo(s.inicio)}</span>
          <span className={`sessao-carga${s.ate < s.de ? " gastou" : ""}`}>
            {s.de}% <span aria-hidden="true">→</span> {s.ate}%
          </span>
          <span className="sessao-duracao">{duracao(Math.round((s.fim - s.inicio) / 60_000))}</span>
        </button>
      ))}
    </div>
  );
}

export function rotuloDaSessao(s: Sessao): string {
  return `${momentoRelativo(s.inicio)} · ${s.de}% a ${s.ate}%`;
}
