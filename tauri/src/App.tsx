import { useEffect } from "react";

import { qualJanela } from "./estado";
import { Aviso } from "./telas/Aviso";
import { Painel } from "./telas/Painel";
import { Principal } from "./telas/Principal";
import { Sobreposicao } from "./telas/Sobreposicao";

/**
 * As quatro janelas do app compartilham um unico pacote e se separam pela consulta na
 * URL. Um pacote por janela multiplicaria o mesmo codigo por quatro sem ganhar nada.
 */
export function App() {
  const janela = qualJanela();

  useEffect(() => {
    document.body.dataset.janela = janela;
  }, [janela]);

  switch (janela) {
    case "painel":
      return <Painel />;
    case "sobreposicao":
      return <Sobreposicao />;
    case "aviso":
      return <Aviso />;
    default:
      return <Principal />;
  }
}
