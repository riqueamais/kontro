import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useSyncExternalStore } from "react";

import { Config } from "./config.gerada";

export type { CloseAction, Config, OverlayMode, Theme } from "./config.gerada";

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
  autonomiaMinutos: number | null;
  consumoPorHora: number | null;
  procurando: boolean;
  titulo: string;
}

export interface Amostra {
  t: number;
  p: number;
  via: Via | null;
}

export interface Recusa {
  atalho: "Mostrar" | "Mover";
  combinacao: string;
  motivo: "Invalida" | "EmUso";
}

export interface VersaoNova {
  versao: string;
  notas: string | null;
  beta: boolean;
  atual: string;
}

export interface Sessao {
  inicio: number;
  fim: number;
  de: number;
  ate: number;
  jogo: string | null;
}

interface Fonte<T> {
  assinar: (aviso: () => void) => () => void;
  ler: () => T;
}

function criarFonte<T>(comando: string | null, evento: string, inicial: T): Fonte<T> {
  let atual = inicial;
  let ligada = false;
  let chegouPeloEvento = false;
  const assinantes = new Set<() => void>();

  const trocar = (valor: T) => {
    atual = valor;
    assinantes.forEach((aviso) => aviso());
  };

  const ligar = () => {
    if (ligada) return;
    ligada = true;
    listen<T>(evento, (e) => {
      chegouPeloEvento = true;
      trocar(e.payload);
    });
    if (comando) {
      invoke<T>(comando).then((valor) => {
        if (!chegouPeloEvento) trocar(valor);
      });
    }
  };

  return {
    assinar(aviso) {
      ligar();
      assinantes.add(aviso);
      return () => {
        assinantes.delete(aviso);
      };
    },
    ler: () => atual,
  };
}

function doEndereco(nome: string): string | null {
  return new URLSearchParams(window.location.search).get(nome);
}

function useFonte<T>(fonte: Fonte<T>): T {
  return useSyncExternalStore(fonte.assinar, fonte.ler);
}

const fonteDoEstado = criarFonte<Estado | null>("estado_atual", "kontro://estado", null);
const fonteDosControles = criarFonte<Estado[]>("controles", "kontro://controles", []);
const fonteDaConfig = criarFonte<Config | null>("configuracoes", "kontro://config", null);
const fonteDaPilulaSolta = criarFonte<boolean>("pilula_solta", "kontro://solta", false);
const fonteDaPilulaCoberta = criarFonte<string | null>("pilula_coberta", "kontro://coberta", null);
const fonteDosAtalhosRecusados = criarFonte<Recusa[]>("atalhos_recusados", "kontro://atalhos", []);
const fonteDaNovidade = criarFonte<VersaoNova | null>(
  "versao_disponivel",
  "kontro://novidade",
  null,
);
const fonteDoTemaDoSistema = criarFonte<boolean>(
  "windows_no_claro",
  "kontro://tema-do-sistema",
  doEndereco("tema") === "dia",
);
const fonteDoMaterial = criarFonte<boolean>(
  null,
  "kontro://material",
  doEndereco("material") === "sim",
);

export function useEstado(): Estado | null {
  return useFonte(fonteDoEstado);
}

export function useControles(): Estado[] {
  return useFonte(fonteDosControles);
}

export interface Limiares {
  critico: number;
  aviso: number;
}

export const LIMIARES_PADRAO: Limiares = { critico: 10, aviso: 20 };

export function corDoAnel(estado: Estado, limiares: Limiares = LIMIARES_PADRAO): string {
  if (estado.girando || estado.via === "Desligado" || estado.preenchimento === null) {
    return "var(--gray)";
  }
  if (estado.preenchimento < limiares.critico) return "var(--red)";
  if (estado.preenchimento < limiares.aviso) return "var(--amber)";
  return "var(--accent-green)";
}

export function useTema(): string {
  const cfg = useConfig();
  const claro = useFonte(fonteDoTemaDoSistema);

  if (!cfg) return document.body.dataset.tema ?? "noite";
  if (cfg.Theme !== "Sistema") return cfg.Theme.toLowerCase();
  return claro ? "dia" : "noite";
}

export function useMaterial(): boolean {
  return useFonte(fonteDoMaterial);
}

export function useLimiares(): Limiares {
  const cfg = useConfig();
  const critico = cfg?.CriticalThreshold ?? LIMIARES_PADRAO.critico;
  const aviso = cfg?.WarnThreshold ?? LIMIARES_PADRAO.aviso;
  return useMemo(() => ({ critico, aviso }), [critico, aviso]);
}

export function useNovidade(): VersaoNova | null {
  return useFonte(fonteDaNovidade);
}

export function usePilulaSolta(): boolean {
  return useFonte(fonteDaPilulaSolta);
}

export function useAoMudarOHistorico(buscar: () => void, dependencias: unknown[]) {
  useEffect(() => {
    buscar();
    const parar = listen("kontro://historico", () => buscar());
    return () => {
      parar.then((f) => f());
    };
  }, dependencias);
}

export function usePilulaCoberta(): string | null {
  return useFonte(fonteDaPilulaCoberta);
}

export function useAtalhosRecusados(): Recusa[] {
  return useFonte(fonteDosAtalhosRecusados);
}

export function qualJanela(): string {
  return doEndereco("janela") ?? "principal";
}

export function useConfig(): Config | null {
  return useFonte(fonteDaConfig);
}
