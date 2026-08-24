import { Amostra } from "../estado";
import "./historico.css";

/**
 * O historico e contexto, nao protagonista: linha de um pixel, sem eixo e sem grade.
 */
export function Historico({
  serie,
  cor,
  altura = 56,
}: {
  serie: Amostra[];
  cor: string;
  altura?: number;
}) {
  if (serie.length < 2) return <div className="historico vazio">medindo o consumo</div>;

  const largura = 320;
  const folga = 6;

  const t0 = serie[0].t;
  const span = serie[serie.length - 1].t - t0 || 1;
  const min = Math.min(...serie.map((s) => s.p));
  const max = Math.max(...serie.map((s) => s.p));
  // piso de amplitude para variacao minuscula nao virar uma serra gigante
  const faixa = Math.max(max - min, 8);
  const base = (max + min) / 2 - faixa / 2;

  const pontos = serie
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
        <span>últimas 24 h</span>
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
