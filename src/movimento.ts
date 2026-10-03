import { useSyncExternalStore } from "react";

const consulta = window.matchMedia("(prefers-reduced-motion: reduce)");

function assinar(aviso: () => void) {
  consulta.addEventListener("change", aviso);
  return () => consulta.removeEventListener("change", aviso);
}

export function useMovimentoReduzido(): boolean {
  return useSyncExternalStore(assinar, () => consulta.matches);
}
