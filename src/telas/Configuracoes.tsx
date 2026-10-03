import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import { useEffect, useRef, useState } from "react";

import {
  ATALHO_DA_PILULA,
  ATALHO_DE_MOVER,
  LIMIARES_CRITICOS,
  LIMIARES_DE_AVISO,
  ciclar,
  salvar,
} from "../ajustes";
import { Chave, Linha, MiniTela } from "../componentes/Controles";
import { decimal } from "../formato";
import {
  Config,
  OverlayMode,
  Recusa,
  Theme,
  useAtalhosRecusados,
  useConfig,
  usePilulaCoberta,
  usePilulaSolta,
} from "../estado";

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

const TEMAS: { id: Theme; rotulo: string; chao: string; realce: string; texto: string }[] = [
  {
    id: "Sistema",
    rotulo: "Do Windows",
    chao: "linear-gradient(115deg, #0b0e11 0 50%, #eef1f5 50% 100%)",
    realce: "#35d7a8",
    texto: "#e8ecef",
  },
  { id: "Noite", rotulo: "Noite", chao: "#0b0e11", realce: "#35d7a8", texto: "#e8ecef" },
  { id: "Preto", rotulo: "Preto", chao: "#000000", realce: "#35d7a8", texto: "#e8ecef" },
  { id: "Ardosia", rotulo: "Ardósia", chao: "#0d1219", realce: "#4ea8ff", texto: "#e7edf4" },
  { id: "Brasa", rotulo: "Brasa", chao: "#100c0a", realce: "#e0925a", texto: "#f2ebe5" },
  { id: "Dia", rotulo: "Dia", chao: "#eef1f5", realce: "#0a6e5b", texto: "#0f151b" },
];

const MODOS: Record<OverlayMode, string> = {
  Desligada: "Desligada",
  EmJogo: "Só em jogo",
  Sempre: "Sempre visível",
};

interface VersaoNova {
  versao: string;
  notas: string | null;
  beta: boolean;
  atual: string;
}

interface Busca {
  estado: "nova" | "em-dia" | "falhou";
  versao: string | null;
  notas: string | null;
  beta: boolean;
  atual: string;
  motivo: string | null;
}

type Andamento =
  | { etapa: "baixando"; bytes: number; total: number | null }
  | { etapa: "instalando" };

type Passo =
  | { tipo: "parado" }
  | { tipo: "procurando" }
  | { tipo: "atualizado" }
  | { tipo: "preparando" }
  | { tipo: "baixando"; porcento: number | null; bytes: number }
  | { tipo: "instalando" }
  | { tipo: "falhou"; motivo: string; ao: "verificar" | "atualizar" };

export function Configuracoes({ ativa, aoRever }: { ativa: boolean; aoRever: () => void }) {
  const doRust = useConfig();
  const [cfg, setCfg] = useState<Config | null>(doRust);
  const [nova, setNova] = useState<VersaoNova | null>(null);
  const [passo, setPasso] = useState<Passo>({ tipo: "parado" });
  const [telas, setTelas] = useState(1);
  const [atual, setAtual] = useState("");
  const [diagnostico, setDiagnostico] = useState<"parado" | "gravando" | "pronto" | "falhou">(
    "parado",
  );
  const solta = usePilulaSolta();
  const coberta = usePilulaCoberta();
  const recusados = useAtalhosRecusados();

  useEffect(() => {
    if (doRust) setCfg(doRust);
  }, [doRust]);

  useEffect(() => {
    invoke<string>("versao_do_app").then(setAtual).catch(() => {});
  }, []);

  useEffect(() => {
    if (!ativa) return;
    invoke<VersaoNova | null>("versao_disponivel").then(setNova).catch(() => {});
    invoke<number>("quantidade_de_telas").then(setTelas).catch(() => {});
  }, [ativa]);

  const procurar = async () => {
    setPasso({ tipo: "procurando" });
    try {
      const busca = await invoke<Busca>("procurar_atualizacao");

      if (busca.estado === "falhou") {
        setPasso({
          tipo: "falhou",
          ao: "verificar",
          motivo: busca.motivo ?? "não deu para falar com o GitHub",
        });
        return;
      }

      if (busca.estado === "nova" && busca.versao) {
        setNova({ versao: busca.versao, notas: busca.notas, beta: busca.beta, atual: busca.atual });
        setPasso({ tipo: "parado" });
        return;
      }

      setNova(null);
      setPasso({ tipo: "atualizado" });
    } catch {
      setPasso({
        tipo: "falhou",
        ao: "verificar",
        motivo: "não deu para falar com o GitHub",
      });
    }
  };

  const atualizarAgora = async () => {
    setPasso({ tipo: "preparando" });
    const parar = await listen<Andamento>("kontro://atualizacao", ({ payload }) => {
      if (payload.etapa === "instalando") {
        setPasso({ tipo: "instalando" });
        return;
      }
      const { bytes, total } = payload;
      setPasso({
        tipo: "baixando",
        porcento: total ? Math.min(100, Math.round((bytes / total) * 100)) : null,
        bytes,
      });
    });
    try {
      const achou = await invoke<boolean>("instalar_atualizacao");
      if (!achou) {
        setNova(null);
        setPasso({ tipo: "atualizado" });
        return;
      }
      await relaunch();
    } catch (e) {
      setPasso({
        tipo: "falhou",
        ao: "atualizar",
        motivo: typeof e === "string" ? e : "não deu para baixar a versão nova",
      });
    } finally {
      parar();
    }
  };

  if (!cfg) return null;
  const gravar = (mudanca: Partial<Config>) => setCfg(salvar(cfg, mudanca));
  const ocupado =
    passo.tipo === "procurando" ||
    passo.tipo === "preparando" ||
    passo.tipo === "baixando" ||
    passo.tipo === "instalando";

  const trocarCanal = async (BetaUpdates: boolean) => {
    const novas = { ...cfg, BetaUpdates };
    setCfg(novas);
    await invoke("salvar_configuracoes", { novas });
    if (!ocupado) await procurar();
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
          className="botao"
          onClick={() =>
            gravar({ CloseAction: ciclar(cfg.CloseAction, ["MinimizeToTray", "Exit"] as const) })
          }
        >
          {cfg.CloseAction === "MinimizeToTray" ? "Minimizar" : "Encerrar"}
        </button>
      </Linha>
      <h2>Aparência</h2>
      <Linha
        titulo="Tema"
        descricao={`${TEMAS.find((t) => t.id === cfg.Theme)?.rotulo ?? "Noite"} · o chão da janela. As cores de carga não mudam.`}
        classe="tema"
      >
        <SeletorDeTema escolhido={cfg.Theme} aoEscolher={(Theme) => gravar({ Theme })} />
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
          className="botao"
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
          className="botao"
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
          className="botao"
          onClick={() =>
            gravar({
              OverlayMode: ciclar(cfg.OverlayMode, ["Desligada", "EmJogo", "Sempre"] as const),
            })
          }
        >
          {MODOS[cfg.OverlayMode]}
        </button>
      </Linha>
      {coberta && (
        <p className="coberta" role="status">
          Agora: {coberta} está por cima da pílula e não deixa ela voltar. Ela tenta de novo
          quando você trocar de janela.
        </p>
      )}
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
          className={solta ? "botao destaque" : "botao"}
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
          className="botao"
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
          className="botao"
          onClick={() => gravar({ OverlayScale: ciclar(cfg.OverlayScale, tamanhos()) })}
        >
          {rotulo(TAMANHOS, cfg.OverlayScale)}
        </button>
      </Linha>
      <Linha titulo="Transparência" descricao="Para a pílula não competir com o HUD do jogo.">
        <button
          className="botao"
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
      <LinhaDeAtalho
        titulo="Mostrar e esconder a pílula"
        base="Tira a pílula da frente e traz de volta."
        recusa={recusados.find((r) => r.atalho === "Mostrar")}
        combinacao={cfg.OverlayShortcut}
        padrao={ATALHO_DA_PILULA}
        outra={cfg.OverlayMoveShortcut}
        nomeDaOutra="soltar a pílula"
        desabilitado={!cfg.OverlayShortcutEnabled}
        ativa={ativa}
        aoTrocar={(c) => gravar({ OverlayShortcut: c })}
      />
      <LinhaDeAtalho
        titulo="Soltar a pílula para mover"
        base="Solta a pílula para arrastar, e prende de novo onde você largar."
        recusa={recusados.find((r) => r.atalho === "Mover")}
        combinacao={cfg.OverlayMoveShortcut}
        padrao={ATALHO_DE_MOVER}
        outra={cfg.OverlayShortcut}
        nomeDaOutra="mostrar a pílula"
        desabilitado={!cfg.OverlayShortcutEnabled}
        ativa={ativa}
        aoTrocar={(c) => gravar({ OverlayMoveShortcut: c })}
      />
      <h2>Versão</h2>
      {nova && <Novidade nova={nova} passo={passo} aoAtualizar={() => void atualizarAgora()} />}
      <Linha titulo={tituloDaVersao(passo)} descricao={detalheDaVersao(atual, passo)}>
        <button className="botao" disabled={ocupado} onClick={() => void procurar()}>
          {passo.tipo === "procurando" ? "Procurando..." : "Procurar"}
        </button>
      </Linha>
      <Linha
        titulo="Avisar sobre versões novas"
        descricao="Consulta o repositório de tempos em tempos, sem baixar nada sozinho."
      >
        <Chave ligado={cfg.AutoCheckUpdates} aoTrocar={(v) => gravar({ AutoCheckUpdates: v })} />
      </Linha>
      <Linha
        titulo="Receber versões beta"
        descricao="Versões de teste, publicadas antes da versão final. Podem ter defeitos que a final não terá."
      >
        <Chave ligado={cfg.BetaUpdates} aoTrocar={(v) => void trocarCanal(v)} />
      </Linha>
      <h2>Problemas</h2>
      <Linha
        titulo="Passo a passo"
        descricao="As seis telas que explicam o app, de novo do começo."
      >
        <button className="botao" onClick={aoRever}>
          Rever
        </button>
      </Linha>
      <Linha titulo="Salvar diagnóstico" descricao={textoDoDiagnostico(diagnostico)}>
        <button
          className="botao"
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

function tituloDaVersao(passo: Passo): string {
  switch (passo.tipo) {
    case "atualizado":
      return "Você está na versão mais recente";
    case "falhou":
      return passo.ao === "verificar"
        ? "Sem resposta do GitHub"
        : "A atualização não terminou";
    default:
      return "Procurar atualizações";
  }
}
function detalheDaVersao(atual: string, passo: Passo): string {
  if (passo.tipo === "falhou") return passo.motivo;
  if (passo.tipo === "procurando") return "Consultando o repositório...";
  if (passo.tipo === "atualizado") return "Nada novo publicado desde esta versão.";

  const onde = atual ? `Você está na ${atual}. ` : "";
  return `${onde}O app verifica sozinho uma vez por dia, e você pode procurar quando quiser.`;
}

function Novidade({
  nova,
  passo,
  aoAtualizar,
}: {
  nova: VersaoNova;
  passo: Passo;
  aoAtualizar: () => void;
}) {
  const baixando = passo.tipo === "baixando";
  const instalando = passo.tipo === "instalando";
  const preparando = passo.tipo === "preparando";
  const andando = baixando || instalando || preparando;
  const porcento = passo.tipo === "baixando" ? passo.porcento : null;
  const bytes = passo.tipo === "baixando" ? passo.bytes : 0;
  const indefinido = preparando || (baixando && porcento === null);

  return (
    <section className="cartao novidade">
      <div className="cabeca">
        <div className="leitura">
          <div className="dispositivo">
            {nova.beta ? "Beta" : "Versão"} {nova.versao} disponível
          </div>
          <div className="rodape">
            você está na <span className="mono">{nova.atual}</span>
          </div>
        </div>
        {!andando && (
          <button className="botao destaque" onClick={aoAtualizar}>
            Atualizar agora
          </button>
        )}
      </div>

      {andando && (
        <div className="andamento">
          <div className={`trilho${indefinido ? " indefinido" : ""}`} aria-hidden="true">
            <span
              style={
                indefinido ? undefined : { width: `${instalando ? 100 : (porcento ?? 0)}%` }
              }
            />
          </div>
          <div className="rodape">{andamento(passo, porcento, bytes)}</div>
        </div>
      )}

      <Notas texto={nova.notas} />
    </section>
  );
}

function andamento(passo: Passo, porcento: number | null, bytes: number): React.ReactNode {
  if (passo.tipo === "preparando") return "consultando a release publicada";
  if (passo.tipo === "instalando") return "instalando — o app reinicia sozinho";
  const quanto = porcento === null ? megabytes(bytes) : `${porcento}%`;
  return (
    <>
      baixando <span className="mono">{quanto}</span> — o pacote é verificado antes de rodar
    </>
  );
}

function megabytes(bytes: number): string {
  return `${decimal(bytes / 1_048_576)} MB`;
}

function Notas({ texto }: { texto: string | null }) {
  const blocos = emBlocos(texto);
  if (blocos.length === 0) return null;

  return (
    <div className="notas">
      {blocos.map((bloco, i) =>
        bloco.tipo === "titulo" ? (
          <h3 key={i}>{bloco.texto}</h3>
        ) : (
          <ul key={i}>
            {bloco.itens.map((item, j) => (
              <li key={j}>{item}</li>
            ))}
          </ul>
        ),
      )}
    </div>
  );
}

type Bloco = { tipo: "titulo"; texto: string } | { tipo: "lista"; itens: string[] };

function emBlocos(texto: string | null): Bloco[] {
  const blocos: Bloco[] = [];
  let lista: string[] | null = null;

  for (const bruta of (texto ?? "").split(/\r?\n/)) {
    const linha = bruta.replace(/^﻿/, "").trim();
    if (!linha || /^kontro[\s\d.]*$/i.test(linha)) continue;

    const marcador = linha.match(/^[-*·]\s+(.*)$/);
    if (marcador) {
      if (!lista) {
        lista = [];
        blocos.push({ tipo: "lista", itens: lista });
      }
      lista.push(marcador[1]);
      continue;
    }

    if (lista && /^\s/.test(bruta)) {
      lista[lista.length - 1] = `${lista[lista.length - 1]} ${linha}`;
      continue;
    }

    lista = null;
    blocos.push({ tipo: "titulo", texto: linha });
  }

  return blocos;
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
      return "Grava o que o app enxerga de cada fonte de carga: Bluetooth, HID, XInput e o que o Windows guarda. Inclui o nome do programa em primeiro plano na hora do clique.";
  }
}

function SeletorDeTema({
  escolhido,
  aoEscolher,
}: {
  escolhido: Theme;
  aoEscolher: (tema: Theme) => void;
}) {
  const amostras = useRef<Partial<Record<Theme, HTMLButtonElement | null>>>({});

  const aoTeclar = (evento: React.KeyboardEvent) => {
    const passo = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[evento.key];
    if (!passo) return;
    evento.preventDefault();
    const i = TEMAS.findIndex((t) => t.id === escolhido);
    const proximo = TEMAS[(i + passo + TEMAS.length) % TEMAS.length].id;
    aoEscolher(proximo);
    amostras.current[proximo]?.focus();
  };

  return (
    <div className="temas" role="radiogroup" aria-label="Tema" onKeyDown={aoTeclar}>
      {TEMAS.map((t) => {
        const escolhida = escolhido === t.id;
        return (
          <button
            key={t.id}
            ref={(el) => {
              amostras.current[t.id] = el;
            }}
            role="radio"
            aria-checked={escolhida}
            tabIndex={escolhida ? 0 : -1}
            className={`amostra-de-tema${escolhida ? " escolhida" : ""}`}
            style={
              {
                "--amostra-chao": t.chao,
                "--amostra-cor": t.realce,
                "--amostra-texto": t.texto,
              } as React.CSSProperties
            }
            onClick={() => aoEscolher(t.id)}
          >
            <span className="caixa" aria-hidden="true">
              <span className="pingo" />
              <span className="traco" />
            </span>
            <span className="nome">{t.rotulo}</span>
          </button>
        );
      })}
    </div>
  );
}

const PRAZO_DA_DICA_MS = 2000;

function descricaoDoAtalho(base: string, recusa: Recusa | undefined) {
  if (!recusa) return { texto: base, erro: false };
  if (recusa.motivo === "Invalida") return { texto: "O Windows não aceita essa combinação.", erro: true };
  return { texto: "Outro programa já usa essa combinação. Escolha outra.", erro: true };
}

function LinhaDeAtalho({
  titulo,
  base,
  recusa,
  combinacao,
  padrao,
  outra,
  nomeDaOutra,
  desabilitado,
  ativa,
  aoTrocar,
}: {
  titulo: string;
  base: string;
  recusa: Recusa | undefined;
  combinacao: string;
  padrao: string;
  outra: string;
  nomeDaOutra: string;
  desabilitado: boolean;
  ativa: boolean;
  aoTrocar: (combinacao: string) => void;
}) {
  const { texto, erro } = descricaoDoAtalho(base, recusa);
  return (
    <Linha titulo={titulo} descricao={texto} erro={erro}>
      {combinacao !== padrao && (
        <button
          className="botao miudo padrao-do-atalho"
          disabled={desabilitado}
          onClick={() => aoTrocar(padrao)}
        >
          Padrão
        </button>
      )}
      <Captura
        combinacao={combinacao}
        outra={outra}
        nomeDaOutra={nomeDaOutra}
        recusada={erro}
        desabilitado={desabilitado}
        ativa={ativa}
        aoTrocar={aoTrocar}
      />
    </Linha>
  );
}

function Captura({
  combinacao,
  outra,
  nomeDaOutra,
  recusada,
  aoTrocar,
  desabilitado,
  ativa,
}: {
  combinacao: string;
  outra: string;
  nomeDaOutra: string;
  recusada: boolean;
  aoTrocar: (combinacao: string) => void;
  desabilitado: boolean;
  ativa: boolean;
}) {
  const [ouvindo, setOuvindo] = useState(false);
  const [dica, setDica] = useState<string | null>(null);
  const botao = useRef<HTMLButtonElement>(null);
  const gravou = useRef(false);

  useEffect(() => {
    if (!ativa) setOuvindo(false);
  }, [ativa]);

  useEffect(() => {
    if (!ouvindo) return;
    gravou.current = false;
    void invoke("pausar_atalhos", { pausar: true });
    return () => {
      setDica(null);
      if (!gravou.current) void invoke("pausar_atalhos", { pausar: false });
    };
  }, [ouvindo]);

  useEffect(() => {
    if (!dica) return;
    const apagar = window.setTimeout(() => setDica(null), PRAZO_DA_DICA_MS);
    return () => window.clearTimeout(apagar);
  }, [dica]);

  useEffect(() => {
    if (!ouvindo || !ativa) return;

    const aoTeclar = (evento: KeyboardEvent) => {
      if (evento.key === "Tab") {
        setOuvindo(false);
        return;
      }

      evento.preventDefault();
      evento.stopPropagation();

      if (evento.key === "Escape") {
        setOuvindo(false);
        return;
      }

      const nova = lerCombinacao(evento);
      if (!nova) {
        if (!MODIFICADORES.includes(evento.key)) setDica("Junte Ctrl, Alt, Shift ou Win");
        return;
      }
      if (nova === outra) {
        setDica(`Já é o atalho de ${nomeDaOutra}`);
        return;
      }

      gravou.current = true;
      setOuvindo(false);
      aoTrocar(nova);
    };

    const largar = () => setOuvindo(false);
    const aoApontar = (evento: PointerEvent) => {
      if (!botao.current?.contains(evento.target as Node)) setOuvindo(false);
    };

    window.addEventListener("keydown", aoTeclar, true);
    window.addEventListener("blur", largar);
    document.addEventListener("pointerdown", aoApontar, true);
    return () => {
      window.removeEventListener("keydown", aoTeclar, true);
      window.removeEventListener("blur", largar);
      document.removeEventListener("pointerdown", aoApontar, true);
    };
  }, [ouvindo, ativa, aoTrocar, outra, nomeDaOutra]);

  const classe = ["captura", ouvindo && "ouvindo", recusada && !ouvindo && "recusada"]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      ref={botao}
      className={classe}
      disabled={desabilitado}
      onClick={() => setOuvindo(!ouvindo)}
    >
      {ouvindo ? (
        <span className="pedindo" role="status">
          {dica ?? "Pressione a combinação · Esc cancela"}
        </span>
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
