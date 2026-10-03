import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";

import { Anel } from "../componentes/Anel";
import { BotaoLerAgora } from "../componentes/BotaoLerAgora";
import { Glifo } from "../componentes/Glifo";
import { Historico } from "../componentes/Historico";
import { ListaDeControles } from "../componentes/ListaDeControles";
import { Amostra, corDoAnel, useAoMudarOHistorico, useEstado, useLimiares } from "../estado";
import { detalhe, quandoLeu } from "../formato";
import "./painel.css";

type Fase = "guardado" | "aberto" | "saindo";

const PRAZO_DA_SAIDA_MS = 400;

export function Painel() {
  const estado = useEstado();
  const limiares = useLimiares();
  const [serie, setSerie] = useState<Amostra[]>([]);
  const [fase, setFase] = useState<Fase>("guardado");
  const [aberturas, setAberturas] = useState(0);
  const [remontagens, setRemontagens] = useState(0);
  const [ouvindo, setOuvindo] = useState(false);
  const painel = useRef<HTMLDivElement>(null);
  const faseAgora = useRef<Fase>("guardado");
  const saida = useRef<number | undefined>(undefined);

  const aberto = fase === "aberto";
  const chegou = estado !== null;

  const mudar = useCallback((nova: Fase) => {
    faseAgora.current = nova;
    setFase(nova);
  }, []);

  useEffect(() => {
    let vivo = true;

    const parar = listen("kontro://painel-abriu", () => {
      if (!vivo) return;
      window.clearTimeout(saida.current);
      if (faseAgora.current === "aberto") setRemontagens((n) => n + 1);
      mudar("aberto");
      setAberturas((n) => n + 1);
    });

    void parar.then(() => {
      if (!vivo) return;
      setOuvindo(true);
      void getCurrentWindow()
        .isVisible()
        .then((visivel) => {
          if (!vivo || !visivel || faseAgora.current !== "guardado") return;
          mudar("aberto");
          setAberturas((n) => Math.max(n, 1));
        });
    });

    return () => {
      vivo = false;
      window.clearTimeout(saida.current);
      void parar.then((f) => f());
    };
  }, [mudar]);

  useEffect(() => {
    if (chegou && ouvindo) void invoke("janela_pronta", { rotulo: "painel" });
  }, [chegou, ouvindo]);

  const guardar = useCallback(() => {
    window.clearTimeout(saida.current);
    if (faseAgora.current !== "saindo") return;
    mudar("guardado");
    void getCurrentWindow().hide();
  }, [mudar]);

  const fechar = useCallback(() => {
    if (faseAgora.current !== "aberto") return;
    mudar("saindo");
    window.clearTimeout(saida.current);
    saida.current = window.setTimeout(guardar, PRAZO_DA_SAIDA_MS);
  }, [mudar, guardar]);

  useEffect(() => {
    const parar = listen("kontro://painel-fechar", fechar);

    const aoTeclar = (e: KeyboardEvent) => {
      if (e.key === "Escape") fechar();
    };
    window.addEventListener("keydown", aoTeclar);

    return () => {
      void parar.then((f) => f());
      window.removeEventListener("keydown", aoTeclar);
    };
  }, [fechar]);

  useAoMudarOHistorico(() => {
    if (!aberto) return;
    invoke<Amostra[]>("serie_do_historico").then(setSerie).catch(() => {});
  }, [estado?.chave, aberto, aberturas]);

  useEffect(() => {
    const alvo = painel.current;
    if (!aberto || !alvo) return;

    const medir = () => {
      const estilo = getComputedStyle(alvo);
      const folga =
        parseFloat(estilo.marginTop || "0") + parseFloat(estilo.marginBottom || "0");
      void invoke("ajustar_altura_do_painel", {
        altura: Math.ceil(alvo.offsetHeight + folga),
      });
    };

    const observador = new ResizeObserver(medir);
    observador.observe(alvo);
    return () => observador.disconnect();
  }, [aberto, aberturas, chegou]);

  if (!estado) return null;

  return (
    <div
      key={remontagens}
      className={fase === "aberto" ? "painel" : `painel ${fase}`}
      ref={painel}
      onAnimationEnd={(e) => {
        if (e.target === e.currentTarget && e.animationName === "kontro-descer") guardar();
      }}
    >
      <button className="fechar" aria-label="Fechar" title="Fechar (Esc)" onClick={fechar}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path
            d="M1 1 L9 9 M9 1 L1 9"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
          />
        </svg>
      </button>

      <div
        className="topo"
        style={{ "--cor-do-estado": corDoAnel(estado, limiares) } as React.CSSProperties}
      >
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado, limiares)}
          espessura={38}
          tamanho={96}
          girando={estado.girando}
          marcas={estado.temNumero ? limiares : null}
        >
          {estado.temNumero ? (
            <span className="numero">{estado.percentual}%</span>
          ) : (
            <Glifo tamanho={38} cor="var(--text-secondary)" />
          )}
        </Anel>

        <div className="leitura">
          <div className="dispositivo">
            {estado.via === "Desligado" ? "Desconectado" : estado.nome}
          </div>
          <div className="detalhe">{detalhe(estado)}</div>
          <div className="rodape">{quandoLeu(estado)}</div>
        </div>
      </div>

      <Historico serie={serie} compacto autonomiaMinutos={estado.autonomiaMinutos} />

      <ListaDeControles principal={estado.chave} />

      <div className="acoes">
        <button onClick={() => invoke("abrir_aba", { aba: "config" })}>
          Configurações
        </button>
        <BotaoLerAgora estado={estado} />
      </div>
    </div>
  );
}
