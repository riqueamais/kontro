import { useEffect, useState } from "react";

const CAIXA = 512;
const CENTRO = CAIXA / 2;

const ARCO_DO_GIRO = 96;

function ponto(raio: number, graus: number) {
  const rad = (graus * Math.PI) / 180;
  return { x: CENTRO + raio * Math.cos(rad), y: CENTRO + raio * Math.sin(rad) };
}

function arco(raio: number, inicio: number, varredura: number) {
  const v = Math.min(Math.max(varredura, 0), 359.9);
  const a = ponto(raio, inicio);
  const b = ponto(raio, inicio + v);
  return `M${a.x} ${a.y} A${raio} ${raio} 0 ${v > 180 ? 1 : 0} 1 ${b.x} ${b.y}`;
}

interface Props {
  /** Percentual de preenchimento. Ignorado quando girando. */
  valor: number | null;
  cor: string;
  espessura: number;
  tamanho: number;
  /** Giro continuo no lugar da carga: na energia, nao ha numero a mostrar. */
  girando?: boolean;
  children?: React.ReactNode;
}

export function Anel({ valor, cor, espessura, tamanho, girando, children }: Props) {
  const raio = (CAIXA - espessura) / 2;

  // A transicao entre percentuais e animada aqui, e nao no CSS, porque o arco muda de
  // forma e nao so de tamanho -- interpolar o atributo `d` daria um caminho invalido.
  const [suave, setSuave] = useState(valor ?? 0);
  useEffect(() => {
    if (girando) return;
    const alvo = valor ?? 0;
    let quadro = 0;
    const inicio = performance.now();
    const partida = suave;
    const passo = (agora: number) => {
      const t = Math.min((agora - inicio) / 180, 1);
      const eased = 1 - Math.pow(1 - t, 3);
      setSuave(partida + (alvo - partida) * eased);
      if (t < 1) quadro = requestAnimationFrame(passo);
    };
    quadro = requestAnimationFrame(passo);
    return () => cancelAnimationFrame(quadro);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [valor, girando]);

  const cheio = !girando && suave >= 99.9;

  return (
    <div style={{ position: "relative", width: tamanho, height: tamanho }}>
      <svg
        width={tamanho}
        height={tamanho}
        viewBox={`0 0 ${CAIXA} ${CAIXA}`}
        style={{ display: "block" }}
      >
        <circle
          cx={CENTRO}
          cy={CENTRO}
          r={raio}
          fill="none"
          stroke="var(--ring-track)"
          strokeWidth={espessura}
        />
        {girando ? (
          <g style={{ transformOrigin: "50% 50%", animation: "kontro-girar 1.7s linear infinite" }}>
            <path
              d={arco(raio, -90, ARCO_DO_GIRO)}
              fill="none"
              stroke={cor}
              strokeWidth={espessura}
              strokeLinecap="round"
            />
          </g>
        ) : cheio ? (
          <circle cx={CENTRO} cy={CENTRO} r={raio} fill="none" stroke={cor} strokeWidth={espessura} />
        ) : suave > 0.1 ? (
          <path
            d={arco(raio, -90, (360 * suave) / 100)}
            fill="none"
            stroke={cor}
            strokeWidth={espessura}
            strokeLinecap="round"
          />
        ) : null}
      </svg>

      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "grid",
          placeItems: "center",
        }}
      >
        {children}
      </div>
    </div>
  );
}
