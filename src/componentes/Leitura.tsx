import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { type Estado, useLimiares } from "../estado";
import { detalhe, quandoLeu } from "../formato";
import "./leitura.css";

export function Leitura({
  estado,
  comAutonomia = true,
  renomeavel = false,
}: {
  estado: Estado;
  comAutonomia?: boolean;
  renomeavel?: boolean;
}) {
  const { texto, hora } = quandoLeu(estado);
  const antiga = estado.leituraAntiga && estado.lidoEm !== null && !estado.procurando;
  const limiares = useLimiares();
  const linha = detalhe(estado, comAutonomia, limiares);
  const conhecido = renomeavel && estado.quantosConhecidos > 0 && !estado.procurando;
  const nomeNoTitulo = conhecido && estado.titulo === estado.nome;
  const nomeNoDetalhe = conhecido && !nomeNoTitulo && linha === estado.nome;

  return (
    <div className={antiga ? "leitura antiga" : "leitura"}>
      <div className="dispositivo">
        {nomeNoTitulo ? <NomeEditavel estado={estado} /> : estado.titulo}
      </div>
      <div className="detalhe">
        {nomeNoDetalhe ? <NomeEditavel estado={estado} /> : linha || "\u00a0"}
      </div>
      <div className="rodape">
        {texto || "\u00a0"}
        {hora && (
          <>
            {" "}
            <span className="mono">{hora}</span>
          </>
        )}
      </div>
    </div>
  );
}

function NomeEditavel({ estado }: { estado: Estado }) {
  const [editando, setEditando] = useState(false);
  const campo = useRef<HTMLInputElement>(null);
  const botao = useRef<HTMLButtonElement>(null);
  const sair = (voltarOFoco = true) => {
    setEditando(false);
    if (voltarOFoco) requestAnimationFrame(() => botao.current?.focus());
  };

  useEffect(() => {
    if (!editando) return;
    campo.current?.focus();
    campo.current?.select();
  }, [editando]);

  const salvar = (voltarOFoco = true) => {
    const nome = campo.current?.value ?? "";
    if (nome !== estado.nome) void invoke("renomear_controle", { chave: estado.chave, nome });
    sair(voltarOFoco);
  };

  if (editando) {
    return (
      <input
        ref={campo}
        className="renomear"
        defaultValue={estado.nome}
        maxLength={40}
        placeholder="Nome do controle"
        aria-label="Nome do controle"
        onBlur={() => salvar(false)}
        onKeyDown={(e) => {
          if (e.key === "Enter") salvar();
          if (e.key === "Escape") sair();
        }}
      />
    );
  }

  return (
    <button ref={botao} className="item-nome" title="Renomear" onClick={() => setEditando(true)}>
      <span className="item-nome-texto">{estado.nome}</span>
      <svg className="lapis" width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
        <path
          d="M8.2 1.8 10.2 3.8 4 10H2V8Z"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
      </svg>
    </button>
  );
}
