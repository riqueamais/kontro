import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { corDoAnel, useEstado } from "../estado";
import "./sobreposicao.css";

/**
 * A pilula fixa na tela, no estilo de um contador de quadros.
 *
 * Ela e puramente visual: a janela nao recebe clique e nunca toma o foco. Um aviso que
 * rouba o foco no meio de uma partida seria pior que nao avisar nada.
 */
export function Sobreposicao() {
  const estado = useEstado();
  if (!estado) return null;

  const texto = estado.girando
    ? "cabo"
    : estado.precisao === "Aproximada" && estado.nivel !== null
      ? ["baixa", "baixa", "media", "cheia"][Math.min(Math.max(estado.nivel, 0), 3)]
      : estado.textoDaCarga;

  return (
    <div className="sobreposicao">
      <div className="pilula">
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado)}
          espessura={60}
          tamanho={30}
          girando={estado.girando}
        >
          <Glifo tamanho={15} cor="var(--text-primary)" />
        </Anel>
        <span className="valor">{texto}</span>
      </div>
    </div>
  );
}
