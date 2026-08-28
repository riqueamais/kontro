import { Sessao, diasAtras } from "../estado";
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
          <span className="sessao-quando">{quando(s.inicio)}</span>
          <span className={`sessao-carga${s.ate < s.de ? " gastou" : ""}`}>
            {s.de}% <span aria-hidden="true">→</span> {s.ate}%
          </span>
          <span className="sessao-duracao">{duracao(s.fim - s.inicio)}</span>
        </button>
      ))}
    </div>
  );
}

export function rotuloDaSessao(s: Sessao): string {
  return `${quando(s.inicio)} · ${s.de}% a ${s.ate}%`;
}

function quando(ms: number): string {
  const q = new Date(ms);
  const hora = q.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });
  const dias = diasAtras(q);

  if (dias === 0) return `hoje, ${hora}`;
  if (dias === 1) return `ontem, ${hora}`;
  return `${q.toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" })}, ${hora}`;
}

function duracao(ms: number): string {
  const minutos = Math.round(ms / 60_000);
  if (minutos < 60) return `${minutos} min`;

  const h = Math.floor(minutos / 60);
  const m = minutos % 60;
  return m > 0 ? `${h} h ${m}` : `${h} h`;
}
