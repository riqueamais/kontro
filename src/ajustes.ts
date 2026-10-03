import { invoke } from "@tauri-apps/api/core";

import { Config } from "./estado";

export const LIMIARES_DE_AVISO = [40, 30, 25, 20, 15, 10] as const;
export const LIMIARES_CRITICOS = [20, 15, 10, 5] as const;

export const ATALHO_DA_PILULA = "Ctrl+Shift+KeyK";
export const ATALHO_DE_MOVER = "Ctrl+Shift+KeyM";

export function ciclar<T extends string | number>(atual: T, opcoes: readonly T[]): T {
  const i = opcoes.indexOf(atual);
  return i < 0 ? opcoes[0] : opcoes[(i + 1) % opcoes.length];
}

export interface Salvo {
  config: Config;
  recusadas: string[];
}

export function salvar(cfg: Config, mudanca: Partial<Config>): Promise<Salvo> {
  return invoke<Salvo>("salvar_configuracoes", { novas: { ...cfg, ...mudanca } });
}
