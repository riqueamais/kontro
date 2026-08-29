import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { Config } from "./config.gerada";

export type { CloseAction, Config, OverlayCorner, OverlayMode } from "./config.gerada";

export type Via = "Desligado" | "Bluetooth" | "Cabo" | "SemFio";
export type Precisao = "Nenhuma" | "Aproximada" | "Exata";

export interface Estado {
  via: Via;
  percentual: number | null;
  precisao: Precisao;
  nivel: number | null;
  lidoEm: number | null;
  carregando: boolean;
  leituraAntiga: boolean;
  nome: string;
  endereco: string | null;
  chave: string;
  quantosConhecidos: number;
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
  via: Via | null;
}

export interface Sessao {
  inicio: number;
  fim: number;
  de: number;
  ate: number;
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

export function useControles(): Estado[] {
  const [controles, setControles] = useState<Estado[]>([]);

  useEffect(() => {
    let vivo = true;

    invoke<Estado[]>("controles").then((c) => {
      if (vivo) setControles(c);
    });

    const parar = listen<Estado[]>("kontro://controles", (evento) => {
      if (vivo) setControles(evento.payload);
    });

    return () => {
      vivo = false;
      parar.then((f) => f());
    };
  }, []);

  return controles;
}

export interface Limiares {
  critico: number;
  aviso: number;
}

export const LIMIARES_PADRAO: Limiares = { critico: 10, aviso: 20 };

export function corDoAnel(estado: Estado, limiares: Limiares = LIMIARES_PADRAO): string {
  if (estado.girando) return "var(--accent-teal)";
  if (estado.via === "Desligado" || estado.preenchimento === null) return "var(--gray)";
  if (estado.preenchimento < limiares.critico) return "var(--red)";
  if (estado.preenchimento < limiares.aviso) return "var(--amber)";
  return "var(--accent-green)";
}

export function useLimiares(): Limiares {
  const cfg = useConfig();
  if (!cfg) return LIMIARES_PADRAO;
  return { critico: cfg.CriticalThreshold, aviso: cfg.WarnThreshold };
}

export function qualJanela(): string {
  return new URLSearchParams(window.location.search).get("janela") ?? "principal";
}

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
