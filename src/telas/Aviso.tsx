import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Estado, corDoAnel, useLimiares } from "../estado";
import "./aviso.css";

type Assunto = "Conectou" | "Desconectou" | "TrocouDeVia";

interface Pacote {
  assunto: Assunto;
  estado: Estado;
}

const PERMANENCIA_MS = 4000;

const SAIDA_MS = 240;

export function Aviso() {
  const limiares = useLimiares();
  const [pacote, setPacote] = useState<Pacote | null>(null);
  const [saindo, setSaindo] = useState(false);
  const [rodada, setRodada] = useState(0);
  useEffect(() => {
    const parar = listen<Pacote>("kontro://aviso", (evento) => {
      setPacote(evento.payload);
      setSaindo(false);
      setRodada((n) => n + 1);
    });
    return () => {
      void parar.then((f) => f());
    };
  }, []);
  useEffect(() => {
    if (!pacote) return;
    const aSair = window.setTimeout(() => setSaindo(true), PERMANENCIA_MS);
    const aFechar = window.setTimeout(() => {
      void invoke("esconder_janela", { rotulo: "aviso" });
    }, PERMANENCIA_MS + SAIDA_MS);
    return () => {
      window.clearTimeout(aSair);
      window.clearTimeout(aFechar);
    };
  }, [rodada, pacote]);
  if (!pacote) return null;
  const { assunto, estado } = pacote;

  return (
    <div className="aviso">
      <div key={rodada} className={`cartao${saindo ? " saindo" : ""}`}>
        <Anel
          valor={estado.preenchimento}
          cor={assunto === "Desconectou" ? "var(--gray)" : corDoAnel(estado, limiares)}
          espessura={46}
          tamanho={44}
          girando={assunto !== "Desconectou" && estado.girando}
        >
          {estado.temNumero && assunto !== "Desconectou" ? (
            <span className="pct">{estado.percent}%</span>
          ) : (
            <Glifo tamanho={18} cor="var(--text-secondary)" />
          )}
        </Anel>
        <div className="texto">
          <div className="nome">{estado.deviceName}</div>
          <div className="linha">{legenda(assunto, estado)}</div>
        </div>
      </div>
    </div>
  );
}

function legenda(assunto: Assunto, estado: Estado): string {
  if (assunto === "Desconectou") {
    // A ultima carga conhecida vale como referencia -- "desligou com 68%" e uma
    // informacao util -- desde que fique claro que e memoria, nao medida.
    return estado.preenchimento !== null
      ? `desconectado · ${estado.textoDaCarga} na última leitura`
      : "desconectado";
  }

  const abertura = assunto === "TrocouDeVia" ? "agora" : "conectado";
  if (estado.girando) {
    return estado.charging ? `${abertura} no cabo · carregando` : `${abertura} no cabo`;
  }
  if (estado.temNumero) return `${abertura} · ${estado.textoDaLigacao}`;
  if (estado.preenchimento !== null) {
    return `${estado.textoDaCarga} · ${estado.textoDaLigacao}`;
  }
  return `${abertura} · ${estado.textoDaLigacao} · sem leitura de bateria`;
}
