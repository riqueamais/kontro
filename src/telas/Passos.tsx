import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { LIMIARES_CRITICOS, LIMIARES_DE_AVISO, ciclar, salvar } from "../ajustes";
import { Anel } from "../componentes/Anel";
import { Chave, Linha, MiniTela } from "../componentes/Controles";
import { Glifo } from "../componentes/Glifo";
import { Config, useConfig, usePilulaSolta } from "../estado";
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
  const [passo, setPasso] = useState(0);

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
      <div className="pontos" aria-hidden="true">
        {Array.from({ length: TOTAL }, (_, i) => (
          <span key={i} className={`ponto${i === passo ? " agora" : i < passo ? " feito" : ""}`} />
        ))}
      </div>

      <div className="folha">{folha(passo, cfg, solta)}</div>

      <div className="rodape">
        {ultimo ? (
          <span />
        ) : (
          <button className="ciclo fantasma" onClick={terminar}>
            Pular
          </button>
        )}
        <div className="adiante">
          {passo > 0 && (
            <button className="ciclo" onClick={() => setPasso(passo - 1)}>
              Voltar
            </button>
          )}
          <button
            className="ciclo destaque"
            onClick={() => (ultimo ? terminar() : setPasso(passo + 1))}
          >
            {ultimo ? "Começar" : "Avançar"}
          </button>
        </div>
      </div>
    </div>
  );
}

function folha(passo: number, cfg: Config, solta: boolean) {
  switch (passo) {
    case 0:
      return (
        <>
          <div className="palco">
            <Anel valor={72} cor="var(--accent-green)" espessura={54} tamanho={120}>
              <Glifo tamanho={46} cor="var(--text-primary)" />
            </Anel>
          </div>
          <h1>A bateria do controle, na bandeja</h1>
          <p>
            O Kontro lê a carga direto do controle e mantém o número na bandeja do Windows. Seis
            telas e você sabe tudo o que ele faz.
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
              <div className="riscado">
                <Anel valor={null} cor="var(--gray)" espessura={70} tamanho={54}>
                  <Glifo tamanho={22} cor="var(--gray)" />
                </Anel>
                <svg className="risco" viewBox="0 0 54 54" aria-hidden="true">
                  <path
                    d="M14 40 L40 14"
                    stroke="var(--gray)"
                    strokeWidth="3.4"
                    strokeLinecap="round"
                  />
                </svg>
              </div>
              <span className="legenda">desligado</span>
            </div>
          </div>
          <h1>O ícone é o dado</h1>
          <p>
            O anel mostra quanto sobrou e troca de cor nos limiares que você escolher. Sem controle
            ligado, ele vira um controle riscado — nada de número velho fingindo ser de agora.
          </p>
          <div className="recado">
            O Windows 11 esconde ícones novos atrás da setinha <b>^</b> da bandeja. Arraste o Kontro
            para fora dela uma vez e ele fica fixo.
          </div>
        </>
      );

    case 2:
      return (
        <>
          <div className="palco">
            <div className="previa">
              <Anel valor={null} cor="var(--accent-teal)" espessura={60} tamanho={34} girando>
                <Glifo tamanho={17} cor="var(--text-primary)" />
              </Anel>
              <div className="dizeres">
                <span className="valor">no cabo</span>
                <span className="carimbo">lido ontem às 23:53</span>
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
            <MiniTela x={cfg.OverlayX} y={cfg.OverlayY} solta={solta} escala={3} />
          </div>
          <h1>A pílula em jogo</h1>
          {solta ? (
            <p>
              Ela está solta na sua tela agora: arraste para onde quiser. Perto de um canto ou do
              meio ela encaixa sozinha, e o cadeado ao lado dela prende no lugar.
            </p>
          ) : (
            <p>
              É onde a carga aparece por cima do jogo. Está presa no ponto do desenho acima —
              <b> Ctrl + Shift + M</b> solta ela de novo a qualquer momento, sem sair do jogo.
            </p>
          )}
          {!solta && (
            <button
              className="ciclo"
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
          <div className="palco fileira">
            <div className="amostra">
              <Anel valor={cfg.WarnThreshold} cor="var(--amber)" espessura={70} tamanho={54}>
                <Glifo tamanho={22} cor="var(--text-primary)" />
              </Anel>
              <span className="legenda">avisa</span>
            </div>
            <div className="amostra">
              <Anel valor={cfg.CriticalThreshold} cor="var(--red)" espessura={70} tamanho={54}>
                <Glifo tamanho={22} cor="var(--text-primary)" />
              </Anel>
              <span className="legenda">insiste</span>
            </div>
          </div>
          <h1>Quando ele te avisa</h1>
          <p>
            Os mesmos números mandam na cor do anel: âmbar quando o app diz carga baixa, vermelho
            quando diz crítica.
          </p>
          <Linha titulo="Avisar em" descricao="A primeira vez que ele te chama.">
            <button
              className="ciclo"
              onClick={() => {
                const aviso = ciclar(cfg.WarnThreshold, LIMIARES_DE_AVISO);
                salvar(cfg, {
                  WarnThreshold: aviso,
                  CriticalThreshold: Math.min(cfg.CriticalThreshold, aviso - 5),
                });
              }}
            >
              {cfg.WarnThreshold}%
            </button>
          </Linha>
          <Linha titulo="Avisar de novo em" descricao="O segundo aviso, mais urgente.">
            <button
              className="ciclo"
              onClick={() =>
                salvar(cfg, {
                  CriticalThreshold: Math.min(
                    ciclar(cfg.CriticalThreshold, LIMIARES_CRITICOS),
                    cfg.WarnThreshold - 5,
                  ),
                })
              }
            >
              {cfg.CriticalThreshold}%
            </button>
          </Linha>
        </>
      );

    default:
      return (
        <>
          <div className="palco">
            <Anel valor={100} cor="var(--accent-green)" espessura={54} tamanho={120}>
              <Glifo tamanho={46} cor="var(--text-primary)" />
            </Anel>
          </div>
          <h1>Pronto</h1>
          <p>
            O ícone fica na bandeja: um clique nele abre o resumo, e o menu do botão direito leva às
            configurações, onde mora tudo o que você viu aqui — e o resto.
          </p>
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
