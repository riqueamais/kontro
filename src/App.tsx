import { invoke } from "@tauri-apps/api/core";
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
    document.body.dataset.janela = janela;
  }, [janela]);

  useEffect(() => {
    document.body.dataset.tema = tema;
  }, [tema]);

  useEffect(() => {
    invoke<boolean>("material_da_janela")
      .then((tem) => {
        document.body.dataset.material = tem ? "sim" : "nao";
      })
      .catch(() => {
        document.body.dataset.material = "nao";
      });
  }, []);
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
