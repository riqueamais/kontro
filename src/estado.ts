import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export type LinkMode = "Offline" | "Bluetooth" | "Cable" | "Wireless";
export type Precisao = "Nenhuma" | "Aproximada" | "Exata";

/// O estado chega pronto do Rust: a interface nunca reimplementa a regra de o que
/// pode ser afirmado sobre a carga.
export interface Estado {
  mode: LinkMode;
  percent: number | null;
  precisao: Precisao;
  nivel: number | null;
  readAt: number | null;
  charging: boolean;
  stale: boolean;
  deviceName: string;
  address: string | null;
  key: string;
  knownCount: number;
  preenchimento: number | null;
  textoDaCarga: string;
  textoDaLigacao: string;
  temNumero: boolean;
  conectadoSemCarga: boolean;
  girando: boolean;
  autonomia: string | null;
}

export interface Amostra {
  t: number;
  p: number;
}

export function useEstado(): Estado | null {
  const [estado, setEstado] = useState<Estado | null>(null);

  useEffect(() => {
    let vivo = true;

    invoke<Estado>("estado_atual").then((e) => {
      if (vivo) setEstado(e);
    });

    const parar = listen<Estado>("kontro://estado", (evento) => {
      if (vivo) setEstado(evento.payload);
    });

    return () => {
      vivo = false;
      parar.then((f) => f());
    };
  }, []);

  return estado;
}

/// Cor do anel para um estado. Teal nao fala de carga, fala de estar na energia.
export function corDoAnel(estado: Estado): string {
  if (estado.girando) return "var(--accent-teal)";
  if (estado.mode === "Offline" || estado.preenchimento === null) return "var(--gray)";
  if (estado.preenchimento < 30) return "var(--red)";
  if (estado.preenchimento < 60) return "var(--amber)";
  return "var(--accent-green)";
}

export function qualJanela(): string {
  return new URLSearchParams(window.location.search).get("janela") ?? "principal";
}

export type CloseAction = "MinimizeToTray" | "Exit";
export type OverlayMode = "Desligada" | "EmJogo" | "Sempre";
export type OverlayCorner =
  | "SuperiorEsquerdo"
  | "SuperiorDireito"
  | "InferiorEsquerdo"
  | "InferiorDireito";

/// Os nomes vem em PascalCase porque o arquivo em disco e o mesmo desde a versao em
/// .NET, e trocar os nomes agora faria todo mundo perder o que ja tinha configurado.
export interface Config {
  StartWithWindows: boolean;
  StartMinimized: boolean;
  CloseAction: CloseAction;
  NotificationsEnabled: boolean;
  WarnThreshold: number;
  CriticalThreshold: number;
  ConnectToastEnabled: boolean;
  OverlayMode: OverlayMode;
  OverlayCorner: OverlayCorner;
  OverlayMonitor: number;
  OverlayScale: number;
  OverlayOpacity: number;
  AutoCheckUpdates: boolean;
  OverlayShortcutEnabled: boolean;
  FirstRunDone: boolean;
}

/// A configuracao, acompanhando quem a mudar em outra janela.
///
/// A sobreposicao e uma janela separada da tela de ajustes: sem escutar o aviso, mexer
/// no tamanho da pilula so teria efeito na proxima vez que ela aparecesse.
export function useConfig(): Config | null {
  const [cfg, setCfg] = useState<Config | null>(null);

  useEffect(() => {
    let vivo = true;

    invoke<Config>("configuracoes").then((c) => {
      if (vivo) setCfg(c);
    });

    const parar = listen<Config>("kontro://config", (evento) => {
      if (vivo) setCfg(evento.payload);
    });

    return () => {
      vivo = false;
      parar.then((f) => f());
    };
  }, []);

  return cfg;
}

/// Quando a leitura foi feita, em texto.
///
/// A data entra sempre que a leitura nao for de hoje. So a hora fazia um numero de tres
/// dias atras aparecer como "lido as 21:34" e passar por recente -- e o horario esta ali
/// justamente para levantar essa duvida, nao para escondê-la.
export function quandoLeu(estado: Estado): string {
  if (!estado.readAt) return "sem leitura ainda";

  const quando = new Date(estado.readAt);
  const verbo = estado.stale ? "lido" : "atualizado";
  const hora = quando.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });

  const dias = diasAtras(quando);
  if (dias === 0) return `${verbo} às ${hora}`;
  if (dias === 1) return `${verbo} ontem às ${hora}`;

  const dia = quando.toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" });
  return `${verbo} em ${dia}, às ${hora}`;
}

/// Quantos dias de calendario separam a data de hoje. Comparar timestamps daria "ontem"
/// para uma leitura de vinte minutos atras feita pouco depois da meia-noite.
function diasAtras(quando: Date): number {
  const meiaNoite = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  return Math.round((meiaNoite(new Date()) - meiaNoite(quando)) / 86_400_000);
}
