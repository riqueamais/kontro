import { Estado } from "./estado";

export function hora(quando: Date): string {
  return quando.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });
}

export function diaEMes(quando: Date): string {
  return quando.toLocaleDateString("pt-BR", { day: "2-digit", month: "2-digit" });
}

export function diasAtras(quando: Date): number {
  const meiaNoite = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  return Math.round((meiaNoite(new Date()) - meiaNoite(quando)) / 86_400_000);
}

export function momento(ms: number): string {
  const quando = new Date(ms);
  return `${diaEMes(quando)} às ${hora(quando)}`;
}

export function momentoRelativo(ms: number): string {
  const quando = new Date(ms);
  const dias = diasAtras(quando);

  if (dias === 0) return `hoje, ${hora(quando)}`;
  if (dias === 1) return `ontem, ${hora(quando)}`;
  return `${diaEMes(quando)}, ${hora(quando)}`;
}

export function duracao(minutos: number): string {
  if (minutos < 60) return `${minutos} min`;

  const h = Math.floor(minutos / 60);
  const m = minutos % 60;
  return m > 0 ? `${h} h ${m} min` : `${h} h`;
}

export function quandoLeu(estado: Estado): string {
  if (!estado.readAt) return "sem leitura ainda";

  const quando = new Date(estado.readAt);
  const verbo = estado.stale ? "lido" : "atualizado";
  const dias = diasAtras(quando);

  if (dias === 0) return `${verbo} às ${hora(quando)}`;
  if (dias === 1) return `${verbo} ontem às ${hora(quando)}`;
  return `${verbo} em ${diaEMes(quando)}, às ${hora(quando)}`;
}

export function detalhe(estado: Estado): string {
  if (estado.mode === "Offline") return estado.deviceName;
  if (estado.conectadoSemCarga) {
    return `conectado ${estado.textoDaLigacao} · não informa bateria`;
  }
  if (estado.precisao === "Aproximada") {
    return `${estado.textoDaCarga} · sem percentual neste controle`;
  }
  return estado.autonomia ?? estado.textoDaLigacao;
}
