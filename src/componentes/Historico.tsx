import { memo, useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { flushSync } from "react-dom";

import { Amostra, useLimiares } from "../estado";
import { hora, momento } from "../formato";
import "./historico.css";

const MINUTO = 60_000;
const DIA = 86_400_000;
const SALTO_SEM_VIA_GRAVADA = 30 * MINUTO;
const SALTO_DE_SEGURANCA = 6 * 60 * MINUTO;

const LARGURA_ANTES_DE_MEDIR = 520;
const ALTO = { topo: 10, direita: 10, base: 20, esquerda: 36, altura: 150 };
const BAIXO = { topo: 6, direita: 4, base: 6, esquerda: 4, altura: 64 };
const ESPACO_DO_ROTULO_DE_TROCA = 90;

type Margens = typeof ALTO;

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
  autonomiaMinutos?: number | null;
  aoSairDaJanela?: () => void;
}

interface Escala {
  M: Margens;
  largura: number;
  x: (t: number) => number;
  y: (p: number) => number;
  t: (x: number) => number;
}

interface Tracado {
  linha: string;
  area: string | null;
}

interface Projecao {
  de: Amostra;
  zeraEm: number;
}

export function Historico({
  serie,
  trocadaEm,
  compacto,
  janela,
  autonomiaMinutos,
  aoSairDaJanela,
}: Props) {
  const [dias, setDias] = useState(7);
  const [sob, setSob] = useState<Amostra | null>(null);
  const caixa = useRef<HTMLDivElement>(null);
  const largura = useLargura(caixa);
  const agora = useMinuto();
  const { critico, aviso } = useLimiares();
  const tinta = useId().replace(/:/g, "");

  const M = compacto ? BAIXO : ALTO;
  const dentroDeUmaSessao = !!janela;
  const folga = janela ? Math.max((janela.fim - janela.inicio) * 0.06, MINUTO) : 0;
  const ultimaLida = serie.length > 0 ? serie[serie.length - 1].t : 0;
  const zeraEm =
    !janela && autonomiaMinutos && autonomiaMinutos > 0 ? agora + autonomiaMinutos * MINUTO : null;
  const fim = janela ? janela.fim + folga : Math.max(agora, ultimaLida, zeraEm ?? 0);
  const inicio = janela ? janela.inicio - folga : agora - (compacto ? 7 : dias) * DIA;

  const trechos = useMemo(() => segmentar(serie, inicio, fim), [serie, inicio, fim]);
  const amostras = useMemo(() => trechos.flat(), [trechos]);
  const escala = useMemo(() => medir(M, largura, inicio, fim), [M, largura, inicio, fim]);
  const tracados = useMemo(() => trechos.map((t) => tracar(t, escala)), [trechos, escala]);

  const ultima = amostras.length > 0 ? amostras[amostras.length - 1] : null;
  const projecao = useMemo<Projecao | null>(
    () => (zeraEm && ultima && zeraEm > ultima.t ? { de: ultima, zeraEm } : null),
    [zeraEm, ultima],
  );

  const legenda = useMemo(() => {
    if (amostras.length < 2) return "";
    return dentroDeUmaSessao ? resumoDaSessao(amostras) : resumo(amostras, trechos.length);
  }, [amostras, trechos.length, dentroDeUmaSessao]);

  if (amostras.length < 2) {
    return (
      <div ref={caixa} className={`historico vazio${compacto ? " compacto" : ""}`}>
        sem histórico nesta janela
      </div>
    );
  }

  const aoMover = (e: React.MouseEvent<SVGSVGElement>) => {
    const area = e.currentTarget.getBoundingClientRect();
    if (area.width === 0) return;
    const px = ((e.clientX - area.left) / area.width) * largura;
    setSob(maisPerto(amostras, escala.t(px)));
  };

  return (
    <div ref={caixa} className={`historico${compacto ? " compacto" : ""}`}>
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
        viewBox={`0 0 ${largura} ${M.altura}`}
        className="grafico"
        onMouseMove={aoMover}
        onMouseLeave={() => setSob(null)}
      >
        <Desenho
          escala={escala}
          tracados={tracados}
          tinta={tinta}
          compacto={!!compacto}
          dentroDeUmaSessao={dentroDeUmaSessao}
          inicio={inicio}
          fim={fim}
          critico={critico}
          aviso={aviso}
          trocadaEm={trocadaEm ?? null}
          projecao={projecao}
        />
        {sob && <Mira sob={sob} escala={escala} />}
      </svg>

      <div className="historico-rodape">
        {sob ? (
          <>
            <span className="destaque">{sob.p}%</span>
            <span>{momento(sob.t)}</span>
          </>
        ) : (
          <span>{legenda}</span>
        )}
      </div>
    </div>
  );
}

const Desenho = memo(function Desenho({
  escala,
  tracados,
  tinta,
  compacto,
  dentroDeUmaSessao,
  inicio,
  fim,
  critico,
  aviso,
  trocadaEm,
  projecao,
}: {
  escala: Escala;
  tracados: Tracado[];
  tinta: string;
  compacto: boolean;
  dentroDeUmaSessao: boolean;
  inicio: number;
  fim: number;
  critico: number;
  aviso: number;
  trocadaEm: number | null;
  projecao: Projecao | null;
}) {
  const { M, largura, x, y } = escala;
  const util = largura - M.esquerda - M.direita;
  const direita = largura - M.direita;

  return (
    <>
      <defs>
        <linearGradient id={tinta} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="var(--accent-green)" stopOpacity="0.28" />
          <stop offset="100%" stopColor="var(--accent-green)" stopOpacity="0.02" />
        </linearGradient>
      </defs>

      <g className="faixas-de-limiar">
        <rect
          x={M.esquerda}
          y={y(aviso)}
          width={util}
          height={Math.max(0, y(critico) - y(aviso))}
          className="faixa-aviso"
        />
        <rect
          x={M.esquerda}
          y={y(critico)}
          width={util}
          height={Math.max(0, y(0) - y(critico))}
          className="faixa-critica"
        />
      </g>

      {[0, 50, 100].map((p) => (
        <g key={p}>
          <line x1={M.esquerda} x2={direita} y1={y(p)} y2={y(p)} className="grade" />
          {!compacto && (
            <text x={M.esquerda - 6} y={y(p) + 4} className="rotulo-y">
              {p}
            </text>
          )}
        </g>
      ))}

      {!compacto &&
        marcas(inicio, fim, dentroDeUmaSessao).map(({ t, texto }) => (
          <text key={t} x={x(t)} y={M.altura - 4} className="rotulo-x">
            {texto}
          </text>
        ))}

      {trocadaEm && trocadaEm > inicio && (
        <g>
          <line x1={x(trocadaEm)} x2={x(trocadaEm)} y1={M.topo} y2={y(0)} className="troca" />
          {!compacto && (
            <text
              x={x(trocadaEm) + (x(trocadaEm) > largura - ESPACO_DO_ROTULO_DE_TROCA ? -4 : 4)}
              y={M.topo + 11}
              className="rotulo-troca"
              textAnchor={x(trocadaEm) > largura - ESPACO_DO_ROTULO_DE_TROCA ? "end" : "start"}
            >
              bateria nova
            </text>
          )}
        </g>
      )}

      {tracados.map((tracado, i) => (
        <g key={i}>
          {tracado.area && <path d={tracado.area} className="area" fill={`url(#${tinta})`} />}
          <path d={tracado.linha} className="linha" />
        </g>
      ))}

      {projecao && (
        <g className="projecao">
          <path
            d={`M${x(projecao.de.t)} ${y(projecao.de.p)} L${x(projecao.zeraEm)} ${y(0)}`}
            className="linha-projetada"
          />
          {!compacto && (
            <text
              x={Math.min(x(projecao.zeraEm), direita) - 4}
              y={y(0) - 6}
              className="rotulo-projecao"
              textAnchor="end"
            >
              zera {hora(new Date(projecao.zeraEm))}
            </text>
          )}
        </g>
      )}
    </>
  );
});

function Mira({ sob, escala }: { sob: Amostra; escala: Escala }) {
  const { M, x, y } = escala;
  return (
    <g>
      <line x1={x(sob.t)} x2={x(sob.t)} y1={M.topo} y2={y(0)} className="mira" />
      <circle cx={x(sob.t)} cy={y(sob.p)} r={4} className="ponto" />
    </g>
  );
}

function useLargura(caixa: React.RefObject<HTMLDivElement | null>): number {
  const [largura, setLargura] = useState(LARGURA_ANTES_DE_MEDIR);

  useLayoutEffect(() => {
    const alvo = caixa.current;
    if (!alvo) return;

    const inicial = alvo.getBoundingClientRect().width;
    if (inicial > 0) setLargura(inicial);

    const observador = new ResizeObserver(([entrada]) => {
      const medida = entrada.contentRect.width;
      if (medida > 0) flushSync(() => setLargura(medida));
    });
    observador.observe(alvo);
    return () => observador.disconnect();
  }, [caixa]);

  return largura;
}

function useMinuto(): number {
  const [minuto, setMinuto] = useState(() => Math.floor(Date.now() / MINUTO) * MINUTO);

  useEffect(() => {
    const relogio = setInterval(() => setMinuto(Math.floor(Date.now() / MINUTO) * MINUTO), MINUTO);
    return () => clearInterval(relogio);
  }, []);

  return minuto;
}

function medir(M: Margens, largura: number, inicio: number, fim: number): Escala {
  const util = largura - M.esquerda - M.direita;
  const alto = M.altura - M.topo - M.base;
  const duracao = fim - inicio;
  return {
    M,
    largura,
    x: (t) => M.esquerda + ((t - inicio) / duracao) * util,
    y: (p) => M.topo + (1 - p / 100) * alto,
    t: (x) => inicio + ((x - M.esquerda) / util) * duracao,
  };
}

function tracar(trecho: Amostra[], { x, y }: Escala): Tracado {
  const linha = trecho
    .map((a, i) => `${i === 0 ? "M" : "L"}${x(a.t).toFixed(1)} ${y(a.p).toFixed(1)}`)
    .join(" ");
  if (trecho.length < 2) return { linha, area: null };

  const primeira = x(trecho[0].t).toFixed(1);
  const ultima = x(trecho[trecho.length - 1].t).toFixed(1);
  const chao = y(0).toFixed(1);
  return { linha, area: `${linha} L${ultima} ${chao} L${primeira} ${chao} Z` };
}

function maisPerto(amostras: Amostra[], t: number): Amostra {
  let baixo = 0;
  let alto = amostras.length - 1;
  while (baixo < alto) {
    const meio = (baixo + alto) >> 1;
    if (amostras[meio].t < t) baixo = meio + 1;
    else alto = meio;
  }

  const depois = amostras[baixo];
  const antes = amostras[baixo - 1];
  return antes && t - antes.t <= depois.t - t ? antes : depois;
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
  if (anterior.via === "Desligado") return true;
  const limite = anterior.via ? SALTO_DE_SEGURANCA : SALTO_SEM_VIA_GRAVADA;
  return seguinte.t - anterior.t > limite;
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
