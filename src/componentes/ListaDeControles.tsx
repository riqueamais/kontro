import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { Estado, Limiares, corDoAnel, useControles, useLimiares } from "../estado";
import { Anel } from "./Anel";
import { Glifo } from "./Glifo";
import "./lista.css";

export function ListaDeControles({
  principal,
  sempre = false,
}: {
  principal: string;
  sempre?: boolean;
}) {
  const controles = useControles();
  const limiares = useLimiares();
  const [editando, setEditando] = useState<string | null>(null);
  const titulo = useRef<HTMLDivElement>(null);
  if (controles.length < (sempre ? 1 : 2)) return null;
  return (
    <div className={`lista${sempre ? " solta" : ""}`}>
      <div className="lista-titulo" ref={titulo} tabIndex={-1}>
        Seus controles
      </div>
      {controles.map((c) => (
        <Linha
          key={c.chave}
          controle={c}
          limiares={limiares}
          principal={c.chave === principal}
          podeEsquecer={sempre}
          editando={editando === c.chave}
          aoEditar={() => setEditando(c.chave)}
          aoSair={() => setEditando(null)}
          aoEsquecer={() => titulo.current?.focus()}
        />
      ))}
    </div>
  );
}
function Linha({
  controle,
  limiares,
  principal,
  podeEsquecer,
  editando,
  aoEditar,
  aoSair,
  aoEsquecer,
}: {
  controle: Estado;
  limiares: Limiares;
  principal: boolean;
  podeEsquecer: boolean;
  editando: boolean;
  aoEditar: () => void;
  aoSair: () => void;
  aoEsquecer: () => void;
}) {
  const campo = useRef<HTMLInputElement>(null);
  const nome = useRef<HTMLButtonElement>(null);
  const esquecer = useRef<HTMLButtonElement>(null);
  const sair = (voltarOFoco = true) => {
    aoSair();
    if (voltarOFoco) requestAnimationFrame(() => nome.current?.focus());
  };
  const [confirmando, setConfirmando] = useState(false);
  const confirmar = useRef<HTMLButtonElement>(null);
  const cancelar = (voltarOFoco = true) => {
    setConfirmando(false);
    if (voltarOFoco) requestAnimationFrame(() => esquecer.current?.focus());
  };
  useEffect(() => {
    if (editando) {
      campo.current?.focus();
      campo.current?.select();
    }
  }, [editando]);
  const salvar = (voltarOFoco = true) => {
    const nome = campo.current?.value ?? "";
    if (nome !== controle.nome) {
      void invoke("renomear_controle", { chave: controle.chave, nome });
    }
    sair(voltarOFoco);
  };
  const removivel = podeEsquecer && controle.via === "Desligado";
  return (
    <div className={`item${principal ? " principal" : ""}`}>
      <Anel
        valor={controle.preenchimento}
        cor={corDoAnel(controle, limiares)}
        espessura={70}
        tamanho={26}
        girando={controle.girando}
      >
        <Glifo tamanho={11} cor="var(--text-secondary)" />
      </Anel>
      <div className="item-texto">
        {editando ? (
          <input
            ref={campo}
            className="renomear"
            defaultValue={controle.nome}
            maxLength={40}
            placeholder="Nome do controle"
            aria-label="Nome do controle"
            onBlur={() => salvar(false)}
            onKeyDown={(e) => {
              if (e.key === "Enter") salvar();
              if (e.key === "Escape") sair();
            }}
          />
        ) : (
          <span className="item-cabeca">
            <button
              ref={nome}
              className="item-nome"
              aria-label={`Renomear ${controle.nome}`}
              onClick={aoEditar}
            >
              <span className="item-nome-texto">{controle.nome}</span>
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
            {principal && <span className="item-selo">na bandeja</span>}
          </span>
        )}
        <div className="item-estado">
          {confirmando
            ? "Some da lista e leva o histórico e as sessões junto."
            : controle.via === "Desligado"
              ? controle.preenchimento !== null
                ? `desconectado · ${controle.textoDaCarga} na última leitura`
                : "desconectado"
              : controle.conectadoSemCarga
                ? controle.textoDaLigacao
                : `${controle.textoDaCarga} · ${controle.textoDaLigacao}`}
        </div>
      </div>
      {removivel &&
        (confirmando ? (
          <div
            className="confirmar"
            onKeyDown={(e) => {
              if (e.key === "Escape") cancelar();
            }}
            onBlur={(e) => {
              if (!e.currentTarget.contains(e.relatedTarget as Node | null)) cancelar(false);
            }}
          >
            <button
              ref={confirmar}
              className="botao perigo miudo"
              onClick={() => {
                aoEsquecer();
                void invoke("esquecer_controle", { chave: controle.chave });
              }}
            >
              Esquecer
            </button>
            <button className="botao miudo" onClick={() => cancelar()}>
              Cancelar
            </button>
          </div>
        ) : (
          <button
            ref={esquecer}
            className="esquecer"
            aria-label="Esquecer este controle"
            onClick={() => {
              setConfirmando(true);
              requestAnimationFrame(() => confirmar.current?.focus());
            }}
          >
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <path
                d="M2 3.2h8M4.6 3.2V2h2.8v1.2M3.2 3.2l.5 6.8h4.6l.5-6.8"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        ))}
    </div>
  );
}
