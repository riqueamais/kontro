import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Historico } from "../componentes/Historico";
import { ListaDeControles } from "../componentes/ListaDeControles";
import { Amostra, Estado, corDoAnel, quandoLeu, useEstado, useLimiares } from "../estado";
import "./painel.css";

export function Painel() {
  const estado = useEstado();
  const limiares = useLimiares();
  const [serie, setSerie] = useState<Amostra[]>([]);
  const painel = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<Amostra[]>("serie_do_historico").then(setSerie).catch(() => {});
  }, [estado?.key, estado?.percent]);

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

  useEffect(() => {
    const alvo = painel.current;
    if (!alvo) return;

    const medir = () => {
      const estilo = getComputedStyle(alvo);
      const folga =
        parseFloat(estilo.marginTop || "0") + parseFloat(estilo.marginBottom || "0");
      void invoke("ajustar_altura_do_painel", {
        altura: Math.ceil(alvo.offsetHeight + folga),
      });
    };

    medir();
    const observador = new ResizeObserver(medir);
    observador.observe(alvo);
    return () => observador.disconnect();
  }, [estado, serie.length]);

  if (!estado) return null;

  return (
    <div className="painel" ref={painel}>
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
          cor={corDoAnel(estado, limiares)}
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
          <div className="rodape">{quandoLeu(estado)}</div>
        </div>
      </div>

      <Historico serie={serie} compacto />

      <ListaDeControles principal={estado.key} />

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
