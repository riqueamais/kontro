import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import type { Estado } from "../estado";

const ESPERA_MAXIMA_MS = 5000;

export function BotaoLerAgora({ estado, className }: { estado: Estado; className?: string }) {
  const [lendoSobre, setLendoSobre] = useState<Estado | null>(null);
  const lendo = lendoSobre === estado;

  useEffect(() => {
    if (!lendo) return;
    const desistir = setTimeout(() => setLendoSobre(null), ESPERA_MAXIMA_MS);
    return () => clearTimeout(desistir);
  }, [lendo]);

  const procurar = estado.via === "Desligado";

  return (
    <button
      className={className}
      disabled={lendo}
      aria-busy={lendo}
      title={procurar ? "Procura o controle e relê a bateria" : undefined}
      onClick={() => {
        setLendoSobre(estado);
        invoke("ler_agora").catch(() => setLendoSobre(null));
      }}
    >
      {lendo ? "Lendo…" : procurar ? "Procurar" : "Ler agora"}
    </button>
  );
}
