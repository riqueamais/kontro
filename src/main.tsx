import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "./App";
import "./estilo/tokens.css";
import "./estilo/base.css";

const pedido = new URLSearchParams(window.location.search);
document.body.dataset.janela = pedido.get("janela") ?? "principal";
document.body.dataset.tema = pedido.get("tema") ?? "noite";
document.body.dataset.material = pedido.get("material") ?? "nao";

ReactDOM.createRoot(document.getElementById("raiz")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
