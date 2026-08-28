import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import "./saude.css";

export interface Saude {
  estado: "medindo" | "estavel" | "piorando" | "melhorando";
  dias: number;
  consumoRecente: number | null;
  consumoAntes: number | null;
  variacao: number | null;
  trocadaEm: number | null;
}

interface SaudeCrua {
  estado: Saude["estado"];
  dias: number;
  consumo_recente: number | null;
  consumo_antes: number | null;
  variacao: number | null;
  trocada_em: number | null;
}

const DIAS_PARA_COMPARAR = 14;

export function Saude({ chave, pulso }: { chave: string; pulso?: number | null }) {
  const [saude, setSaude] = useState<Saude | null>(null);

  useEffect(() => {
    invoke<SaudeCrua | null>("saude_da_bateria")
      .then((c) =>
        setSaude(
          c && {
            estado: c.estado,
            dias: c.dias,
            consumoRecente: c.consumo_recente,
            consumoAntes: c.consumo_antes,
            variacao: c.variacao,
            trocadaEm: c.trocada_em,
          },
        ),
      )
      .catch(() => {});
  }, [chave, pulso]);

  if (!saude) return null;

  return (
    <div className={`saude ${saude.estado}`}>
      <div className="veredito">{titulo(saude)}</div>
      <div className="explica">{detalhe(saude)}</div>
      {saude.estado === "medindo" && (
        <div className="trilho" aria-hidden="true">
          <span
            style={{
              width: `${Math.min(100, (saude.dias / DIAS_PARA_COMPARAR) * 100)}%`,
            }}
          />
        </div>
      )}
    </div>
  );
}

function titulo(s: Saude): string {
  switch (s.estado) {
    case "piorando":
      return `Durando ${duracao(s)}% menos`;
    case "melhorando":
      return `Durando ${duracao(s)}% mais`;
    case "estavel":
      return "Consumo estável";
    default:
      return s.trocadaEm ? "Bateria nova" : "Ainda medindo";
  }
}

function duracao(s: Saude): number {
  const recente = s.consumoRecente ?? 0;
  const antes = s.consumoAntes ?? 0;
  if (recente <= 0 || antes <= 0) return 0;
  const fator = antes / recente;
  return Math.round(Math.abs(fator - 1) * 100);
}

function detalhe(s: Saude): string {
  if (s.estado === "medindo") {
    const faltam = Math.max(0, DIAS_PARA_COMPARAR - s.dias);
    const desde = s.trocadaEm
      ? `Contando desde a troca, em ${new Date(s.trocadaEm).toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" })}. `
      : "";

    if (s.dias === 0) return `${desde}Preciso de duas semanas de uso para comparar.`;
    if (faltam > 0) {
      return `${desde}${s.dias} ${s.dias === 1 ? "dia" : "dias"} de histórico. Faltam ${faltam} para eu poder comparar.`;
    }
    return `${desde}Ainda não houve descarga suficiente nas duas janelas para comparar.`;
  }

  const recente = (s.consumoRecente ?? 0).toFixed(1).replace(".", ",");
  const antes = (s.consumoAntes ?? 0).toFixed(1).replace(".", ",");
  const base = `${recente}% por hora esta semana, contra ${antes}% antes.`;

  if (s.estado === "estavel") return `${base} Nada mudou.`;
  if (s.estado === "melhorando") return `${base} Você deve estar usando menos.`;
  return `${base} Pode ser uso mais pesado, ou bateria velha.`;
}
