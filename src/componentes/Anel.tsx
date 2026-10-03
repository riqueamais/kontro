import { useEffect, useId, useState } from "react";

import { movimentoReduzidoAgora, useMovimentoReduzido } from "../movimento";
import "./anel.css";

const CAIXA = 512;
const CENTRO = CAIXA / 2;

const COMPRIMENTO_DA_MARCA = 0.5;

const RAIO_DO_HALO = CAIXA * 0.125;

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
  rotulo?: string;
  desde?: number;
  children?: React.ReactNode;
}

export function Anel({
  valor,
  cor,
  espessura,
  tamanho,
  girando,
  marcas,
  rotulo,
  desde,
  children,
}: Props) {
  const raio = (CAIXA - espessura) / 2;
  const luz = useId().replace(/:/g, "");

  const reduzido = useMovimentoReduzido();
  const [suave, setSuave] = useState(() =>
    desde !== undefined && !movimentoReduzidoAgora() ? desde : (valor ?? 0),
  );
  useEffect(() => {
    if (girando) return;
    const alvo = valor ?? 0;
    if (reduzido) {
      setSuave(alvo);
      return;
    }
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
  }, [valor, girando, reduzido]);

  const cheio = !girando && suave >= 99.9;
  const apagada = cor === "var(--gray)";
  const tinta = `url(#${luz})`;
  const mostrarMarcas = marcas && !girando && valor !== null;

  return (
    <div
      className={apagada ? "anel apagada" : "anel"}
      style={
        {
          width: tamanho,
          height: tamanho,
          "--cor-do-anel": cor,
          "--raio-do-halo": `${RAIO_DO_HALO}px`,
        } as React.CSSProperties
      }
    >
      <svg
        width={tamanho}
        height={tamanho}
        viewBox={`0 0 ${CAIXA} ${CAIXA}`}
        role={rotulo ? "img" : undefined}
        aria-label={rotulo}
        aria-hidden={rotulo ? undefined : true}
      >
        <defs>
          <linearGradient id={luz} x1="0.12" y1="0.08" x2="0.86" y2="0.94">
            <stop offset="0%" />
            <stop offset="100%" />
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

        <g className="carga">
          {girando || cheio ? (
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
              strokeWidth={espessura * 0.1}
              strokeLinecap="round"
            />
            <path
              d={marca(raio, espessura, marcas.critico)}
              stroke="var(--red)"
              strokeWidth={espessura * 0.1}
              strokeLinecap="round"
            />
          </g>
        )}
      </svg>

      <div className="miolo">{children}</div>
    </div>
  );
}
