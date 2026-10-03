import type { Estado } from "../estado";
import { detalhe, quandoLeu } from "../formato";
import "./leitura.css";

export function Leitura({ estado }: { estado: Estado }) {
  return (
    <div className="leitura">
      <div className="dispositivo">{estado.via === "Desligado" ? "Desconectado" : estado.nome}</div>
      <div className="detalhe">{detalhe(estado)}</div>
      <div className="rodape">{quandoLeu(estado)}</div>
    </div>
  );
}
