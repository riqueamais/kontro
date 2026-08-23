import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Amostra, Estado, corDoAnel, useEstado } from "../estado";
import "./painel.css";

/** Faixas de cor do anel, as mesmas gravadas nos icones. */
export function Painel() {
  const estado = useEstado();
  const [serie, setSerie] = useState<Amostra[]>([]);

  useEffect(() => {
    invoke<Amostra[]>("serie_do_historico").then(setSerie).catch(() => {});
  }, [estado?.key, estado?.percent]);

  // O painel e um relance, nao uma janela para deixar aberta. Ele some de tres jeitos,
  // e sao tres de proposito: o foco nem sempre chega a uma janela sem moldura, e um
  // painel que fica preso na tela e pior que um painel que fecha demais.
  useEffect(() => {
    const janela = getCurrentWindow();
    const fechar = () => void janela.hide();

    const parar = janela.onFocusChanged(({ payload: temFoco }) => {
      if (!temFoco) fechar();
    });

    const aoPerderJanela = () => fechar();
    const aoTeclar = (e: KeyboardEvent) => {
      if (e.key === "Escape") fechar();
    };
    window.addEventListener("blur", aoPerderJanela);
    window.addEventListener("keydown", aoTeclar);

    return () => {
      void parar.then((f) => f());
      window.removeEventListener("blur", aoPerderJanela);
      window.removeEventListener("keydown", aoTeclar);
    };
  }, []);

  if (!estado) return null;

  const rodape = estado.readAt
    ? `${estado.stale ? "lido" : "atualizado"} às ${new Date(estado.readAt).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}`
    : "sem leitura ainda";

  return (
    <div className="painel">
      <button
        className="fechar"
        aria-label="Fechar"
        title="Fechar (Esc)"
        onClick={() => void getCurrentWindow().hide()}
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path
            d="M1 1 L9 9 M9 1 L1 9"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
          />
        </svg>
      </button>

      <div className="topo">
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado)}
          espessura={38}
          tamanho={96}
          girando={estado.girando}
        >
          {estado.temNumero ? (
            <span className="numero">{estado.percent}%</span>
          ) : (
            <Glifo tamanho={38} cor="var(--text-secondary)" />
          )}
        </Anel>

        <div className="leitura">
          <div className="dispositivo">
            {estado.mode === "Offline" ? "Desconectado" : estado.deviceName}
          </div>
          <div className="detalhe">{detalhe(estado)}</div>
          <div className="rodape">{rodape}</div>
        </div>
      </div>

      <Historico serie={serie} cor={corDoAnel(estado)} />

      <div className="acoes">
        <button onClick={() => invoke("mostrar_janela", { rotulo: "principal" })}>
          Configurações
        </button>
        <button onClick={() => invoke("ler_agora")}>Atualizar</button>
      </div>
    </div>
  );
}

function detalhe(estado: Estado) {
  if (estado.mode === "Offline") return estado.deviceName;
  if (estado.conectadoSemCarga)
    return `conectado ${estado.textoDaLigacao} · não informa bateria`;
  if (estado.precisao === "Aproximada")
    return `${estado.textoDaCarga} · sem percentual neste controle`;
  // a autonomia e o que o usuario quer saber de fato; a via so importa quando nao ha
  return estado.autonomia ?? estado.textoDaLigacao;
}

/**
 * O historico e contexto, nao protagonista: linha de um pixel, sem eixo e sem grade.
 */
function Historico({ serie, cor }: { serie: Amostra[]; cor: string }) {
  if (serie.length < 2) return <div className="historico vazio">medindo o consumo</div>;

  const largura = 320;
  const altura = 56;
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
      <svg width="100%" height={altura} viewBox={`0 0 ${largura} ${altura}`} preserveAspectRatio="none">
        <polyline points={pontos} fill="none" stroke={cor} strokeWidth="1.5" strokeLinejoin="round" />
      </svg>
    </div>
  );
}
