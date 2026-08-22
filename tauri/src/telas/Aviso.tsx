import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { corDoAnel, useEstado } from "../estado";
import "./aviso.css";

/**
 * O aviso que aparece no topo quando algo muda na ligacao.
 *
 * Ele nasce pronto: quem decide a hora de mostrar e o lado do Rust, que espera a carga
 * existir antes de pedir. Aparecer dizendo "lendo" e trocar o texto no meio era pior
 * que aparecer um segundo depois.
 */
export function Aviso() {
  const estado = useEstado();
  if (!estado) return null;

  return (
    <div className="aviso">
      <div className="cartao">
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado)}
          espessura={46}
          tamanho={44}
          girando={estado.girando}
        >
          {estado.temNumero ? (
            <span className="pct">{estado.percent}%</span>
          ) : (
            <Glifo tamanho={18} cor="var(--text-secondary)" />
          )}
        </Anel>

        <div className="texto">
          <div className="nome">{estado.deviceName}</div>
          <div className="linha">
            {estado.mode === "Offline"
              ? estado.preenchimento !== null
                ? `desconectado - ${estado.textoDaCarga} na ultima leitura`
                : "desconectado"
              : `conectado - ${estado.textoDaLigacao}`}
          </div>
        </div>
      </div>
    </div>
  );
}
