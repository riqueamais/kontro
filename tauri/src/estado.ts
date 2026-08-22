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
