import { useMemo, useRef, useState } from "react";

import { Amostra } from "../estado";
import "./historico.css";

const DIA = 86_400_000;
const SALTO_SEM_VIA_GRAVADA = 30 * 60_000;
const SALTO_DE_SEGURANCA = 6 * 60 * 60_000;

const LARGURA = 520;
const ALTO = { topo: 10, direita: 10, base: 20, esquerda: 32, altura: 150 };
const BAIXO = { topo: 6, direita: 4, base: 6, esquerda: 4, altura: 70 };

const FAIXAS = [
  { dias: 7, rotulo: "7 dias" },
  { dias: 30, rotulo: "30 dias" },
] as const;

interface Janela {
  inicio: number;
  fim: number;
  titulo: string;
}

interface Props {
  serie: Amostra[];
  trocadaEm?: number | null;
  compacto?: boolean;
  janela?: Janela | null;
  aoSairDaJanela?: () => void;
}

export function Historico({ serie, trocadaEm, compacto, janela, aoSairDaJanela }: Props) {
  const [dias, setDias] = useState(7);
  const [sob, setSob] = useState<Amostra | null>(null);
  const area = useRef<SVGSVGElement>(null);

  const M = compacto ? BAIXO : ALTO;
  const ALTURA = M.altura;
  const folga = janela ? Math.max((janela.fim - janela.inicio) * 0.06, 60_000) : 0;
  const fim = janela ? janela.fim + folga : Date.now();
  const inicio = janela ? janela.inicio - folga : fim - (compacto ? 7 : dias) * DIA;

  const trechos = useMemo(() => segmentar(serie, inicio, fim), [serie, inicio, fim]);
  const amostras = useMemo(() => trechos.flat(), [trechos]);

  if (amostras.length < 2) {
    return <div className="historico vazio">sem histórico nesta janela</div>;
  }

  const largura = LARGURA - M.esquerda - M.direita;
  const altura = ALTURA - M.topo - M.base;
  const x = (t: number) => M.esquerda + ((t - inicio) / (fim - inicio)) * largura;
  const y = (p: number) => M.topo + (1 - p / 100) * altura;

  const aoMover = (e: React.MouseEvent<SVGSVGElement>) => {
    const svg = area.current;
    if (!svg) return;
    const caixa = svg.getBoundingClientRect();
    const t = inicio + ((e.clientX - caixa.left) / caixa.width) * (fim - inicio);
    let perto = amostras[0];
    for (const a of amostras) {
      if (Math.abs(a.t - t) < Math.abs(perto.t - t)) perto = a;
    }
    setSob(perto);
  };

  return (
    <div className={`historico${compacto ? " compacto" : ""}`}>
      {!compacto && (
        <div className="historico-topo">
          <span className={`historico-titulo${janela ? " livre" : ""}`}>
            {janela ? janela.titulo : "Carga"}
          </span>
          <div className="faixas">
            {janela ? (
              <button className="faixa ativa" onClick={aoSairDaJanela}>
                voltar
              </button>
            ) : (
              FAIXAS.map((f) => (
                <button
                  key={f.dias}
                  className={`faixa${dias === f.dias ? " ativa" : ""}`}
                  onClick={() => setDias(f.dias)}
                >
                  {f.rotulo}
                </button>
              ))
            )}
          </div>
        </div>
      )}

      <svg
        ref={area}
        viewBox={`0 0 ${LARGURA} ${ALTURA}`}
        preserveAspectRatio="none"
        className="grafico"
        onMouseMove={aoMover}
        onMouseLeave={() => setSob(null)}
      >
        {[0, 50, 100].map((p) => (
          <g key={p}>
            <line x1={M.esquerda} x2={LARGURA - M.direita} y1={y(p)} y2={y(p)} className="grade" />
            {!compacto && (
              <text x={M.esquerda - 6} y={y(p) + 3} className="rotulo-y">
                {p}
              </text>
            )}
          </g>
        ))}

        {!compacto &&
          marcas(inicio, fim, !!janela).map(({ t, texto }) => (
            <text key={t} x={x(t)} y={ALTURA - 6} className="rotulo-x">
              {texto}
            </text>
          ))}

        {trocadaEm && trocadaEm > inicio && (
          <g>
            <line x1={x(trocadaEm)} x2={x(trocadaEm)} y1={M.topo} y2={y(0)} className="troca" />
            {!compacto && (
              <text
                x={x(trocadaEm) + (x(trocadaEm) > LARGURA - 90 ? -4 : 4)}
                y={M.topo + 9}
                className="rotulo-troca"
                textAnchor={x(trocadaEm) > LARGURA - 90 ? "end" : "start"}
              >
                bateria nova
              </text>
            )}
          </g>
        )}

        {trechos.map((trecho, i) => (
          <g key={i}>
            {trecho.length > 1 && (
              <path
                d={`${caminho(trecho, x, y)} L${x(trecho[trecho.length - 1].t)} ${y(0)} L${x(trecho[0].t)} ${y(0)} Z`}
                className="area"
              />
            )}
            <path d={caminho(trecho, x, y)} className="linha" />
          </g>
        ))}

        {sob && (
          <g>
            <line x1={x(sob.t)} x2={x(sob.t)} y1={M.topo} y2={y(0)} className="mira" />
            <circle cx={x(sob.t)} cy={y(sob.p)} r={4} className="ponto" />
          </g>
        )}
      </svg>

      <div className="historico-rodape">
        {sob ? (
          <>
            <span className="destaque">{sob.p}%</span>
            <span>{quando(sob.t)}</span>
          </>
        ) : (
          <span>{janela ? resumoDaSessao(amostras) : resumo(amostras, trechos.length)}</span>
        )}
      </div>
    </div>
  );
}

function segmentar(serie: Amostra[], inicio: number, fim: number): Amostra[][] {
  const trechos: Amostra[][] = [];
  let atual: Amostra[] = [];

  for (const a of serie) {
    if (a.t < inicio || a.t > fim) continue;
    const anterior = atual[atual.length - 1];
    if (anterior && quebra(anterior, a)) {
      trechos.push(atual);
      atual = [];
    }
    atual.push(a);
  }
  if (atual.length) trechos.push(atual);
  return trechos;
}

function quebra(anterior: Amostra, seguinte: Amostra): boolean {
  if (anterior.via === "Offline") return true;
  const limite = anterior.via ? SALTO_DE_SEGURANCA : SALTO_SEM_VIA_GRAVADA;
  return seguinte.t - anterior.t > limite;
}

function caminho(
  trecho: Amostra[],
  x: (t: number) => number,
  y: (p: number) => number,
): string {
  return trecho
    .map((a, i) => `${i === 0 ? "M" : "L"}${x(a.t).toFixed(1)} ${y(a.p).toFixed(1)}`)
    .join(" ");
}

function marcas(inicio: number, fim: number, dentroDeUmaSessao: boolean) {
  const saida: { t: number; texto: string }[] = [];

  if (dentroDeUmaSessao) {
    for (let i = 1; i <= 4; i++) {
      const t = inicio + ((fim - inicio) * i) / 5;
      saida.push({
        t,
        texto: new Date(t).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" }),
      });
    }
    return saida;
  }

  const passo = fim - inicio <= 8 * DIA ? DIA : 7 * DIA;
  const meiaNoite = new Date(fim);
  meiaNoite.setHours(0, 0, 0, 0);

  for (let d = 0; d * passo < fim - inicio; d++) {
    const t = meiaNoite.getTime() - d * passo;
    if (t > inicio + passo / 2) {
      saida.push({
        t,
        texto: new Date(t).toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" }),
      });
    }
  }
  return saida;
}

function quando(ms: number): string {
  const q = new Date(ms);
  return `${q.toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" })} às ${q.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}`;
}

function resumoDaSessao(amostras: Amostra[]): string {
  const de = amostras[0].p;
  const ate = amostras[amostras.length - 1].p;
  const horas = (amostras[amostras.length - 1].t - amostras[0].t) / 3_600_000;
  if (de <= ate || horas <= 0) return `${amostras.length} leituras nesta sessão`;

  const taxa = (de - ate) / horas;
  return `${(de - ate)} pontos em ${Math.round(horas * 60)} min, ou ${taxa.toFixed(1).replace(".", ",")}% por hora`;
}

function resumo(amostras: Amostra[], trechos: number): string {
  const min = Math.min(...amostras.map((a) => a.p));
  const max = Math.max(...amostras.map((a) => a.p));
  const sessoes = trechos === 1 ? "1 período" : `${trechos} períodos`;
  return `${min}% a ${max}%, em ${sessoes} com o controle ligado`;
}
