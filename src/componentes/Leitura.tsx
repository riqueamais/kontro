import type { Estado } from "../estado";
import { detalhe, quandoLeu } from "../formato";
import "./leitura.css";

export function Leitura({ estado }: { estado: Estado }) {
  const { texto, hora } = quandoLeu(estado);
  const antiga = estado.leituraAntiga && estado.lidoEm !== null && !estado.procurando;
  return (
    <div className={antiga ? "leitura antiga" : "leitura"}>
      <div className="dispositivo">{estado.titulo}</div>
      <div className="detalhe">{detalhe(estado) || "\u00a0"}</div>
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
