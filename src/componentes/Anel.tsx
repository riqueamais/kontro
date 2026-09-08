import { useEffect, useId, useState } from "react";

import "./anel.css";

const CAIXA = 512;
const CENTRO = CAIXA / 2;

const ARCO_DO_GIRO = 96;

const COMPRIMENTO_DA_MARCA = 0.62;

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

function marca(raio: number, espessura: number, percentual: number) {
  const graus = -90 + (360 * percentual) / 100;
  const meia = (espessura * COMPRIMENTO_DA_MARCA) / 2;
  const de = ponto(raio - meia, graus);
  const ate = ponto(raio + meia, graus);
  return `M${de.x} ${de.y} L${ate.x} ${ate.y}`;
}

export interface Marcas {
  critico: number;
  aviso: number;
}

interface Props {
  valor: number | null;
  cor: string;
  espessura: number;
  tamanho: number;
  girando?: boolean;
  marcas?: Marcas | null;
  children?: React.ReactNode;
}

export function Anel({ valor, cor, espessura, tamanho, girando, marcas, children }: Props) {
  const raio = (CAIXA - espessura) / 2;
  const luz = useId().replace(/:/g, "");

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
  }, [valor, girando]);

  const cheio = !girando && suave >= 99.9;
  const tinta = `url(#${luz})`;
  const mostrarMarcas = marcas && !girando && valor !== null;

  return (
    <div className="anel" style={{ width: tamanho, height: tamanho }}>
      <svg width={tamanho} height={tamanho} viewBox={`0 0 ${CAIXA} ${CAIXA}`}>
        <defs>
          <linearGradient id={luz} x1="0.12" y1="0.08" x2="0.86" y2="0.94">
            <stop offset="0%" stopColor={`color-mix(in srgb, ${cor} 84%, #ffffff)`} />
            <stop offset="100%" stopColor={cor} />
          </linearGradient>
        </defs>

        <circle
          cx={CENTRO}
          cy={CENTRO}
          r={raio}
          fill="none"
          stroke="var(--ring-track)"
          strokeWidth={espessura}
        />

        <g className="carga" style={{ color: cor }}>
          {girando ? (
            <g className="giro">
              <path
                d={arco(raio, -90, ARCO_DO_GIRO)}
                fill="none"
                stroke={tinta}
                strokeWidth={espessura}
                strokeLinecap="round"
              />
            </g>
          ) : cheio ? (
            <circle
              cx={CENTRO}
              cy={CENTRO}
              r={raio}
              fill="none"
              stroke={tinta}
              strokeWidth={espessura}
            />
          ) : suave > 0.1 ? (
            <path
              d={arco(raio, -90, (360 * suave) / 100)}
              fill="none"
              stroke={tinta}
              strokeWidth={espessura}
              strokeLinecap="round"
            />
          ) : null}
        </g>

        {mostrarMarcas && (
          <g className="marcas">
            <path
              d={marca(raio, espessura, marcas.aviso)}
              stroke="var(--amber)"
              strokeWidth={espessura * 0.14}
              strokeLinecap="round"
            />
            <path
              d={marca(raio, espessura, marcas.critico)}
              stroke="var(--red)"
              strokeWidth={espessura * 0.14}
              strokeLinecap="round"
            />
          </g>
        )}
      </svg>

      <div className="miolo">{children}</div>
    </div>
  );
}
