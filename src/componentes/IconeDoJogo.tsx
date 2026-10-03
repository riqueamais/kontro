import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { Glifo } from "./Glifo";
import "./icone-do-jogo.css";

export function IconeDoJogo({ jogo, tamanho = 32 }: { jogo: string | null; tamanho?: number }) {
  const [uri, setUri] = useState<string | null | undefined>(undefined);

  useEffect(() => {
    if (!jogo) {
      setUri(null);
      return;
    }
    let vivo = true;
    setUri(undefined);
    invoke<string | null>("icone_do_jogo", { nome: jogo })
      .then((achado) => {
        if (vivo) setUri(achado);
      })
      .catch(() => {
        if (vivo) setUri(null);
      });
    return () => {
      vivo = false;
    };
  }, [jogo]);

  const estilo = { width: tamanho, height: tamanho };

  if (!jogo) {
    return (
      <span className="icone-do-jogo sem-jogo" style={estilo}>
        <Glifo tamanho={Math.round(tamanho * 0.625)} cor="var(--text-tertiary)" />
      </span>
    );
  }

  return (
    <span className="icone-do-jogo" style={estilo}>
      {uri ? (
        <img src={uri} alt="" width={tamanho} height={tamanho} />
      ) : uri === null ? (
        <span className="icone-do-jogo-inicial">{jogo.trim().charAt(0).toUpperCase()}</span>
      ) : null}
    </span>
  );
}
