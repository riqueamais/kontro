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

export function Painel() {
  const estado = useEstado();
  const limiares = useLimiares();
  const [serie, setSerie] = useState<Amostra[]>([]);
  const [aberto, setAberto] = useState(false);
  const [aberturas, setAberturas] = useState(0);
  const [saindo, setSaindo] = useState(false);
  const painel = useRef<HTMLDivElement>(null);

  const chegou = estado !== null;
  useEffect(() => {
    if (chegou) void invoke("janela_pronta", { rotulo: "painel" });
  }, [chegou]);

  useEffect(() => {
    const parar = listen("kontro://painel-abriu", () => {
      setAberto(true);
      setSaindo(false);
      setAberturas((n) => n + 1);
    });
    return () => {
      void parar.then((f) => f());
    };
  }, []);

  useAoMudarOHistorico(() => {
    if (!aberto) return;
    invoke<Amostra[]>("serie_do_historico").then(setSerie).catch(() => {});
  }, [estado?.chave, aberto]);

  const fechar = useCallback(() => {
    setAberto(false);
    setSaindo(true);
  }, []);

  useEffect(() => {
    const janela = getCurrentWindow();
    const parar = janela.onFocusChanged(({ payload: temFoco }) => {
      if (!temFoco) fechar();
    });

    const aoTeclar = (e: KeyboardEvent) => {
      if (e.key === "Escape") fechar();
    };
    window.addEventListener("blur", fechar);
    window.addEventListener("keydown", aoTeclar);

    return () => {
      void parar.then((f) => f());
      window.removeEventListener("blur", fechar);
      window.removeEventListener("keydown", aoTeclar);
    };
  }, [fechar]);

  useEffect(() => {
    const alvo = painel.current;
    if (!alvo || !aberto) return;

    const observador = new ResizeObserver(() => {
      const estilo = getComputedStyle(alvo);
      const folga =
        parseFloat(estilo.marginTop || "0") + parseFloat(estilo.marginBottom || "0");
      void invoke("ajustar_altura_do_painel", {
        altura: Math.ceil(alvo.offsetHeight + folga),
      });
    });
    observador.observe(alvo);
    return () => observador.disconnect();
  }, [aberto, aberturas, chegou]);

  if (!estado) return null;

  return (
    <div
      key={aberturas}
      className={saindo ? "painel saindo" : "painel"}
      ref={painel}
      onAnimationEnd={(e) => {
        if (saindo && e.target === e.currentTarget) {
          setSaindo(false);
          void getCurrentWindow().hide();
        }
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
        <button onClick={() => invoke("mostrar_janela", { rotulo: "principal" })}>
          Configurações
        </button>
        <BotaoLerAgora estado={estado} />
      </div>
    </div>
  );
}
