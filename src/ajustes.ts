import { invoke } from "@tauri-apps/api/core";

import { Config } from "./estado";

export const LIMIARES_DE_AVISO = [40, 30, 25, 20, 15, 10] as const;
export const LIMIARES_CRITICOS = [20, 15, 10, 5] as const;

export function ciclar<T extends string | number>(atual: T, opcoes: readonly T[]): T {
  const i = opcoes.indexOf(atual);
  return i < 0 ? opcoes[0] : opcoes[(i + 1) % opcoes.length];
}

export function salvar(cfg: Config, mudanca: Partial<Config>): Config {
  const novas = { ...cfg, ...mudanca };
  void invoke("salvar_configuracoes", { novas });
  return novas;
}
