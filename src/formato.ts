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

export function dia(ms: number): string {
  const quando = new Date(ms);
  switch (diasAtras(quando)) {
    case 0:
      return "hoje";
    case 1:
      return "ontem";
    case -1:
      return "amanhã";
    default:
      return diaEMes(quando);
  }
}

export function quando(ms: number): string {
  return `${dia(ms)} às ${hora(new Date(ms))}`;
}

export function decimal(n: number): string {
  return n.toFixed(1).replace(".", ",");
}

export function taxa(porHora: number): string {
  return `${decimal(porHora)} %/h`;
}

export function duracao(minutos: number): string {
  if (minutos < 60) return `${minutos} min`;

  const h = Math.floor(minutos / 60);
  const m = minutos % 60;
  return m > 0 ? `${h} h ${m} min` : `${h} h`;
}

export interface Rodape {
  texto: string;
  hora: string | null;
}

export function quandoLeu(estado: Estado): Rodape {
  if (estado.procurando) return { texto: "", hora: null };
  if (!estado.lidoEm) return { texto: "sem leitura ainda", hora: null };

  const lido = new Date(estado.lidoEm);
  const abertura = estado.leituraAntiga ? "última leitura" : "atualizado";
  const dias = diasAtras(lido);
  const ponte = dias === 0 ? "às" : dias === 1 ? "ontem às" : `em ${diaEMes(lido)}, às`;
  return { texto: `${abertura} ${ponte}`, hora: hora(lido) };
}

export function detalhe(estado: Estado, comAutonomia = true): string {
  if (estado.procurando) return "";
  if (estado.via === "Desligado") {
    return estado.quantosConhecidos === 0
      ? "Ligue um controle por Bluetooth ou pelo cabo"
      : estado.nome;
  }
  if (estado.conectadoSemCarga) {
    return `conectado ${estado.textoDaLigacao} · não informa bateria`;
  }
  if (estado.precisao === "Aproximada") {
    return `${estado.textoDaCarga} · sem percentual neste controle`;
  }
  return (comAutonomia && estado.autonomia) || estado.textoDaLigacao;
}
