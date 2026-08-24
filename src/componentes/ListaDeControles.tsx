import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { Estado, corDoAnel } from "../estado";
import { Anel } from "./Anel";
import { Glifo } from "./Glifo";
import "./lista.css";

/**
 * Os controles que o app conhece, um por linha.
 *
 * No painel ela aparece so quando ha mais de um: com um controle so ela repetiria, em
 * letra menor, o que ja esta escrito logo acima. Na janela do app ela aparece sempre --
 * la ela e o assunto, e nao um complemento.
 */
export function ListaDeControles({
  principal,
  sempre = false,
}: {
  principal: string;
  sempre?: boolean;
}) {
  const [controles, setControles] = useState<Estado[]>([]);
  const [editando, setEditando] = useState<string | null>(null);

  useEffect(() => {
    invoke<Estado[]>("controles").then(setControles).catch(() => {});
    const parar = listen<Estado[]>("kontro://controles", (e) => setControles(e.payload));
    return () => {
      void parar.then((f) => f());
    };
  }, []);

  if (controles.length < (sempre ? 1 : 2)) return null;

  return (
    <div className={`lista${sempre ? " solta" : ""}`}>
      <div className="lista-titulo">Seus controles</div>
      {controles.map((c) => (
        <Linha
          key={c.key}
          controle={c}
          principal={c.key === principal}
          podeEsquecer={sempre}
          editando={editando === c.key}
          aoEditar={() => setEditando(c.key)}
          aoSair={() => setEditando(null)}
        />
      ))}
    </div>
  );
}

function Linha({
  controle,
  principal,
  podeEsquecer,
  editando,
  aoEditar,
  aoSair,
}: {
  controle: Estado;
  principal: boolean;
  podeEsquecer: boolean;
  editando: boolean;
  aoEditar: () => void;
  aoSair: () => void;
}) {
  const campo = useRef<HTMLInputElement>(null);
  const [confirmando, setConfirmando] = useState(false);

  useEffect(() => {
    if (editando) {
      campo.current?.focus();
      campo.current?.select();
    }
  }, [editando]);

  const salvar = () => {
    const nome = campo.current?.value ?? "";
    // O painel se esconde quando perde o foco, e isso dispara o onBlur. Sem esta guarda,
    // sair do painel com o campo aberto gravaria um apelido que ninguem digitou.
    if (nome !== controle.deviceName) {
      void invoke("renomear_controle", { chave: controle.key, nome });
    }
    aoSair();
  };

  // Esquecer um controle que esta ligado nao adianta: a descoberta o encontra de novo no
  // ciclo seguinte, e ele reaparece como se nada tivesse acontecido.
  const removivel = podeEsquecer && controle.mode === "Offline";

  return (
    <div className={`item${principal ? " principal" : ""}`}>
      <Anel
        valor={controle.preenchimento}
        cor={corDoAnel(controle)}
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
            defaultValue={controle.deviceName}
            maxLength={40}
            placeholder="Nome do controle"
            onBlur={salvar}
            onKeyDown={(e) => {
              if (e.key === "Enter") salvar();
              // Esc desiste sem gravar; sem isso a unica saida seria salvar
              if (e.key === "Escape") aoSair();
            }}
          />
        ) : (
          <button className="item-nome" title="Renomear" onClick={aoEditar}>
            {controle.deviceName}
          </button>
        )}
        <div className="item-estado">
          {controle.mode === "Offline"
            ? controle.preenchimento !== null
              ? `desconectado · ${controle.textoDaCarga} na última leitura`
              : "desconectado"
            : `${controle.textoDaCarga} · ${controle.textoDaLigacao}`}
        </div>
      </div>

      {removivel &&
        (confirmando ? (
          <div className="confirmar">
            <button
              className="ciclo perigo"
              onClick={() => void invoke("esquecer_controle", { chave: controle.key })}
            >
              Esquecer
            </button>
            <button className="ciclo" onClick={() => setConfirmando(false)}>
              Cancelar
            </button>
          </div>
        ) : (
          <button
            className="esquecer"
            title="Esquecer este controle"
            aria-label="Esquecer este controle"
            onClick={() => setConfirmando(true)}
          >
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <path
                d="M1.5 1.5 L10.5 10.5 M10.5 1.5 L1.5 10.5"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
              />
            </svg>
          </button>
        ))}
    </div>
  );
}
