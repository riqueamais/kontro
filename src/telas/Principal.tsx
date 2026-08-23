import { invoke } from "@tauri-apps/api/core";
import { open as abrirNoNavegador } from "@tauri-apps/plugin-shell";
import { useEffect, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { corDoAnel, useEstado } from "../estado";
import "./principal.css";

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

export function Principal() {
  const estado = useEstado();
  const [cfg, setCfg] = useState<Config | null>(null);
  const [nova, setNova] = useState<VersaoNova | null>(null);
  const [procurando, setProcurando] = useState(false);

  useEffect(() => {
    invoke<Config>("configuracoes").then(setCfg).catch(() => {});
    invoke<VersaoNova | null>("versao_disponivel").then(setNova).catch(() => {});
  }, []);

  const procurar = async () => {
    setProcurando(true);
    try {
      setNova(await invoke<VersaoNova | null>("procurar_atualizacao"));
    } catch {
      // sem rede nao ha o que dizer alem de "nao deu"
    } finally {
      setProcurando(false);
    }
  };

  if (!cfg || !estado) return null;

  const gravar = (mudanca: Partial<Config>) => {
    const novas = { ...cfg, ...mudanca };
    setCfg(novas);
    void invoke("salvar_configuracoes", { novas });
  };

  const ciclar = <T extends string>(atual: T, opcoes: readonly T[]): T =>
    opcoes[(opcoes.indexOf(atual) + 1) % opcoes.length];

  return (
    <div className="principal">
      <header>
        <Anel valor={100} cor="var(--accent-teal)" espessura={44} tamanho={30}>
          <Glifo tamanho={13} cor="var(--text-primary)" />
        </Anel>
        <h1>Kontro</h1>
      </header>

      <section className="cartao estado">
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado)}
          espessura={38}
          tamanho={72}
          girando={estado.girando}
        >
          {estado.temNumero ? (
            <span className="numero">{estado.percent}%</span>
          ) : (
            <Glifo tamanho={28} cor="var(--text-secondary)" />
          )}
        </Anel>
        <div>
          <div className="dispositivo">{estado.deviceName}</div>
          <div className="detalhe">
            {estado.mode === "Offline"
              ? "Desconectado"
              : estado.conectadoSemCarga
                ? `Conectado ${estado.textoDaLigacao} · este controle não informa a bateria`
                : estado.precisao === "Aproximada"
                  ? `${estado.textoDaCarga} · este controle não informa percentual`
                  : `Conectado - ${estado.textoDaLigacao}`}
          </div>
        </div>
      </section>

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
      <Linha titulo="Iniciar minimizado" descricao="Abre direto na bandeja, sem mostrar esta janela.">
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
      <Linha titulo="Avisar ao conectar" descricao="Uma caixa no topo quando o controle entra ou sai.">
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
      <Linha
        titulo={nova ? `Versão ${nova.versao} disponível` : "Procurar atualizações"}
        descricao={
          nova
            ? `Você está na ${nova.atual}. O download é pela página da release.`
            : procurando
              ? "Consultando o repositório..."
              : "O app avisa sozinho a cada três horas quando sai versão nova."
        }
      >
        {nova ? (
          <button className="ciclo destaque" onClick={() => void abrirNoNavegador(nova.pagina)}>
            Abrir a página
          </button>
        ) : (
          <button className="ciclo" disabled={procurando} onClick={() => void procurar()}>
            {procurando ? "Procurando..." : "Procurar"}
          </button>
        )}
      </Linha>

      <Linha
        titulo="Avisar sobre versões novas"
        descricao="Consulta o repositório de tempos em tempos, sem baixar nada sozinho."
      >
        <Chave
          ligado={cfg.AutoCheckUpdates}
          aoTrocar={(v) => gravar({ AutoCheckUpdates: v })}
        />
      </Linha>
    </div>
  );
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
