import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";

import { IconeDoJogo } from "../componentes/IconeDoJogo";
import { APP, CAIXA, CORES, PAD, PAD_COM_STICKS_VAZADOS } from "../estilo/geometria.gerada";
import { Limiares, Sessao, useAoMudarOHistorico, useEstado, useLimiares } from "../estado";
import { diaEMes, duracao, taxa } from "../formato";
import "./diario.css";

const DIAS = 30;
const SESSOES_PARA_ENTRAR_NO_RANKING = 3;
const MINUTOS_PARA_CONTAR_TAXA = 20;
const LETRAS_DA_SEMANA = ["S", "T", "Q", "Q", "S", "S", "D"];
const BRILHO_POR_NIVEL = [0, 0.35, 0.55, 0.78, 1];
const AVISO_DO_CARTAO_MS = 4000;

interface Dia {
  quando: number;
  minutos: number;
  menorCarga: number | null;
  jogos: string[];
}

interface Faminto {
  jogo: string;
  porHora: number;
  sessoes: number;
  minutos: number;
}

interface Numero {
  valor: string;
  rotulo: string;
}

export function Diario({ ativa }: { ativa: boolean }) {
  const estado = useEstado();
  const [sessoes, setSessoes] = useState<Sessao[] | null>(null);
  const limiares = useLimiares();

  useAoMudarOHistorico(() => {
    invoke<Sessao[]>("sessoes_do_controle").then(setSessoes).catch(() => {});
  }, [estado?.chave, estado?.percentual, estado?.via, estado?.lidoEm]);

  const lista = sessoes ?? [];
  const dias = useMemo(() => porDia(lista), [sessoes, ativa]);
  const famintos = useMemo(() => porJogo(lista), [sessoes]);
  const numeros = useMemo(() => numerosDoMes(dias, lista), [dias, sessoes]);

  if (sessoes === null) return <h1 className="titulo-da-pagina">Diário</h1>;

  const totalMinutos = dias.reduce((soma, d) => soma + d.minutos, 0);
  const maiorDia = dias.reduce((maior, d) => Math.max(maior, d.minutos), 0);
  const vazio = totalMinutos === 0;

  return (
    <>
      <h1 className="titulo-da-pagina">Diário</h1>

      <section className="cartao">
        <div className="diario-cabeca">
          <div>
            <div className={`diario-numero${vazio ? " apagado" : ""}`}>
              {vazio ? "0 min" : duracao(totalMinutos)}
            </div>
            <div className="diario-rotulo">de jogo em {DIAS} dias</div>
          </div>
          <Exportar
            dias={dias}
            famintos={famintos}
            numeros={numeros}
            totalMinutos={totalMinutos}
            limiares={limiares}
            desabilitado={vazio}
          />
        </div>

        {!vazio && (
          <div className="diario-numeros">
            {numeros.map((n) => (
              <div className="diario-par" key={n.rotulo}>
                <span className="diario-par-valor">{n.valor}</span>
                <span className="diario-par-rotulo">{n.rotulo}</span>
              </div>
            ))}
          </div>
        )}

        <Mapa dias={dias} maior={maiorDia} limiares={limiares} />

        {vazio && (
          <div className="diario-vazio">
            Ainda não há sessão nos últimos {DIAS} dias. Jogue com o controle ligado e esta
            página se preenche sozinha.
          </div>
        )}
      </section>

      <section className="cartao">
        <h2 className="diario-titulo">Quem come mais bateria</h2>
        <Ranking famintos={famintos} />
      </section>
    </>
  );
}

function Mapa({ dias, maior, limiares }: { dias: Dia[]; maior: number; limiares: Limiares }) {
  const [sob, setSob] = useState<Dia | null>(null);
  const [foco, setFoco] = useState(dias.length - 1);
  const celulas = useRef<(HTMLButtonElement | null)[]>([]);
  const enchimento = (new Date(dias[0].quando).getDay() + 6) % 7;

  const linhas: (Dia | null)[][] = [];
  const todas: (Dia | null)[] = [...Array<null>(enchimento).fill(null), ...dias];
  for (let i = 0; i < todas.length; i += 7) linhas.push(todas.slice(i, i + 7));

  const mover = (indice: number) => {
    const alvo = Math.min(dias.length - 1, Math.max(0, indice));
    setFoco(alvo);
    celulas.current[alvo]?.focus();
  };

  const aoTeclar = (e: React.KeyboardEvent, i: number) => {
    const destino = { ArrowLeft: i - 1, ArrowRight: i + 1, ArrowUp: i - 7, ArrowDown: i + 7 }[
      e.key
    ];
    if (e.key === "Home") mover(0);
    else if (e.key === "End") mover(dias.length - 1);
    else if (destino !== undefined) mover(destino);
    else return;
    e.preventDefault();
  };

  return (
    <>
      <div className="mapa" role="group" aria-label={`Últimos ${DIAS} dias`}>
        <span />
        {LETRAS_DA_SEMANA.map((l, i) => (
          <span key={i} className="mapa-semana" aria-hidden="true">
            {l}
          </span>
        ))}
        {linhas.map((linha, n) => {
          const primeiro = linha.find((d) => d && new Date(d.quando).getDate() === 1);
          const mes = primeiro
            ? nomeDoMes(primeiro)
            : n === 0
              ? nomeDoMes(linha.find((d) => d) ?? dias[0])
              : "";
          return [
            <span key={`mes-${n}`} className="mapa-mes" aria-hidden="true">
              {mes}
            </span>,
            ...linha.map((d, j) => {
              if (!d) return <span key={`vazio-${n}-${j}`} className="celula enchimento" />;
              const i = dias.indexOf(d);
              const hoje = i === dias.length - 1;
              return (
                <button
                  key={d.quando}
                  ref={(el) => {
                    celulas.current[i] = el;
                  }}
                  type="button"
                  className={[
                    "celula",
                    faixa(d.menorCarga, limiares),
                    `n${nivel(d.minutos, maior)}`,
                    hoje && "hoje",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  tabIndex={i === foco ? 0 : -1}
                  aria-label={legenda(d, limiares)}
                  onMouseEnter={() => setSob(d)}
                  onMouseLeave={() => setSob(null)}
                  onFocus={() => {
                    setSob(d);
                    setFoco(i);
                  }}
                  onBlur={() => setSob(null)}
                  onKeyDown={(e) => aoTeclar(e, i)}
                />
              );
            }),
          ];
        })}
      </div>

      <div className="mapa-rodape">
        <div className="mapa-leitura" aria-live="polite">
          {sob ? (
            <>
              <span className="mapa-leitura-dia">
                {diaDaSemana(sob.quando)}, {diaEMes(new Date(sob.quando))} ·{" "}
                {sob.minutos > 0 ? duracao(sob.minutos) : "sem sessão"}
              </span>
              <span className="mapa-leitura-resto">{restoDaLegenda(sob, limiares)}</span>
            </>
          ) : (
            <span className="mapa-cores">
              <i className="ponto cheia" /> terminou com {limiares.aviso}% ou mais
              <i className="ponto baixa" /> abaixo de {limiares.aviso}%
              <i className="ponto critica" /> abaixo de {limiares.critico}%
            </span>
          )}
        </div>
        <div className="mapa-escala" aria-hidden="true">
          menos
          {[0, 1, 2, 3, 4].map((n) => (
            <i key={n} className={`degrau n${n}`} />
          ))}
          mais
        </div>
      </div>
    </>
  );
}

function Ranking({ famintos }: { famintos: Faminto[] }) {
  if (famintos.length === 0) {
    return (
      <div className="diario-vazio">
        Nenhuma sessão trouxe o nome do jogo ainda. O nome é gravado quando o jogo roda em tela
        cheia com o controle ligado.
      </div>
    );
  }

  const firmes = famintos.filter((f) => f.sessoes >= SESSOES_PARA_ENTRAR_NO_RANKING);
  const medindo = famintos.filter((f) => f.sessoes < SESSOES_PARA_ENTRAR_NO_RANKING);
  const teto = firmes[0]?.porHora ?? 1;

  return (
    <div className="ranking">
      {firmes.map((f) => (
        <div className="faminto" key={f.jogo}>
          <IconeDoJogo jogo={f.jogo} tamanho={24} />
          <span className="faminto-nome">{f.jogo}</span>
          <span className="faminto-barra">
            {firmes.length > 1 && <span style={{ width: `${(f.porHora / teto) * 100}%` }} />}
          </span>
          <span className="faminto-taxa">{taxa(f.porHora)}</span>
          <span className="faminto-quantas">
            {duracao(f.minutos)} · {f.sessoes} sessões
          </span>
        </div>
      ))}
      {medindo.map((f) => (
        <div className="faminto medindo" key={f.jogo}>
          <IconeDoJogo jogo={f.jogo} tamanho={24} />
          <span className="faminto-nome">{f.jogo}</span>
          <span className="medicao" aria-hidden="true">
            {Array.from({ length: SESSOES_PARA_ENTRAR_NO_RANKING }, (_, i) => (
              <span key={i} className={i < f.sessoes ? "feita" : undefined} />
            ))}
          </span>
          <span className="faminto-taxa">medindo</span>
          <span className="faminto-quantas">
            {f.sessoes} de {SESSOES_PARA_ENTRAR_NO_RANKING} sessões
          </span>
        </div>
      ))}
    </div>
  );
}

function Exportar({
  dias,
  famintos,
  numeros,
  totalMinutos,
  limiares,
  desabilitado,
}: {
  dias: Dia[];
  famintos: Faminto[];
  numeros: Numero[];
  totalMinutos: number;
  limiares: Limiares;
  desabilitado: boolean;
}) {
  const [passo, setPasso] = useState<"parado" | "salvando" | "salvo" | "falhou">("parado");
  const [motivo, setMotivo] = useState("");
  const tela = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (passo !== "salvo" && passo !== "falhou") return;
    const voltar = window.setTimeout(() => setPasso("parado"), AVISO_DO_CARTAO_MS);
    return () => window.clearTimeout(voltar);
  }, [passo]);

  const salvar = async () => {
    setPasso("salvando");
    try {
      const icones = await Promise.all(
        famintos.slice(0, 3).map((f) =>
          invoke<string | null>("icone_do_jogo", { nome: f.jogo }).catch(() => null),
        ),
      );
      const png = await desenharCartao(tela.current, {
        dias,
        famintos,
        numeros,
        totalMinutos,
        limiares,
        icones,
      });
      await invoke("salvar_cartao", { png });
      setPasso("salvo");
      setMotivo("");
    } catch (e) {
      setPasso("falhou");
      setMotivo(typeof e === "string" ? e : "não deu para desenhar o cartão");
    }
  };

  return (
    <div className="exportar">
      <span
        className={`exportar-aviso${passo === "falhou" ? " falhou" : ""}`}
        role="status"
        title={passo === "falhou" ? motivo : undefined}
      >
        {passo === "salvo" ? "Salvo em Downloads" : passo === "falhou" ? "Não consegui salvar" : ""}
      </span>
      <button
        className="botao destaque"
        disabled={desabilitado || passo === "salvando"}
        title={desabilitado ? "Precisa de pelo menos uma sessão" : undefined}
        onClick={() => void salvar()}
      >
        {passo === "salvando" ? "Desenhando…" : "Salvar cartão"}
      </button>
      <canvas ref={tela} width={1200} height={630} style={{ display: "none" }} />
    </div>
  );
}

function porDia(sessoes: Sessao[]): Dia[] {
  const hoje = new Date();
  const meiaNoite = new Date(hoje.getFullYear(), hoje.getMonth(), hoje.getDate()).getTime();

  const dias: Dia[] = [];
  for (let i = DIAS - 1; i >= 0; i--) {
    const d = new Date(meiaNoite);
    d.setDate(d.getDate() - i);
    dias.push({ quando: d.getTime(), minutos: 0, menorCarga: null, jogos: [] });
  }

  for (const s of sessoes) {
    const inicio = new Date(s.inicio);
    const chave = new Date(inicio.getFullYear(), inicio.getMonth(), inicio.getDate()).getTime();
    const dia = dias.find((d) => d.quando === chave);
    if (!dia) continue;

    dia.minutos += Math.round((s.fim - s.inicio) / 60_000);
    dia.menorCarga = dia.menorCarga === null ? s.ate : Math.min(dia.menorCarga, s.ate);
    if (s.jogo && !dia.jogos.includes(s.jogo)) dia.jogos.push(s.jogo);
  }

  return dias;
}

function porJogo(sessoes: Sessao[]): Faminto[] {
  const contas = new Map<string, { gastou: number; minutos: number; sessoes: number }>();

  for (const s of sessoes) {
    if (!s.jogo) continue;
    const minutos = Math.round((s.fim - s.inicio) / 60_000);
    const gastou = s.de - s.ate;
    if (gastou <= 0 || minutos < MINUTOS_PARA_CONTAR_TAXA) continue;

    const atual = contas.get(s.jogo) ?? { gastou: 0, minutos: 0, sessoes: 0 };
    atual.gastou += gastou;
    atual.minutos += minutos;
    atual.sessoes += 1;
    contas.set(s.jogo, atual);
  }

  return [...contas.entries()]
    .map(([jogo, c]) => ({
      jogo,
      porHora: (c.gastou * 60) / c.minutos,
      sessoes: c.sessoes,
      minutos: c.minutos,
    }))
    .sort((a, b) => b.porHora - a.porHora);
}

function numerosDoMes(dias: Dia[], sessoes: Sessao[]): Numero[] {
  const dentro = sessoes.filter((s) => s.inicio >= dias[0].quando);
  const diasComJogo = dias.filter((d) => d.minutos > 0).length;
  const maisLonga = dentro.reduce(
    (maior, s) => Math.max(maior, Math.round((s.fim - s.inicio) / 60_000)),
    0,
  );
  return [
    { valor: `${diasComJogo} de ${DIAS}`, rotulo: "dias com jogo" },
    { valor: duracao(maisLonga), rotulo: "sessão mais longa" },
    { valor: String(dentro.length), rotulo: "sessões" },
  ];
}

function nivel(minutos: number, maior: number): number {
  if (minutos <= 0 || maior <= 0) return 0;
  return Math.min(4, Math.ceil((minutos / maior) * 4));
}

function faixa(carga: number | null, limiares: Limiares): string {
  if (carga === null) return "vazia";
  if (carga < limiares.critico) return "critica";
  if (carga < limiares.aviso) return "baixa";
  return "cheia";
}

function palavraDaFaixa(carga: number, limiares: Limiares): string {
  if (carga < limiares.critico) return "carga crítica";
  if (carga < limiares.aviso) return "carga baixa";
  return "com folga";
}

function diaDaSemana(ms: number): string {
  return new Date(ms).toLocaleDateString("pt-BR", { weekday: "short" }).replace(".", "");
}

function nomeDoMes(d: Dia): string {
  return new Date(d.quando).toLocaleDateString("pt-BR", { month: "short" }).replace(".", "");
}

function restoDaLegenda(d: Dia, limiares: Limiares): string {
  const partes: string[] = [];
  if (d.menorCarga !== null) {
    partes.push(`terminou em ${d.menorCarga}%, ${palavraDaFaixa(d.menorCarga, limiares)}`);
  }
  if (d.jogos.length > 0) partes.push(d.jogos.join(", "));
  return partes.length > 0 ? ` · ${partes.join(" · ")}` : "";
}

function legenda(d: Dia, limiares: Limiares): string {
  const dia = `${diaDaSemana(d.quando)}, ${diaEMes(new Date(d.quando))}`;
  if (d.minutos === 0) return `${dia} · sem sessão`;
  return `${dia} · ${duracao(d.minutos)}${restoDaLegenda(d, limiares)}`;
}

interface Retrato {
  dias: Dia[];
  famintos: Faminto[];
  numeros: Numero[];
  totalMinutos: number;
  limiares: Limiares;
  icones: (string | null)[];
}

function carregar(uri: string): Promise<HTMLImageElement | null> {
  return new Promise((resolver) => {
    const imagem = new Image();
    imagem.onload = () => resolver(imagem);
    imagem.onerror = () => resolver(null);
    imagem.src = uri;
  });
}

function encaixar(pincel: CanvasRenderingContext2D, texto: string, largura: number): string {
  if (pincel.measureText(texto).width <= largura) return texto;
  let corte = texto;
  while (corte.length > 1 && pincel.measureText(`${corte}…`).width > largura) {
    corte = corte.slice(0, -1);
  }
  return `${corte.trimEnd()}…`;
}

function desenharMarca(pincel: CanvasRenderingContext2D, x: number, y: number, lado: number) {
  const centro = CAIXA / 2;
  pincel.save();
  pincel.translate(x, y);
  pincel.scale(lado / CAIXA, lado / CAIXA);

  pincel.fillStyle = CORES.fundo;
  pincel.beginPath();
  pincel.arc(centro, centro, centro, 0, Math.PI * 2);
  pincel.fill();

  pincel.lineWidth = APP.anelLargura;
  pincel.globalAlpha = APP.trilhoOpacidade;
  pincel.strokeStyle = CORES.branco;
  pincel.beginPath();
  pincel.arc(centro, centro, APP.anelRaio, 0, Math.PI * 2);
  pincel.stroke();

  pincel.globalAlpha = 1;
  pincel.strokeStyle = CORES.verde;
  pincel.lineCap = "round";
  const inicio = -Math.PI / 2;
  pincel.beginPath();
  pincel.arc(centro, centro, APP.anelRaio, inicio, inicio + (APP.anelVarredura * Math.PI) / 180);
  pincel.stroke();

  pincel.translate(centro, APP.padCentroY);
  pincel.scale(APP.padEscala, APP.padEscala);
  pincel.translate(-centro, -288);
  pincel.fillStyle = CORES.glifoClaro;
  pincel.fill(new Path2D(PAD));
  pincel.fillStyle = CORES.fundo;
  for (const [cx, cy] of [APP.stickEsq, APP.stickDir]) {
    pincel.beginPath();
    pincel.arc(cx, cy, APP.stickRaio, 0, Math.PI * 2);
    pincel.fill();
  }
  pincel.restore();
}

function desenharGlifo(pincel: CanvasRenderingContext2D, x: number, y: number, lado: number, cor: string) {
  pincel.save();
  pincel.translate(x, y);
  pincel.scale(lado / 412, lado / 412);
  pincel.translate(-50, -158 + (412 - 260) / 2);
  pincel.fillStyle = cor;
  pincel.fill(new Path2D(PAD_COM_STICKS_VAZADOS), "evenodd");
  pincel.restore();
}

async function desenharCartao(tela: HTMLCanvasElement | null, r: Retrato): Promise<string> {
  if (!tela) throw new Error("sem canvas");
  const pincel = tela.getContext("2d");
  if (!pincel) throw new Error("sem contexto");

  const estilo = getComputedStyle(document.body);
  const cor = (nome: string) => estilo.getPropertyValue(nome).trim() || "#9aa4ad";
  const display = "'Segoe UI Variable Display', 'Segoe UI', sans-serif";
  const texto = "'Segoe UI Variable Text', 'Segoe UI', sans-serif";
  const corDaFaixa: Record<string, string> = {
    cheia: cor("--accent-green"),
    baixa: cor("--amber"),
    critica: cor("--red"),
  };

  pincel.fillStyle = cor("--ink");
  pincel.fillRect(0, 0, 1200, 630);

  pincel.fillStyle = cor("--text-primary");
  pincel.font = `600 76px ${display}`;
  pincel.fillText(duracao(r.totalMinutos), 72, 156);

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `400 26px ${texto}`;
  pincel.fillText(
    `de jogo de ${diaEMes(new Date(r.dias[0].quando))} a ${diaEMes(new Date(r.dias[r.dias.length - 1].quando))}`,
    72,
    200,
  );

  const lado = 44;
  const folga = 10;
  const maior = r.dias.reduce((m, d) => Math.max(m, d.minutos), 1);
  r.dias.forEach((d, i) => {
    const x = 72 + (i % 10) * (lado + folga);
    const y = 268 + Math.floor(i / 10) * (lado + folga);
    const n = nivel(d.minutos, maior);
    pincel.globalAlpha = n > 0 ? BRILHO_POR_NIVEL[n] : 0.14;
    pincel.fillStyle =
      n > 0 ? corDaFaixa[faixa(d.menorCarga, r.limiares)] ?? cor("--accent-green") : cor("--stroke-strong");
    pincel.beginPath();
    pincel.roundRect(x, y, lado, lado, Math.round((lado * 8) / 28));
    pincel.fill();
  });
  pincel.globalAlpha = 1;

  const comRanking = r.famintos.length > 0;
  const linhas = comRanking
    ? r.famintos.slice(0, 3).map((f) => ({ titulo: f.jogo, valor: taxa(f.porHora) }))
    : r.numeros.map((n) => ({ titulo: n.rotulo, valor: n.valor }));
  const icones = comRanking ? await Promise.all(r.icones.map((u) => (u ? carregar(u) : null))) : [];

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `600 19px ${texto}`;
  pincel.fillText(comRanking ? "QUEM COME MAIS BATERIA" : "O MÊS EM NÚMEROS", 700, 288);

  linhas.forEach((linha, i) => {
    const y = 340 + i * 66;
    const xTexto = comRanking ? 744 : 700;
    if (comRanking) {
      const icone = icones[i];
      if (icone) pincel.drawImage(icone, 700, y - 26, 32, 32);
      else desenharGlifo(pincel, 700, y - 26, 32, cor("--text-tertiary"));
    }
    pincel.fillStyle = cor("--text-primary");
    pincel.font = `500 30px ${texto}`;
    pincel.fillText(encaixar(pincel, linha.titulo, 1128 - xTexto), xTexto, y);

    pincel.fillStyle = cor("--text-secondary");
    pincel.font = "400 24px Consolas, monospace";
    pincel.fillText(linha.valor, xTexto, y + 34);
  });

  pincel.strokeStyle = cor("--stroke");
  pincel.lineWidth = 1;
  pincel.beginPath();
  pincel.moveTo(72, 528);
  pincel.lineTo(1128, 528);
  pincel.stroke();

  desenharMarca(pincel, 72, 550, 32);

  pincel.fillStyle = cor("--text-secondary");
  pincel.font = `600 22px ${texto}`;
  pincel.fillText("Kontro", 116, 574);

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `400 18px ${texto}`;
  pincel.fillText(encaixar(pincel, "bateria do seu controle na bandeja", 360), 196, 574);

  const uri = tela.toDataURL("image/png");
  return uri.slice(uri.indexOf(",") + 1);
}
