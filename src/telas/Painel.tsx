import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";

import { Anel } from "../componentes/Anel";
import { BotaoLerAgora } from "../componentes/BotaoLerAgora";
import { Glifo } from "../componentes/Glifo";
import { Historico } from "../componentes/Historico";
import { Marca } from "../componentes/Marca";
import { Leitura } from "../componentes/Leitura";
import { ListaDeControles } from "../componentes/ListaDeControles";
import { MODOS, ORDEM_DOS_MODOS, salvar } from "../ajustes";
import {
  Amostra,
  corDoAnel,
  useAoMudarOHistorico,
  useConfig,
  useEstado,
  useLimiares,
} from "../estado";
import { rotuloDoAnel } from "../formato";
import "./painel.css";

type Fase = "guardado" | "aberto" | "saindo";

const PRAZO_DA_SAIDA_MS = 400;

export function Painel() {
  const estado = useEstado();
  const cfg = useConfig();
  const limiares = useLimiares();
  const [erroAoSalvar, setErroAoSalvar] = useState(false);
  const [salvando, setSalvando] = useState(false);
  const [serie, setSerie] = useState<Amostra[]>([]);
  const [fase, setFase] = useState<Fase>("guardado");
  const [aberturas, setAberturas] = useState(0);
  const [remontagens, setRemontagens] = useState(0);
  const [ouvindo, setOuvindo] = useState(false);
  const painel = useRef<HTMLDivElement>(null);
  const conteudo = useRef<HTMLDivElement>(null);
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
    if (aberto) painel.current?.focus();
  }, [aberto, aberturas]);

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
    const moldura = painel.current;
    const alvo = conteudo.current;
    if (!aberto || !moldura || !alvo) return;

    const medir = () => {
      const bordas = moldura.offsetHeight - moldura.clientHeight;
      void invoke("ajustar_altura_do_painel", {
        altura: alvo.offsetHeight + bordas,
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
      role="dialog"
      aria-label={`Kontro, ${estado.textoDaCarga || estado.titulo}, ${estado.nome}`}
      tabIndex={-1}
      onAnimationEnd={(e) => {
        if (e.target === e.currentTarget && e.animationName === "kontro-descer") guardar();
      }}
    >
      <div className="conteudo" ref={conteudo}>
        <header className="cabeca-do-painel">
          <span className="identidade"><Marca tamanho={20} /> Kontro</span>
          <button
            className="botao fantasma"
            onClick={() => invoke("abrir_aba", { aba: "config" })}
          >
            Ajustes
          </button>
          <button
            className="botao fantasma fechar"
            onClick={fechar}
            aria-label="Fechar painel"
            title="Fechar painel"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
              <path d="m4 4 8 8M12 4l-8 8" fill="none" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          </button>
        </header>
        <div
          className={estado.leituraAntiga ? "topo antiga" : "topo"}
          style={{ "--cor-do-estado": corDoAnel(estado, limiares) } as React.CSSProperties}
        >
          <Anel
            valor={estado.preenchimento}
            cor={corDoAnel(estado, limiares)}
            espessura={30}
            tamanho={96}
            girando={estado.girando}
            marcas={estado.temNumero ? limiares : null}
            rotulo={rotuloDoAnel(estado, limiares)}
          >
            {estado.temNumero ? (
              <span className="numero">{estado.percentual}%</span>
            ) : (
              <Glifo tamanho={38} cor="var(--text-secondary)" />
            )}
          </Anel>

          <Leitura estado={estado} />
        </div>

        <Historico serie={serie} compacto autonomiaMinutos={estado.autonomiaMinutos} />

        <ListaDeControles principal={estado.chave} />

        {cfg && (
          <div className="rapidos">
            <span id="rotulo-da-pilula">Visibilidade da pílula</span>
            <div className="modos" role="group" aria-labelledby="rotulo-da-pilula">
              {ORDEM_DOS_MODOS.map((modo) => (
                <button
                  className="botao"
                  key={modo}
                  aria-pressed={cfg.OverlayMode === modo}
                  disabled={salvando}
                  onClick={async () => {
                    if (modo === cfg.OverlayMode) return;
                    setSalvando(true);
                    setErroAoSalvar(false);
                    try {
                      await salvar(cfg, { OverlayMode: modo });
                    } catch {
                      setErroAoSalvar(true);
                    } finally {
                      setSalvando(false);
                    }
                  }}
                >
                  {modo === "Sempre" ? "Sempre" : MODOS[modo]}
                </button>
              ))}
            </div>
            {erroAoSalvar && <span className="erro" role="status">Não foi possível salvar o modo da pílula.</span>}
          </div>
        )}

        <footer className="acoes">
          <BotaoLerAgora estado={estado} className="botao" />
          <button className="botao primario" onClick={() => invoke("abrir_aba", { aba: "resumo" })}>
            Abrir resumo
          </button>
        </footer>
      </div>
    </div>
  );
}
