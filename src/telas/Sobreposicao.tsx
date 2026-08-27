import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { corDoAnel, useConfig, useEstado } from "../estado";
import "./sobreposicao.css";

/**
 * A pilula fixa na tela, no estilo de um contador de quadros.
 *
 * Ela e puramente visual: a janela nao recebe clique e nunca toma o foco. Um aviso que
 * rouba o foco no meio de uma partida seria pior que nao avisar nada.
 */
export function Sobreposicao() {
  const estado = useEstado();
  const cfg = useConfig();
  if (!estado) return null;

  // O tamanho e a transparencia sao escolha do usuario: numa tela de 27" a pilula
  // padrao some, e numa partida com HUD carregado ela precisa sumir um pouco.
  const escala = cfg?.OverlayScale ?? 1;
  const opacidade = cfg?.OverlayOpacity ?? 0.9;

  const texto = estado.girando
    ? "cabo"
    : estado.precisao === "Aproximada" && estado.nivel !== null
      ? ["baixa", "baixa", "média", "cheia"][Math.min(Math.max(estado.nivel, 0), 3)]
      : estado.textoDaCarga;

  return (
    <div
      className="sobreposicao"
      style={{ transform: `scale(${escala})`, transformOrigin: "top left" }}
    >
      <div className="pilula" style={{ opacity: opacidade }}>
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
