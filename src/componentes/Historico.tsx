import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { flushSync } from "react-dom";

import { Amostra, useLimiares } from "../estado";
import { diaEMes, diasAtras, duracao, hora, quando, taxa } from "../formato";
import "./historico.css";

const DIA = 86_400_000;
const MINUTO = 60_000;
const SALTO_SEM_VIA_GRAVADA = 30 * 60_000;
const SALTO_DE_SEGURANCA = 6 * 60 * 60_000;

const LARGURA_ATE_MEDIR = 520;
const ALTO = { topo: 10, direita: 10, base: 22, esquerda: 36, altura: 150 };
const BAIXO = { topo: 6, direita: 4, base: 6, esquerda: 4, altura: 64 };

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

function useMinutoAtual(): number {
  const [agora, setAgora] = useState(() => Math.floor(Date.now() / MINUTO) * MINUTO);
  useEffect(() => {
    const relogio = setInterval(() => setAgora(Math.floor(Date.now() / MINUTO) * MINUTO), MINUTO);
    return () => clearInterval(relogio);
  }, []);
  return agora;
}

function useLarguraMedida(elemento: HTMLElement | null): number {
  const [largura, setLargura] = useState(LARGURA_ATE_MEDIR);
  useLayoutEffect(() => {
    if (!elemento) return;
    const inicial = Math.round(elemento.getBoundingClientRect().width);
    if (inicial > 0) setLargura(inicial);

    const observador = new ResizeObserver(([entrada]) => {
      const medida = Math.round(entrada.contentRect.width);
      if (medida > 0) flushSync(() => setLargura(medida));
    });
    observador.observe(elemento);
    return () => observador.disconnect();
  }, [elemento]);
  return largura;
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
  const [raiz, setRaiz] = useState<HTMLDivElement | null>(null);
  const LARGURA = useLarguraMedida(raiz);
  const limiares = useLimiares();
  const tinta = useId().replace(/:/g, "");
  const agora = useMinutoAtual();

  const M = compacto ? BAIXO : ALTO;
  const dentroDeUmaSessao = !!janela;
  const ultimaLida = serie.length > 0 ? serie[serie.length - 1].t : 0;
  const ALTURA = M.altura;
  const folga = janela ? Math.max((janela.fim - janela.inicio) * 0.06, 60_000) : 0;
  const zeraEm =
    !janela && autonomiaMinutos && autonomiaMinutos > 0
      ? agora + autonomiaMinutos * 60_000
      : null;
  const inicio = janela ? janela.inicio - folga : agora - (compacto ? 7 : dias) * DIA;
  const tetoDaProjecao = agora + (agora - inicio) / 3;
  const fim = janela
    ? janela.fim + folga
    : Math.max(agora, ultimaLida, Math.min(zeraEm ?? 0, tetoDaProjecao));

  const decidiuAJanela = useRef(false);
  useEffect(() => {
    if (decidiuAJanela.current || compacto || janela || serie.length === 0) return;
    decidiuAJanela.current = true;
    const semana = segmentar(serie, agora - 7 * DIA, agora).flat().length;
    const mes = segmentar(serie, agora - 30 * DIA, agora).flat().length;
    if (semana < 2 && mes >= 2) setDias(30);
  }, [serie, compacto, janela, agora]);

  const trechos = useMemo(() => segmentar(serie, inicio, fim), [serie, inicio, fim]);
  const amostras = useMemo(() => trechos.flat(), [trechos]);

  const largura = LARGURA - M.esquerda - M.direita;
  const altura = ALTURA - M.topo - M.base;
  const x = (t: number) => M.esquerda + ((t - inicio) / (fim - inicio)) * largura;
  const y = (p: number) => M.topo + (1 - p / 100) * altura;

  const ultima = amostras[amostras.length - 1];
  const projecao =
    zeraEm && ultima && zeraEm > ultima.t
      ? {
          de: ultima,
          zeraEm,
          cortada: zeraEm > fim,
          ate:
            zeraEm > fim
              ? { t: fim, p: ultima.p * (1 - (fim - ultima.t) / (zeraEm - ultima.t)) }
              : { t: zeraEm, p: 0 },
        }
      : null;

  const estatico = useMemo(
    () => (
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
            y={y(limiares.aviso)}
            width={largura}
            height={Math.max(0, y(limiares.critico) - y(limiares.aviso))}
            className="faixa-aviso"
          />
          <rect
            x={M.esquerda}
            y={y(limiares.critico)}
            width={largura}
            height={Math.max(0, y(0) - y(limiares.critico))}
            className="faixa-critica"
          />
        </g>

        {[0, 50, 100].map((p) => (
          <line
            key={p}
            x1={M.esquerda}
            x2={LARGURA - M.direita}
            y1={y(p)}
            y2={y(p)}
            className="grade"
          />
        ))}

        {[
          { p: limiares.aviso, classe: "aviso" },
          { p: limiares.critico, classe: "critico" },
        ].map(({ p, classe }) => (
          <line
            key={classe}
            x1={M.esquerda}
            x2={LARGURA - M.direita}
            y1={y(p)}
            y2={y(p)}
            className={`limiar ${classe}`}
          />
        ))}

        {!compacto &&
          rotulosDoEixo(limiares, y).map(({ p, classe }) => (
            <text key={`${classe}-${p}`} x={M.esquerda - 6} y={y(p) + 4} className={`rotulo-y ${classe}`}>
              {p}
            </text>
          ))}

        {!compacto &&
          marcas(inicio, fim, dentroDeUmaSessao).map(({ t, texto }) => (
            <text key={t} x={x(t)} y={ALTURA - 6} className="rotulo-x">
              {texto}
            </text>
          ))}

        {!dentroDeUmaSessao && (
          <g>
            <line x1={x(agora)} x2={x(agora)} y1={M.topo} y2={y(0)} className="agora" />
            {!compacto && (
              <text x={x(agora) - 4} y={M.topo + 11} className="rotulo-x rotulo-agora">
                agora
              </text>
            )}
          </g>
        )}

        {trocadaEm && trocadaEm > inicio && (
          <g>
            <line x1={x(trocadaEm)} x2={x(trocadaEm)} y1={M.topo} y2={y(0)} className="troca" />
            {!compacto && (
              <text
                x={x(trocadaEm) + (x(trocadaEm) > LARGURA - 110 ? -4 : 4)}
                y={M.topo + 11}
                className="rotulo-troca"
                textAnchor={x(trocadaEm) > LARGURA - 110 ? "end" : "start"}
              >
                bateria nova
              </text>
            )}
          </g>
        )}

        {trechos.map((trecho, i) => {
          const linha = caminho(trecho, x, y);
          return (
            <g key={i}>
              {trecho.length > 1 && (
                <path
                  d={`${linha} L${x(trecho[trecho.length - 1].t)} ${y(0)} L${x(trecho[0].t)} ${y(0)} Z`}
                  className="area"
                  fill={`url(#${tinta})`}
                />
              )}
              <path d={linha} className="linha" />
            </g>
          );
        })}

        {projecao && (
          <g className="projecao">
            <path
              d={`M${x(projecao.de.t)} ${y(projecao.de.p)} L${x(projecao.ate.t)} ${y(projecao.ate.p)}`}
              className="linha-projetada"
            />
            {!compacto && (
              <text
                x={Math.min(x(projecao.ate.t), LARGURA - M.direita) - 4}
                y={y(0) - 6}
                className="rotulo-projecao"
                textAnchor="end"
              >
                zera {quandoZera(projecao.zeraEm, agora)}
                {projecao.cortada && " →"}
              </text>
            )}
          </g>
        )}
      </>
    ),
    [
      trechos,
      inicio,
      fim,
      LARGURA,
      compacto,
      limiares.aviso,
      limiares.critico,
      trocadaEm,
      dentroDeUmaSessao,
      projecao?.zeraEm,
      projecao?.ate.t,
      agora,
      tinta,
    ],
  );

  const vazio = amostras.length < 2;

  const aoMover = (e: React.MouseEvent<SVGSVGElement>) => {
    const caixa = e.currentTarget.getBoundingClientRect();
    const px = e.clientX - caixa.left;
    const t = inicio + ((px - M.esquerda) / largura) * (fim - inicio);
    const perto = maisPerto(amostras, t);
    const alvo = Math.abs(x(perto.t) - px) <= ALCANCE_DO_CURSOR ? perto : null;
    setSob((atual) => (atual === alvo ? atual : alvo));
  };

  const aoTeclar = (e: React.KeyboardEvent<SVGSVGElement>) => {
    const i = sob ? amostras.indexOf(sob) : -1;
    const proximo = {
      ArrowLeft: i < 0 ? amostras.length - 1 : Math.max(0, i - 1),
      ArrowRight: i < 0 ? 0 : Math.min(amostras.length - 1, i + 1),
      Home: 0,
      End: amostras.length - 1,
    }[e.key];
    if (e.key === "Escape") {
      setSob(null);
      return;
    }
    if (proximo === undefined) return;
    e.preventDefault();
    setSob(amostras[proximo]);
  };

  const legenda = vazio ? "" : janela ? resumoDaSessao(amostras, janela) : resumo(amostras, trechos.length);

  return (
    <div ref={setRaiz} className={`historico${compacto ? " compacto" : ""}`}>
      {!compacto && (
        <div className="historico-topo">
          <span className={`historico-titulo${janela ? " livre" : ""}`} title={janela?.titulo}>
            {janela ? janela.titulo : "Carga"}
          </span>
          <div className="faixas">
            {janela ? (
              <button
                className="faixa ativa"
                aria-label="Voltar ao gráfico de 7 dias"
                onClick={aoSairDaJanela}
              >
                <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
                  <path
                    d="M10 6H2.5M5.5 2.5 2 6l3.5 3.5"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.4"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
                Voltar
              </button>
            ) : (
              FAIXAS.map((f) => (
                <button
                  key={f.dias}
                  className={`faixa${dias === f.dias ? " ativa" : ""}`}
                  aria-pressed={dias === f.dias}
                  onClick={() => setDias(f.dias)}
                >
                  {f.rotulo}
                </button>
              ))
            )}
          </div>
        </div>
      )}

      {vazio ? (
        <div className="vazio" style={{ height: ALTURA }}>
          {serie.length === 0 ? (
            "O histórico começa na primeira leitura com o controle ligado."
          ) : compacto ? (
            "Sem leituras esta semana"
          ) : janela ? (
            "Nenhuma leitura nesta sessão."
          ) : dias === 7 ? (
            <>
              Nada nos últimos 7 dias.{" "}
              <button className="para-30" onClick={() => setDias(30)}>
                Veja os 30 dias.
              </button>
            </>
          ) : (
            "Nada nos últimos 30 dias."
          )}
        </div>
      ) : (
        <>
          <div className="grafico-area">
            <svg
              width={LARGURA}
              height={ALTURA}
              viewBox={`0 0 ${LARGURA} ${ALTURA}`}
              className="grafico"
              tabIndex={0}
              role="img"
              aria-label={legenda}
              onMouseMove={aoMover}
              onMouseLeave={() => setSob(null)}
              onKeyDown={aoTeclar}
              onBlur={() => setSob(null)}
            >
              {estatico}

              {sob && (
                <g>
                  <line x1={x(sob.t)} x2={x(sob.t)} y1={M.topo} y2={y(0)} className="mira" />
                  <circle cx={x(sob.t)} cy={y(sob.p)} r={4} className="ponto" />
                </g>
              )}
            </svg>
            {sob && (
              <div
                className={`dica${y(sob.p) < 40 ? " abaixo" : ""}`}
                role="status"
                aria-live="polite"
                style={{ left: x(sob.t), top: y(sob.p) }}
              >
                <span className="dica-valor">{sob.p}%</span>
                <span className="dica-quando">
                  {quando(sob.t)}
                  {sob.via && ` · ${NOME_DA_VIA[sob.via]}`}
                </span>
              </div>
            )}
          </div>

          <div className="historico-rodape">
            <span>{legenda}</span>
          </div>
        </>
      )}
    </div>
  );
}

const DISTANCIA_ENTRE_ROTULOS = 12;

function rotulosDoEixo(
  limiares: { aviso: number; critico: number },
  y: (p: number) => number,
): { p: number; classe: string }[] {
  const candidatos = [
    { p: limiares.aviso, classe: "aviso" },
    { p: limiares.critico, classe: "critico" },
    { p: 100, classe: "" },
    { p: 50, classe: "" },
    { p: 0, classe: "" },
  ];
  const aceitos: { p: number; classe: string }[] = [];
  for (const c of candidatos) {
    if (aceitos.every((a) => Math.abs(y(a.p) - y(c.p)) >= DISTANCIA_ENTRE_ROTULOS)) aceitos.push(c);
  }
  return aceitos;
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
  const antes = amostras[Math.max(0, baixo - 1)];
  return Math.abs(antes.t - t) <= Math.abs(depois.t - t) ? antes : depois;
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
        texto: hora(new Date(t)),
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
        texto: diaEMes(new Date(t)),
      });
    }
  }
  return saida;
}

const DOZE_HORAS = 12 * 3_600_000;

const ALCANCE_DO_CURSOR = 24;

const NOME_DA_VIA: Record<NonNullable<Amostra["via"]>, string> = {
  Bluetooth: "Bluetooth",
  Cabo: "cabo",
  SemFio: "sem fio",
  Desligado: "desligado",
};

function quandoZera(ms: number, agora: number): string {
  return ms - agora > DOZE_HORAS || diasAtras(new Date(ms)) !== 0
    ? quando(ms)
    : `às ${hora(new Date(ms))}`;
}

function resumoDaSessao(amostras: Amostra[], janela: Janela): string {
  const de = amostras[0].p;
  const ate = amostras[amostras.length - 1].p;
  const minutos = Math.round((janela.fim - janela.inicio) / 60_000);
  const partes = [quando(amostras[0].t), `${de}% → ${ate}%`, duracao(minutos)];
  if (de > ate && minutos > 0) partes.push(taxa(((de - ate) * 60) / minutos));
  return partes.join(" · ");
}

function resumo(amostras: Amostra[], trechos: number): string {
  const min = Math.min(...amostras.map((a) => a.p));
  const max = Math.max(...amostras.map((a) => a.p));
  const sessoes = trechos === 1 ? "1 período" : `${trechos} períodos`;
  return `${min}% a ${max}%, em ${sessoes} com o controle ligado`;
}
