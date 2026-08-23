import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "./App";
import "./estilo/tokens.css";
import "./estilo/base.css";

ReactDOM.createRoot(document.getElementById("raiz")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
