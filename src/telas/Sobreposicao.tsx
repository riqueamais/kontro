import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Estado, corDoAnel, useConfig, useControles, useEstado, useLimiares } from "../estado";
import "./sobreposicao.css";

export function Sobreposicao() {
  const estado = useEstado();
  const todos = useControles();
  const cfg = useConfig();
  const limiares = useLimiares();
  if (!estado) return null;
  const escala = cfg?.OverlayScale ?? 1;
  const opacidade = cfg?.OverlayOpacity ?? 0.9;
  const acompanhantes = todos.filter((c) => c.mode !== "Offline" && c.key !== estado.key);
  return (
    <div
      className="sobreposicao"
      style={{ transform: `scale(${escala})`, transformOrigin: "top left" }}
    >
      <div className="pilula" style={{ opacity: opacidade }}>
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado, limiares)}
          espessura={60}
          tamanho={30}
          girando={estado.girando}
        >
          <Glifo tamanho={15} cor="var(--text-primary)" />
        </Anel>
        <span className="valor">{resumir(estado)}</span>
        {acompanhantes.length > 0 && (
          <div className="acompanhantes">
            {acompanhantes.map((c) => (
              <div className="acompanhante" key={c.key} title={c.deviceName}>
                <Anel
                  valor={c.preenchimento}
                  cor={corDoAnel(c, limiares)}
                  espessura={70}
                  tamanho={20}
                  girando={c.girando}
                >
                  <Glifo tamanho={10} cor="var(--text-secondary)" />
                </Anel>
                <span className="valor menor">{resumir(c)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
/// O que escrever ao lado do anel, no menor numero de caracteres que ainda diz algo.
function resumir(estado: Estado): string {
  if (estado.girando) return "cabo";
  if (estado.precisao === "Aproximada" && estado.nivel !== null) {
    return ["baixa", "baixa", "média", "cheia"][Math.min(Math.max(estado.nivel, 0), 3)];
  }
  return estado.textoDaCarga;
}
