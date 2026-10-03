import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { LIMIARES_CRITICOS, LIMIARES_DE_AVISO, salvar } from "../ajustes";
import { Anel } from "../componentes/Anel";
import { BotaoDeCiclo } from "../componentes/BotaoDeCiclo";
import { Chave, Linha, MiniTela } from "../componentes/Controles";
import { Glifo } from "../componentes/Glifo";
import { Marca } from "../componentes/Marca";
import { Teclas } from "../componentes/Teclas";
import {
  Config,
  Estado,
  Recusa,
  corDoAnel,
  useAtalhosRecusados,
  useConfig,
  useEstado,
  usePilulaSolta,
} from "../estado";
import { quandoLeu } from "../formato";
import "./passos.css";

const TOTAL = 6;
const PASSO_DA_PILULA = 3;

const CORES: [number, string, string][] = [
  [61, "var(--accent-green)", "com folga"],
  [18, "var(--amber)", "carga baixa"],
  [7, "var(--red)", "crítica"],
];

export function Passos({ aoTerminar }: { aoTerminar: () => void }) {
  const cfg = useConfig();
  const solta = usePilulaSolta();
  const recusados = useAtalhosRecusados();
  const estado = useEstado();
  const [passo, setPasso] = useState(0);
  const [pouso, setPouso] = useState<Pouso | null>(null);

  useEffect(() => {
    const parar = listen<Pouso>("kontro://pouso", ({ payload }) => setPouso(payload));
    return () => {
      void parar.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (!solta) setPouso(null);
  }, [solta]);
  const avancar = useRef<HTMLButtonElement>(null);
  const pular = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    avancar.current?.focus();
  }, [passo, cfg !== null]);

  useEffect(() => {
    if (passo !== PASSO_DA_PILULA) return;

    void invoke("soltar_a_pilula", { solta: true });
    return () => {
      void invoke("soltar_a_pilula", { solta: false });
    };
  }, [passo]);

  useEffect(() => {
    const aoTeclar = (evento: KeyboardEvent) => {
      if (evento.key === "ArrowRight") setPasso((p) => Math.min(p + 1, TOTAL - 1));
      if (evento.key === "ArrowLeft") setPasso((p) => Math.max(p - 1, 0));
      if (evento.key === "Enter" && evento.target === document.body) avancar.current?.click();
      if (evento.key === "Escape") {
        evento.preventDefault();
        (pular.current ?? avancar.current)?.click();
      }
    };

    window.addEventListener("keydown", aoTeclar);
    return () => window.removeEventListener("keydown", aoTeclar);
  }, []);

  if (!cfg) return null;

  const terminar = () => {
    if (!cfg.FirstRunDone) salvar(cfg, { FirstRunDone: true });
    aoTerminar();
  };

  const ultimo = passo === TOTAL - 1;

  return (
    <div className="passos">
      <div className="cabeca-dos-passos">
        <div className="pontos" aria-hidden="true">
          {Array.from({ length: TOTAL }, (_, i) => (
            <span key={i} className={`ponto${i === passo ? " agora" : i < passo ? " feito" : ""}`} />
          ))}
        </div>
        <span className="contador" aria-live="polite">
          Passo {passo + 1} de {TOTAL}
        </span>
      </div>

      <div className="folha" key={passo}>
        {folha(passo, { cfg, solta, recusados, estado, pouso })}
      </div>

      <div className="rodape">
        <button
          ref={ultimo ? undefined : pular}
          className={`botao fantasma${ultimo ? " oculto" : ""}`}
          tabIndex={ultimo ? -1 : undefined}
          aria-hidden={ultimo || undefined}
          onClick={() => setPasso(TOTAL - 1)}
        >
          Pular
        </button>
        <div className="adiante">
          <button className="botao" disabled={passo === 0} onClick={() => setPasso(passo - 1)}>
            Voltar
          </button>
          <button
            ref={avancar}
            className="botao destaque"
            onClick={() => (ultimo ? terminar() : setPasso(passo + 1))}
          >
            {ultimo ? "Começar" : "Avançar"}
          </button>
        </div>
      </div>
    </div>
  );
}

interface Pouso {
  x: number;
  y: number;
}

interface Contexto {
  cfg: Config;
  solta: boolean;
  recusados: Recusa[];
  estado: Estado | null;
  pouso: Pouso | null;
}

function folha(passo: number, { cfg, solta, recusados, estado, pouso }: Contexto) {
  const limiares = { critico: cfg.CriticalThreshold, aviso: cfg.WarnThreshold };
  const aoVivo =
    !!estado &&
    estado.via !== "Desligado" &&
    !estado.leituraAntiga &&
    estado.preenchimento !== null;
  const anel = {
    valor: aoVivo ? estado.preenchimento : 72,
    cor: aoVivo ? corDoAnel(estado, limiares) : "var(--accent-green)",
  };

  switch (passo) {
    case 0:
      return (
        <>
          <div className="palco coluna">
            <Anel valor={anel.valor} cor={anel.cor} espessura={54} tamanho={112} desde={0}>
              <Glifo tamanho={42} cor="var(--text-primary)" />
            </Anel>
            <span className="legenda">
              {aoVivo ? `${estado.nome} · ${estado.textoDaCarga}` : "exemplo"}
            </span>
          </div>
          <h1>A bateria do controle, na bandeja</h1>
          <p>
            O Kontro lê a carga direto do controle e mantém o número na bandeja do Windows. Seis
            telas e você sabe tudo o que ele faz. Ele fica na bandeja — o último passo mostra onde.
          </p>
        </>
      );

    case 1:
      return (
        <>
          <div className="palco fileira">
            {CORES.map(([valor, cor, rotulo]) => (
              <div className="amostra" key={rotulo}>
                <Anel valor={valor} cor={cor} espessura={70} tamanho={54}>
                  <Glifo tamanho={22} cor="var(--text-primary)" />
                </Anel>
                <span className="legenda">{rotulo}</span>
              </div>
            ))}
            <div className="amostra apagada">
              <span className="riscado">
                <Glifo tamanho={22} cor="var(--gray)" />
                <svg className="risco" viewBox="0 0 54 54" aria-hidden="true">
                  <path
                    d="M12.7 41.3 L41.3 12.7"
                    stroke="var(--gray)"
                    strokeWidth="4.9"
                    strokeLinecap="round"
                  />
                </svg>
              </span>
              <span className="legenda">desligado</span>
            </div>
          </div>
          <h1>O ícone é o dado</h1>
          <p>
            O anel mostra quanto sobrou e troca de cor nos limiares que você escolher. Sem controle
            ligado, ele vira um controle riscado — nada de número velho fingindo ser de agora.
          </p>
        </>
      );

    case 2:
      return (
        <>
          <div className="palco">
            <div className="previa">
              <Anel valor={null} cor="var(--gray)" espessura={60} tamanho={34} girando>
                <Glifo tamanho={17} cor="var(--text-primary)" />
              </Anel>
              <div className="dizeres">
                <span className="valor">No cabo</span>
                <span className="carimbo">{carimbo(estado)}</span>
              </div>
            </div>
          </div>
          <h1>No cabo não existe porcentagem</h1>
          <p>
            Plugado, o controle troca de protocolo e para de publicar a carga exata — o que sobra
            erra feio. Em vez de inventar um número, o Kontro mostra a última leitura com a hora e a
            data em que ela foi medida.
          </p>
        </>
      );

    case PASSO_DA_PILULA:
      return (
        <>
          <div className="palco">
            <MiniTela
              x={pouso?.x ?? cfg.OverlayX}
              y={pouso?.y ?? cfg.OverlayY}
              solta={solta}
              escala={3}
            />
          </div>
          <h1>A pílula em jogo</h1>
          {solta ? (
            <p>
              Ela está solta na sua tela agora: arraste para onde quiser. Perto de um canto ou do
              meio ela encaixa sozinha, e o cadeado ao lado dela prende no lugar.
            </p>
          ) : (
            <p>
              É onde a carga aparece por cima do jogo. Está presa no ponto do desenho acima.{" "}
              <ComoSoltar cfg={cfg} recusados={recusados} />
            </p>
          )}
          {solta && estado?.via === "Desligado" && (
            <p>
              Sem controle ligado ela fica cinza, sem número; ligue um e a carga aparece nela na
              hora.
            </p>
          )}
          {!solta && (
            <button
              className="botao"
              onClick={() => void invoke("soltar_a_pilula", { solta: true })}
            >
              Soltar de novo
            </button>
          )}
        </>
      );

    case 4:
      return (
        <>
          <div className="palco">
            <Anel
              valor={anel.valor}
              cor={anel.cor}
              espessura={54}
              tamanho={120}
              marcas={limiares}
              desde={0}
            >
              <Glifo tamanho={46} cor="var(--text-primary)" />
            </Anel>
          </div>
          <h1>Quando ele te avisa</h1>
          <p>
            Os mesmos números mandam na cor do anel: âmbar quando o app diz carga baixa, vermelho
            quando diz crítica.
          </p>
          <Linha titulo="Carga baixa" descricao="A primeira vez que ele te chama.">
            <BotaoDeCiclo
              opcoes={LIMIARES_DE_AVISO}
              valor={cfg.WarnThreshold}
              rotulo={(v) => `${v}%`}
              aoMudar={(aviso) =>
                void salvar(cfg, {
                  WarnThreshold: aviso,
                  CriticalThreshold: Math.min(cfg.CriticalThreshold, aviso - 1),
                })
              }
            />
          </Linha>
          <Linha titulo="Carga crítica" descricao="O segundo aviso, mais urgente.">
            <BotaoDeCiclo
              opcoes={LIMIARES_CRITICOS.filter((c) => c < cfg.WarnThreshold)}
              valor={cfg.CriticalThreshold}
              rotulo={(v) => `${v}%`}
              aoMudar={(CriticalThreshold) => void salvar(cfg, { CriticalThreshold })}
            />
          </Linha>
        </>
      );

    default:
      return (
        <>
          <div className="palco">
            <div className="mini-barra" aria-hidden="true">
              <kbd className="tecla">^</kbd>
              <span className="mini-barra-marca">
                <Marca tamanho={16} />
              </span>
            </div>
          </div>
          <h1>Pronto</h1>
          <p>
            O ícone fica na bandeja: um clique abre o painel rápido com a carga, e o botão direito
            abre o menu. Configurações traz esta janela de volta, já na aba certa, onde mora tudo o
            que você viu aqui e o resto.
          </p>
          <div className="recado">
            O Windows 11 esconde ícones novos atrás da setinha <b>^</b> da bandeja. Arraste o Kontro
            para fora dela uma vez e ele fica fixo.
          </div>
          <Linha
            titulo="Iniciar com o Windows"
            descricao="Sobe junto com o sistema e já começa a monitorar."
          >
            <Chave
              ligado={cfg.StartWithWindows}
              aoTrocar={(v) => salvar(cfg, { StartWithWindows: v })}
            />
          </Linha>
        </>
      );
  }
}

function ComoSoltar({ cfg, recusados }: { cfg: Config; recusados: Recusa[] }) {
  if (!cfg.OverlayShortcutEnabled) {
    return (
      <>
        Os atalhos estão desligados; dá para ligar em Configurações &gt; Atalhos, e o botão Soltar
        de lá solta ela de novo.
      </>
    );
  }
  if (recusados.some((r) => r.atalho === "Mover")) {
    return <>O Windows recusou esse atalho; escolha outro em Configurações &gt; Atalhos.</>;
  }
  return (
    <>
      <span className="teclas">
        <Teclas combinacao={cfg.OverlayMoveShortcut} />
      </span>{" "}
      solta ela de novo, e{" "}
      <span className="teclas">
        <Teclas combinacao={cfg.OverlayShortcut} />
      </span>{" "}
      esconde e traz de volta, sem sair do jogo.
    </>
  );
}

function carimbo(estado: Estado | null): string {
  if (!estado) return "sem leitura ainda";
  const { texto, hora } = quandoLeu(estado);
  return hora ? `${texto} ${hora}` : texto || "sem leitura ainda";
}
