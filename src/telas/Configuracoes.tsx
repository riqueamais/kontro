import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

type CloseAction = "MinimizeToTray" | "Exit";
type OverlayMode = "Desligada" | "EmJogo" | "Sempre";
type OverlayCorner =
  | "SuperiorEsquerdo"
  | "SuperiorDireito"
  | "InferiorEsquerdo"
  | "InferiorDireito";

interface Config {
  StartWithWindows: boolean;
  StartMinimized: boolean;
  CloseAction: CloseAction;
  NotificationsEnabled: boolean;
  WarnThreshold: number;
  CriticalThreshold: number;
  ConnectToastEnabled: boolean;
  OverlayMode: OverlayMode;
  OverlayTipShown: boolean;
  OverlayCorner: OverlayCorner;
  OverlayMonitor: number;
  OverlayScale: number;
  OverlayOpacity: number;
  AutoCheckUpdates: boolean;
  FirstRunDone: boolean;
}

const CANTOS: Record<OverlayCorner, string> = {
  SuperiorEsquerdo: "Superior esquerdo",
  SuperiorDireito: "Superior direito",
  InferiorEsquerdo: "Inferior esquerdo",
  InferiorDireito: "Inferior direito",
};

const MODOS: Record<OverlayMode, string> = {
  Desligada: "Desligada",
  EmJogo: "Só em jogo",
  Sempre: "Sempre visível",
};

interface VersaoNova {
  versao: string;
  pagina: string;
  atual: string;
}

/** Em que ponto da atualizacao estamos. */
type Passo =
  | { tipo: "parado" }
  | { tipo: "procurando" }
  | { tipo: "atualizado" }
  | { tipo: "baixando"; porcento: number | null }
  | { tipo: "instalando" }
  | { tipo: "falhou"; motivo: string };

export function Configuracoes() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [nova, setNova] = useState<VersaoNova | null>(null);
  const [passo, setPasso] = useState<Passo>({ tipo: "parado" });

  useEffect(() => {
    invoke<Config>("configuracoes").then(setCfg).catch(() => {});
    invoke<VersaoNova | null>("versao_disponivel").then(setNova).catch(() => {});
  }, []);

  const procurar = async () => {
    setPasso({ tipo: "procurando" });
    try {
      const achado = await invoke<VersaoNova | null>("procurar_atualizacao");
      setNova(achado);
      // Nao achar nada e um resultado, e precisa ser dito. Voltar ao estado inicial em
      // silencio faz parecer que o botao nao fez nada.
      setPasso(achado ? { tipo: "parado" } : { tipo: "atualizado" });
    } catch {
      setPasso({ tipo: "falhou", motivo: "não deu para consultar o repositório" });
    }
  };

  /**
   * Baixa, instala e reinicia.
   *
   * O pacote e verificado contra a chave publica que vive dentro do app: um instalador
   * que nao tenha sido assinado com a chave correspondente e recusado antes de rodar.
   * Sem isso, atualizar sozinho seria executar o que quer que estivesse naquela URL.
   */
  const atualizarAgora = async () => {
    setPasso({ tipo: "baixando", porcento: null });
    try {
      const atualizacao = await check();
      if (!atualizacao) {
        setNova(null);
        setPasso({ tipo: "parado" });
        return;
      }

      let total = 0;
      let baixado = 0;
      await atualizacao.downloadAndInstall((evento) => {
        if (evento.event === "Started") {
          total = evento.data.contentLength ?? 0;
        } else if (evento.event === "Progress") {
          baixado += evento.data.chunkLength;
          setPasso({
            tipo: "baixando",
            porcento: total > 0 ? Math.round((baixado / total) * 100) : null,
          });
        } else if (evento.event === "Finished") {
          setPasso({ tipo: "instalando" });
        }
      });

      await relaunch();
    } catch (e) {
      setPasso({ tipo: "falhou", motivo: String(e) });
    }
  };

  if (!cfg) return null;

  const gravar = (mudanca: Partial<Config>) => {
    const novas = { ...cfg, ...mudanca };
    setCfg(novas);
    void invoke("salvar_configuracoes", { novas });
  };

  const ciclar = <T extends string>(atual: T, opcoes: readonly T[]): T =>
    opcoes[(opcoes.indexOf(atual) + 1) % opcoes.length];

  return (
    <>
      <h1 className="titulo-da-pagina">Configurações</h1>

      <h2>Inicialização</h2>
      <Linha
        titulo="Iniciar com o Windows"
        descricao="Sobe junto com o sistema e já começa a monitorar."
      >
        <Chave
          ligado={cfg.StartWithWindows}
          aoTrocar={(v) => gravar({ StartWithWindows: v })}
        />
      </Linha>
      <Linha
        titulo="Iniciar minimizado"
        descricao="Abre direto na bandeja, sem mostrar esta janela."
      >
        <Chave ligado={cfg.StartMinimized} aoTrocar={(v) => gravar({ StartMinimized: v })} />
      </Linha>
      <Linha titulo="Ao clicar no X" descricao="Fechar a janela pode só esconder o app.">
        <button
          className="ciclo"
          onClick={() =>
            gravar({ CloseAction: ciclar(cfg.CloseAction, ["MinimizeToTray", "Exit"] as const) })
          }
        >
          {cfg.CloseAction === "MinimizeToTray" ? "Minimizar" : "Encerrar"}
        </button>
      </Linha>

      <h2>Avisos</h2>
      <Linha titulo="Avisar carga baixa" descricao="Notificação ao cruzar os limiares abaixo.">
        <Chave
          ligado={cfg.NotificationsEnabled}
          aoTrocar={(v) => gravar({ NotificationsEnabled: v })}
        />
      </Linha>
      <Linha
        titulo="Avisar ao conectar"
        descricao="Uma caixa no topo quando o controle entra ou sai."
      >
        <Chave
          ligado={cfg.ConnectToastEnabled}
          aoTrocar={(v) => gravar({ ConnectToastEnabled: v })}
        />
      </Linha>

      <h2>Sobreposição</h2>
      <Linha titulo="Quando aparecer" descricao="Fixa na tela por cima do que estiver aberto.">
        <button
          className="ciclo"
          onClick={() =>
            gravar({
              OverlayMode: ciclar(cfg.OverlayMode, ["Desligada", "EmJogo", "Sempre"] as const),
            })
          }
        >
          {MODOS[cfg.OverlayMode]}
        </button>
      </Linha>
      <Linha titulo="Canto" descricao="Onde a pílula fica ancorada.">
        <button
          className="ciclo"
          onClick={() =>
            gravar({
              OverlayCorner: ciclar(cfg.OverlayCorner, [
                "InferiorDireito",
                "InferiorEsquerdo",
                "SuperiorDireito",
                "SuperiorEsquerdo",
              ] as const),
            })
          }
        >
          {CANTOS[cfg.OverlayCorner]}
        </button>
      </Linha>
      <Linha
        titulo="Monitor"
        descricao="Fixa a pílula numa tela em vez de deixar que ela siga o foco."
      >
        <button
          className="ciclo"
          onClick={() =>
            gravar({ OverlayMonitor: cfg.OverlayMonitor >= 1 ? -1 : cfg.OverlayMonitor + 1 })
          }
        >
          {cfg.OverlayMonitor < 0 ? "Segue o jogo" : `Monitor ${cfg.OverlayMonitor + 1}`}
        </button>
      </Linha>

      <h2>Versão</h2>
      <Linha titulo={tituloDaVersao(nova, passo)} descricao={detalheDaVersao(nova, passo)}>
        {nova && passo.tipo === "parado" ? (
          <button className="ciclo destaque" onClick={() => void atualizarAgora()}>
            Atualizar agora
          </button>
        ) : (
          <button
            className="ciclo"
            disabled={
              passo.tipo === "procurando" ||
              passo.tipo === "baixando" ||
              passo.tipo === "instalando"
            }
            onClick={() => void procurar()}
          >
            {passo.tipo === "procurando" ? "Procurando..." : "Procurar"}
          </button>
        )}
      </Linha>

      <Linha
        titulo="Avisar sobre versões novas"
        descricao="Consulta o repositório de tempos em tempos, sem baixar nada sozinho."
      >
        <Chave ligado={cfg.AutoCheckUpdates} aoTrocar={(v) => gravar({ AutoCheckUpdates: v })} />
      </Linha>
    </>
  );
}

function tituloDaVersao(nova: VersaoNova | null, passo: Passo): string {
  switch (passo.tipo) {
    case "atualizado":
      return "Você está na versão mais recente";
    case "baixando":
      return passo.porcento === null
        ? "Baixando a atualização..."
        : `Baixando a atualização... ${passo.porcento}%`;
    case "instalando":
      return "Instalando — o app vai reiniciar";
    case "falhou":
      return "Não foi possível atualizar";
    default:
      return nova ? `Versão ${nova.versao} disponível` : "Procurar atualizações";
  }
}

function detalheDaVersao(nova: VersaoNova | null, passo: Passo): string {
  if (passo.tipo === "falhou") return passo.motivo;
  if (passo.tipo === "procurando") return "Consultando o repositório...";
  if (passo.tipo === "atualizado") return "Nada novo publicado desde esta versão.";
  if (passo.tipo === "baixando" || passo.tipo === "instalando") {
    return "O pacote é verificado antes de rodar.";
  }
  return nova
    ? `Você está na ${nova.atual}. O app baixa e instala sozinho.`
    : "O app avisa sozinho a cada três horas quando sai versão nova.";
}

function Linha({
  titulo,
  descricao,
  children,
}: {
  titulo: string;
  descricao: string;
  children: React.ReactNode;
}) {
  return (
    <div className="linha">
      <div className="rotulo">
        <div className="titulo">{titulo}</div>
        <div className="descricao">{descricao}</div>
      </div>
      <div className="controle">{children}</div>
    </div>
  );
}

function Chave({ ligado, aoTrocar }: { ligado: boolean; aoTrocar: (v: boolean) => void }) {
  return (
    <button
      role="switch"
      aria-checked={ligado}
      className={`chave${ligado ? " ligada" : ""}`}
      onClick={() => aoTrocar(!ligado)}
    >
      <span className="bolinha" />
    </button>
  );
}
