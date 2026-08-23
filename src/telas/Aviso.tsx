import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Estado, corDoAnel } from "../estado";
import "./aviso.css";

type Assunto = "Conectou" | "Desconectou" | "TrocouDeVia";

interface Pacote {
  assunto: Assunto;
  estado: Estado;
}

/** Quanto tempo o aviso fica na tela depois de pronto. */
const PERMANENCIA_MS = 4000;

/** Precisa bater com a duracao da animacao de saida no CSS. */
const SAIDA_MS = 240;

/**
 * O aviso que aparece no topo quando algo muda na ligacao.
 *
 * Ele nasce pronto: quem decide a hora de mostrar e o lado do Rust, que espera a carga
 * existir antes de pedir. Aparecer dizendo "lendo" e trocar o texto no meio era pior que
 * aparecer um segundo depois.
 *
 * Quem apaga a janela e esta tela, e nao o Rust: so aqui se sabe quando a animacao de
 * saida terminou, e esconder antes disso faria o aviso sumir no meio do gesto.
 */
export function Aviso() {
  const [pacote, setPacote] = useState<Pacote | null>(null);
  const [saindo, setSaindo] = useState(false);
  // muda a cada aviso para reiniciar a animacao de entrada mesmo com a janela ja aberta
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
          cor={assunto === "Desconectou" ? "var(--gray)" : corDoAnel(estado)}
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
