import { Amostra } from "../estado";
import "./historico.css";

const HORA = 3_600_000;
const DIA = 24 * HORA;

export function Historico({
  serie,
  cor,
  altura = 56,
  dias = 7,
}: {
  serie: Amostra[];
  cor: string;
  altura?: number;
  dias?: number;
}) {
  const corte = Date.now() - dias * DIA;
  const janela = serie.filter((s) => s.t >= corte);

  if (janela.length < 2) return <div className="historico vazio">medindo o consumo</div>;

  const largura = 320;
  const folga = 6;

  const t0 = janela[0].t;
  const span = janela[janela.length - 1].t - t0 || 1;
  const min = Math.min(...janela.map((s) => s.p));
  const max = Math.max(...janela.map((s) => s.p));
  const faixa = Math.max(max - min, 8);
  const base = (max + min) / 2 - faixa / 2;

  const pontos = janela
    .map((s) => {
      const x = ((s.t - t0) / span) * largura;
      const y = altura - folga - ((s.p - base) / faixa) * (altura - folga * 2);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  return (
    <div className="historico">
      <div className="faixa">
        <span>
          {min}% – {max}%
        </span>
        <span>{legenda(span)}</span>
      </div>
      <svg
        width="100%"
        height={altura}
        viewBox={`0 0 ${largura} ${altura}`}
        preserveAspectRatio="none"
      >
        <polyline
          points={pontos}
          fill="none"
          stroke={cor}
          strokeWidth="1.5"
          strokeLinejoin="round"
        />
      </svg>
    </div>
  );
}

function legenda(span: number): string {
  if (span < 2 * HORA) {
    const minutos = Math.max(1, Math.round(span / 60_000));
    return `últimos ${minutos} min`;
  }
  if (span < 36 * HORA) {
    return `últimas ${Math.round(span / HORA)} h`;
  }
  return `últimos ${Math.round(span / DIA)} dias`;
}
