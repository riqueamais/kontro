import { useEffect } from "react";

import { qualJanela, useTema } from "./estado";
import { Aviso } from "./telas/Aviso";
import { Painel } from "./telas/Painel";
import { Principal } from "./telas/Principal";
import { Sobreposicao } from "./telas/Sobreposicao";

export function App() {
  const janela = qualJanela();
  const tema = useTema();

  useEffect(() => {
    document.body.dataset.tema = tema;
  }, [tema]);

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
