import { useId } from "react";

import { ciclar } from "../ajustes";
import { useLinha, useNomeDoControle } from "./Controles";

export function BotaoDeCiclo<T extends string | number>({
  opcoes,
  valor,
  rotulo,
  aoMudar,
}: {
  opcoes: readonly T[];
  valor: T;
  rotulo: (valor: T) => string;
  aoMudar: (valor: T) => void;
}) {
  const linha = useLinha();
  const id = useId();
  const nome = useNomeDoControle(id);
  const andar = (sentido: 1 | -1) => aoMudar(ciclar(valor, opcoes, sentido));

  return (
    <button
      id={id}
      className="botao"
      disabled={linha?.desabilitada}
      onClick={(e) => andar(e.shiftKey ? -1 : 1)}
      onKeyDown={(e) => {
        const sentido = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1 }[e.key] as
          | 1
          | -1
          | undefined;
        if (!sentido) return;
        e.preventDefault();
        andar(sentido);
      }}
      {...nome}
    >
      {rotulo(valor)}
    </button>
  );
}
