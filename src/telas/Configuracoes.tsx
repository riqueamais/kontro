import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

import { Config, OverlayMode, useAtalhosRecusados, usePilulaSolta } from "../estado";

const MODIFICADORES = ["Control", "Shift", "Alt", "Meta"];

const NOME_DA_TECLA: Record<string, string> = {
  Ctrl: "Ctrl",
  Alt: "Alt",
  Shift: "Shift",
  Super: "Win",
  Space: "Espaço",
  Escape: "Esc",
  Delete: "Del",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backslash: "\\",
  PageUp: "Page Up",
  PageDown: "Page Down",
};

const LIMIARES_DE_AVISO = [40, 30, 25, 20, 15, 10] as const;
const LIMIARES_CRITICOS = [20, 15, 10, 5] as const;

const TAMANHOS: [number, string][] = [
  [0.85, "Pequena"],
  [1, "Padrão"],
  [1.2, "Grande"],
  [1.45, "Enorme"],
];

const OPACIDADES: [number, string][] = [
  [1, "Sólida"],
  [0.9, "90%"],
  [0.75, "75%"],
  [0.55, "55%"],
];

const MODOS: Record<OverlayMode, string> = {
  Desligada: "Desligada",
  EmJogo: "Só em jogo",
  Sempre: "Sempre visível",
};

interface VersaoNova {
  versao: string;
  notas: string | null;
  atual: string;
}

interface Busca {
  estado: "nova" | "em-dia" | "falhou";
  versao: string | null;
  notas: string | null;
  atual: string;
  motivo: string | null;
}

type Passo =
  | { tipo: "parado" }
  | { tipo: "procurando" }
  | { tipo: "atualizado" }
  | { tipo: "baixando"; porcento: number | null }
  | { tipo: "instalando" }
  | { tipo: "falhou"; motivo: string; ao: "verificar" | "atualizar" };

export function Configuracoes() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [nova, setNova] = useState<VersaoNova | null>(null);
  const [passo, setPasso] = useState<Passo>({ tipo: "parado" });
  const [telas, setTelas] = useState(1);
  const [diagnostico, setDiagnostico] = useState<"parado" | "gravando" | "pronto" | "falhou">(
    "parado",
  );
  const solta = usePilulaSolta();
  const recusados = useAtalhosRecusados();

  useEffect(() => {
    invoke<Config>("configuracoes").then(setCfg).catch(() => {});
    invoke<VersaoNova | null>("versao_disponivel").then(setNova).catch(() => {});
    invoke<number>("quantidade_de_telas").then(setTelas).catch(() => {});
  }, []);

  const procurar = async () => {
    setPasso({ tipo: "procurando" });
    try {
      const busca = await invoke<Busca>("procurar_atualizacao");

      if (busca.estado === "falhou") {
        setPasso({
          tipo: "falhou",
          ao: "verificar",
          motivo: busca.motivo ?? "não deu para consultar o repositório",
        });
        return;
      }

      if (busca.estado === "nova" && busca.versao) {
        setNova({ versao: busca.versao, notas: busca.notas, atual: busca.atual });
        setPasso({ tipo: "parado" });
        return;
      }

      setNova(null);
      setPasso({ tipo: "atualizado" });
    } catch {
      setPasso({
        tipo: "falhou",
        ao: "verificar",
        motivo: "não deu para consultar o repositório",
      });
    }
  };

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
      setPasso({ tipo: "falhou", ao: "atualizar", motivo: String(e) });
    }
  };

  if (!cfg) return null;
  const gravar = (mudanca: Partial<Config>) => {
    const novas = { ...cfg, ...mudanca };
    setCfg(novas);
    void invoke("salvar_configuracoes", { novas });
  };

  const ciclar = <T extends string | number>(atual: T, opcoes: readonly T[]): T => {
    const i = opcoes.indexOf(atual);
    return i < 0 ? opcoes[0] : opcoes[(i + 1) % opcoes.length];
  };
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
        titulo="Avisar em"
        descricao="Carga a partir da qual o Kontro avisa que ela está baixa."
      >
        <button
          className="ciclo"
          disabled={!cfg.NotificationsEnabled}
          onClick={() => {
            const aviso = ciclar(cfg.WarnThreshold, LIMIARES_DE_AVISO);
            gravar({
              WarnThreshold: aviso,
              CriticalThreshold: Math.min(cfg.CriticalThreshold, aviso - 5),
            });
          }}
        >
          {cfg.WarnThreshold}%
        </button>
      </Linha>
      <Linha
        titulo="Avisar de novo em"
        descricao="O segundo aviso, mais urgente. É ele que também traz a pílula para a tela fora de jogo."
      >
        <button
          className="ciclo"
          disabled={!cfg.NotificationsEnabled}
          onClick={() =>
            gravar({
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
      <Linha
        titulo="Posição"
        descricao={
          solta
            ? "Arraste a pílula pela tela. Perto de um canto ou do meio ela encaixa sozinha."
            : "A pílula fica onde você largar: solte e arraste até o ponto que quiser."
        }
      >
        <MiniTela x={cfg.OverlayX} y={cfg.OverlayY} solta={solta} />
        <button
          className={solta ? "ciclo destaque" : "ciclo"}
          onClick={() => void invoke("soltar_a_pilula", { solta: !solta })}
        >
          {solta ? "Prender" : "Soltar"}
        </button>
      </Linha>
      <Linha
        titulo="Monitor"
        descricao="Fixa a pílula numa tela em vez de deixar que ela siga a janela em foco."
      >
        <button
          className="ciclo"
          onClick={() =>
            gravar({
              OverlayMonitor: cfg.OverlayMonitor + 1 >= telas ? -1 : cfg.OverlayMonitor + 1,
            })
          }
        >
          {cfg.OverlayMonitor < 0 ? "Segue o jogo" : `Monitor ${cfg.OverlayMonitor + 1}`}
        </button>
      </Linha>
      <Linha titulo="Tamanho" descricao="Quanto espaço a pílula ocupa na tela.">
        <button
          className="ciclo"
          onClick={() => gravar({ OverlayScale: ciclar(cfg.OverlayScale, tamanhos()) })}
        >
          {rotulo(TAMANHOS, cfg.OverlayScale)}
        </button>
      </Linha>
      <Linha titulo="Transparência" descricao="Para a pílula não competir com o HUD do jogo.">
        <button
          className="ciclo"
          onClick={() => gravar({ OverlayOpacity: ciclar(cfg.OverlayOpacity, opacidades()) })}
        >
          {rotulo(OPACIDADES, cfg.OverlayOpacity)}
        </button>
      </Linha>
      <h2>Atalhos</h2>
      <Linha
        titulo="Usar atalhos"
        descricao="Valem por cima do jogo, sem precisar sair dele."
      >
        <Chave
          ligado={cfg.OverlayShortcutEnabled}
          aoTrocar={(v) => gravar({ OverlayShortcutEnabled: v })}
        />
      </Linha>
      <Linha
        titulo="Mostrar e esconder a pílula"
        descricao={descricaoDoAtalho(
          "Tira a pílula da frente e traz de volta.",
          cfg.OverlayShortcut,
          recusados,
        )}
      >
        <Captura
          combinacao={cfg.OverlayShortcut}
          desabilitado={!cfg.OverlayShortcutEnabled}
          aoTrocar={(c) => gravar({ OverlayShortcut: c })}
        />
      </Linha>
      <Linha
        titulo="Soltar a pílula para mover"
        descricao={descricaoDoAtalho(
          "Solta a pílula para arrastar, e prende de novo onde você largar.",
          cfg.OverlayMoveShortcut,
          recusados,
        )}
      >
        <Captura
          combinacao={cfg.OverlayMoveShortcut}
          desabilitado={!cfg.OverlayShortcutEnabled}
          aoTrocar={(c) => gravar({ OverlayMoveShortcut: c })}
        />
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
      <h2>Problemas</h2>
      <Linha titulo="Salvar diagnóstico" descricao={textoDoDiagnostico(diagnostico)}>
        <button
          className="ciclo"
          disabled={diagnostico === "gravando"}
          onClick={async () => {
            setDiagnostico("gravando");
            try {
              await invoke<string>("salvar_diagnostico");
              setDiagnostico("pronto");
            } catch {
              setDiagnostico("falhou");
            }
          }}
        >
          {diagnostico === "gravando" ? "Gravando..." : "Salvar"}
        </button>
      </Linha>
    </>
  );
}
const tamanhos = () => TAMANHOS.map(([v]) => v);
const opacidades = () => OPACIDADES.map(([v]) => v);
function rotulo(degraus: [number, string][], valor: number): string {
  return degraus.find(([v]) => v === valor)?.[1] ?? `${Math.round(valor * 100)}%`;
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
      return passo.ao === "verificar"
        ? "Não foi possível verificar"
        : "Não foi possível atualizar";
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
  if (!nova) return "O app verifica sozinho uma vez por dia, e você pode procurar quando quiser.";

  const resumo = primeiraLinha(nova.notas);
  return resumo
    ? `${resumo} Você está na ${nova.atual}; o app baixa e instala sozinho.`
    : `Você está na ${nova.atual}. O app baixa e instala sozinho.`;
}
function textoDoDiagnostico(passo: "parado" | "gravando" | "pronto" | "falhou"): string {
  switch (passo) {
    case "gravando":
      return "Perguntando a cada fonte o que ela sabe da carga...";
    case "pronto":
      return "Salvo como diagnostico.txt, e a pasta abriu. É o arquivo para anexar ao relatar um problema.";
    case "falhou":
      return "Não deu para gravar o arquivo.";
    default:
      return "Grava o que o app enxerga de cada fonte de carga: Bluetooth, HID, XInput e o que o Windows guarda.";
  }
}

function primeiraLinha(notas: string | null): string {
  const linha = (notas ?? "")
    .split(/\r?\n/)
    .map((l) => l.replace(/^﻿/, "").trim())
    .find((l) => l.length > 0 && !/^kontro[\s\d.]*$/i.test(l));

  if (!linha) return "";
  return linha.length > 150 ? `${linha.slice(0, 147)}...` : linha;
}

function MiniTela({ x, y, solta }: { x: number; y: number; solta: boolean }) {
  return (
    <span className={solta ? "mini-tela solta" : "mini-tela"} aria-hidden="true">
      <span className="mini-pilula" style={{ left: 1 + x * 36, top: 1 + y * 22 }} />
    </span>
  );
}

function descricaoDoAtalho(base: string, combinacao: string, recusados: string[]): string {
  if (recusados.includes(combinacao)) {
    return "Outro programa já usa essa combinação. Escolha outra.";
  }
  return base;
}

function Captura({
  combinacao,
  aoTrocar,
  desabilitado,
}: {
  combinacao: string;
  aoTrocar: (combinacao: string) => void;
  desabilitado: boolean;
}) {
  const [ouvindo, setOuvindo] = useState(false);

  useEffect(() => {
    if (!ouvindo) return;

    const aoTeclar = (evento: KeyboardEvent) => {
      evento.preventDefault();
      evento.stopPropagation();

      if (evento.key === "Escape") {
        setOuvindo(false);
        return;
      }

      const nova = lerCombinacao(evento);
      if (!nova) return;

      setOuvindo(false);
      aoTrocar(nova);
    };

    window.addEventListener("keydown", aoTeclar, true);
    return () => window.removeEventListener("keydown", aoTeclar, true);
  }, [ouvindo, aoTrocar]);

  return (
    <button
      className={ouvindo ? "captura ouvindo" : "captura"}
      disabled={desabilitado}
      onClick={() => setOuvindo(!ouvindo)}
    >
      {ouvindo ? (
        <span className="pedindo">pressione a combinação</span>
      ) : (
        combinacao.split("+").map((parte, i) => (
          <kbd className="tecla" key={i}>
            {nomeDaTecla(parte)}
          </kbd>
        ))
      )}
    </button>
  );
}

function lerCombinacao(evento: KeyboardEvent): string | null {
  if (MODIFICADORES.includes(evento.key) || !evento.code) return null;

  const partes: string[] = [];
  if (evento.ctrlKey) partes.push("Ctrl");
  if (evento.altKey) partes.push("Alt");
  if (evento.shiftKey) partes.push("Shift");
  if (evento.metaKey) partes.push("Super");
  if (partes.length === 0) return null;

  partes.push(evento.code);
  return partes.join("+");
}

function nomeDaTecla(parte: string): string {
  if (NOME_DA_TECLA[parte]) return NOME_DA_TECLA[parte];
  if (parte.startsWith("Key")) return parte.slice(3);
  if (parte.startsWith("Digit")) return parte.slice(5);
  if (parte.startsWith("Numpad")) return `Num ${parte.slice(6)}`;
  return parte;
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
