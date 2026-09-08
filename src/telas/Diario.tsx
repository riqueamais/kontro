import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";

import { Sessao, useLimiares } from "../estado";
import { diaEMes, duracao } from "../formato";
import "./diario.css";

const DIA = 86_400_000;
const DIAS = 30;
const SESSOES_PARA_ENTRAR_NO_RANKING = 3;
const MINUTOS_PARA_CONTAR_TAXA = 20;

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

export function Diario() {
  const [sessoes, setSessoes] = useState<Sessao[]>([]);
  const limiares = useLimiares();

  useEffect(() => {
    invoke<Sessao[]>("sessoes_do_controle").then(setSessoes).catch(() => {});
  }, []);

  const dias = useMemo(() => porDia(sessoes), [sessoes]);
  const famintos = useMemo(() => porJogo(sessoes), [sessoes]);

  const totalMinutos = dias.reduce((soma, d) => soma + d.minutos, 0);
  const maiorDia = dias.reduce((maior, d) => Math.max(maior, d.minutos), 0);

  return (
    <>
      <h1 className="titulo-da-pagina">Diário</h1>

      {totalMinutos === 0 ? (
        <section className="cartao">
          <div className="diario-vazio">
            Ainda não há sessão suficiente nos últimos {DIAS} dias. Jogue com o controle
            ligado e esta página se preenche sozinha.
          </div>
        </section>
      ) : (
        <>
          <section className="cartao">
            <div className="diario-cabeca">
              <div>
                <div className="diario-numero">{duracao(totalMinutos)}</div>
                <div className="diario-rotulo">de jogo em {DIAS} dias</div>
              </div>
              <Exportar
                dias={dias}
                famintos={famintos}
                sessoes={sessoes}
                totalMinutos={totalMinutos}
              />
            </div>

            <Mapa dias={dias} maior={maiorDia} limiares={limiares} />
          </section>

          <section className="cartao">
            <h2 className="diario-titulo">Quem come mais bateria</h2>
            <Ranking famintos={famintos} />
          </section>
        </>
      )}
    </>
  );
}

function Mapa({
  dias,
  maior,
  limiares,
}: {
  dias: Dia[];
  maior: number;
  limiares: { critico: number; aviso: number };
}) {
  return (
    <div className="mapa">
      {dias.map((d) => (
        <div
          key={d.quando}
          className={`celula ${faixa(d.menorCarga, limiares)}`}
          style={{ opacity: d.minutos > 0 ? 0.25 + 0.75 * (d.minutos / maior) : 1 }}
          title={legenda(d)}
        />
      ))}
    </div>
  );
}

function Ranking({ famintos }: { famintos: Faminto[] }) {
  const firmes = famintos.filter((f) => f.sessoes >= SESSOES_PARA_ENTRAR_NO_RANKING);
  const medindo = famintos.filter((f) => f.sessoes < SESSOES_PARA_ENTRAR_NO_RANKING);

  if (firmes.length === 0) {
    return (
      <div className="diario-vazio">
        {medindo.length === 0
          ? "Nenhuma sessão trouxe o nome do jogo ainda."
          : `${medindo.length} ${medindo.length === 1 ? "jogo" : "jogos"} em medição. Cada um precisa de ${SESSOES_PARA_ENTRAR_NO_RANKING} sessões antes de virar média.`}
      </div>
    );
  }

  const teto = firmes[0].porHora;

  return (
    <div className="ranking">
      {firmes.map((f) => (
        <div className="faminto" key={f.jogo}>
          <span className="faminto-nome">{f.jogo}</span>
          <span className="faminto-barra">
            <span style={{ width: `${(f.porHora / teto) * 100}%` }} />
          </span>
          <span className="faminto-taxa">{f.porHora.toFixed(1).replace(".", ",")} %/h</span>
          <span className="faminto-quantas">
            {f.sessoes} {f.sessoes === 1 ? "sessão" : "sessões"}
          </span>
        </div>
      ))}
      {medindo.length > 0 && (
        <div className="diario-vazio">
          Mais {medindo.length} em medição, com menos de {SESSOES_PARA_ENTRAR_NO_RANKING} sessões.
        </div>
      )}
    </div>
  );
}

function Exportar({
  dias,
  famintos,
  sessoes,
  totalMinutos,
}: {
  dias: Dia[];
  famintos: Faminto[];
  sessoes: Sessao[];
  totalMinutos: number;
}) {
  const [passo, setPasso] = useState<"parado" | "salvando" | "salvo" | "falhou">("parado");
  const [motivo, setMotivo] = useState("");
  const tela = useRef<HTMLCanvasElement>(null);

  const salvar = async () => {
    setPasso("salvando");
    try {
      const png = await desenharCartao(tela.current, {
        dias,
        famintos,
        sessoes,
        totalMinutos,
      });
      await invoke("salvar_cartao", { png });
      setPasso("salvo");
      setMotivo("");
    } catch (e) {
      setPasso("falhou");
      setMotivo(String(e));
    }
  };

  return (
    <div className="exportar">
      <button
        className="ciclo destaque"
        disabled={passo === "salvando"}
        onClick={() => void salvar()}
      >
        {rotuloDoBotao(passo)}
      </button>
      {passo === "falhou" && <div className="exportar-erro">{motivo}</div>}
      <canvas ref={tela} width={1200} height={630} style={{ display: "none" }} />
    </div>
  );
}

function rotuloDoBotao(passo: string): string {
  if (passo === "salvando") return "Desenhando…";
  if (passo === "salvo") return "Salvo em Downloads";
  if (passo === "falhou") return "Não deu";
  return "Salvar cartão";
}

function porDia(sessoes: Sessao[]): Dia[] {
  const hoje = new Date();
  const meiaNoite = new Date(hoje.getFullYear(), hoje.getMonth(), hoje.getDate()).getTime();

  const dias: Dia[] = [];
  for (let i = DIAS - 1; i >= 0; i--) {
    dias.push({ quando: meiaNoite - i * DIA, minutos: 0, menorCarga: null, jogos: [] });
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

function faixa(carga: number | null, limiares: { critico: number; aviso: number }): string {
  if (carga === null) return "vazia";
  if (carga < limiares.critico) return "critica";
  if (carga < limiares.aviso) return "baixa";
  return "cheia";
}

function legenda(d: Dia): string {
  const quando = diaEMes(new Date(d.quando));
  if (d.minutos === 0) return `${quando} · sem sessão`;
  const jogos = d.jogos.length > 0 ? ` · ${d.jogos.join(", ")}` : "";
  return `${quando} · ${duracao(d.minutos)}${jogos}`;
}

interface Retrato {
  dias: Dia[];
  famintos: Faminto[];
  sessoes: Sessao[];
  totalMinutos: number;
}

async function desenharCartao(tela: HTMLCanvasElement | null, r: Retrato): Promise<string> {
  if (!tela) throw new Error("sem canvas");
  const pincel = tela.getContext("2d");
  if (!pincel) throw new Error("sem contexto");

  const estilo = getComputedStyle(document.body);
  const cor = (nome: string) => estilo.getPropertyValue(nome).trim() || "#5fe083";
  const display = "'Segoe UI Variable Display', 'Segoe UI', sans-serif";
  const texto = "'Segoe UI Variable Text', 'Segoe UI', sans-serif";

  pincel.fillStyle = cor("--ink");
  pincel.fillRect(0, 0, 1200, 630);

  pincel.fillStyle = cor("--text-primary");
  pincel.font = `600 76px ${display}`;
  pincel.fillText(duracao(r.totalMinutos), 72, 156);

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `400 26px ${texto}`;
  pincel.fillText(`de jogo nos últimos ${DIAS} dias`, 72, 200);

  const lado = 44;
  const folga = 10;
  const maior = r.dias.reduce((m, d) => Math.max(m, d.minutos), 1);
  r.dias.forEach((d, i) => {
    const x = 72 + (i % 10) * (lado + folga);
    const y = 268 + Math.floor(i / 10) * (lado + folga);
    pincel.globalAlpha = d.minutos > 0 ? 0.3 + 0.7 * (d.minutos / maior) : 0.14;
    pincel.fillStyle = d.minutos > 0 ? cor("--accent-green") : cor("--stroke-strong");
    pincel.beginPath();
    pincel.roundRect(x, y, lado, lado, 9);
    pincel.fill();
  });
  pincel.globalAlpha = 1;

  const linhas =
    r.famintos.length > 0
      ? r.famintos.slice(0, 3).map((f) => ({
          titulo: f.jogo,
          valor: `${f.porHora.toFixed(1).replace(".", ",")} %/h`,
        }))
      : numerosDoMes(r);

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `600 19px ${texto}`;
  pincel.fillText(
    r.famintos.length > 0 ? "QUEM COME MAIS BATERIA" : "O MÊS EM NÚMEROS",
    700,
    288,
  );

  linhas.forEach((linha, i) => {
    const y = 340 + i * 66;
    pincel.fillStyle = cor("--text-primary");
    pincel.font = `500 30px ${texto}`;
    pincel.fillText(linha.titulo, 700, y);

    pincel.fillStyle = cor("--accent-green");
    pincel.font = "400 24px Consolas, monospace";
    pincel.fillText(linha.valor, 700, y + 34);
  });

  pincel.strokeStyle = cor("--stroke");
  pincel.lineWidth = 1;
  pincel.beginPath();
  pincel.moveTo(72, 528);
  pincel.lineTo(1128, 528);
  pincel.stroke();

  pincel.fillStyle = cor("--text-secondary");
  pincel.font = `600 22px ${texto}`;
  pincel.fillText("Kontro", 72, 574);

  pincel.fillStyle = cor("--text-tertiary");
  pincel.font = `400 18px ${texto}`;
  pincel.fillText("bateria do seu controle na bandeja", 152, 574);

  const uri = tela.toDataURL("image/png");
  return uri.slice(uri.indexOf(",") + 1);
}

function numerosDoMes(r: Retrato): { titulo: string; valor: string }[] {
  const diasComJogo = r.dias.filter((d) => d.minutos > 0).length;
  const maisLonga = r.sessoes.reduce(
    (maior, s) => Math.max(maior, Math.round((s.fim - s.inicio) / 60_000)),
    0,
  );

  return [
    { titulo: "Sessões", valor: String(r.sessoes.length) },
    { titulo: "Dias com jogo", valor: `${diasComJogo} de ${DIAS}` },
    { titulo: "Sessão mais longa", valor: duracao(maisLonga) },
  ];
}
