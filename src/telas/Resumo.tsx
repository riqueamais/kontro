import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { Anel } from "../componentes/Anel";
import { Glifo } from "../componentes/Glifo";
import { Historico } from "../componentes/Historico";
import { ListaDeControles } from "../componentes/ListaDeControles";
import { Saude } from "../componentes/Saude";
import type { Saude as DadosDeSaude } from "../componentes/Saude";
import { Sessoes, rotuloDaSessao } from "../componentes/Sessoes";
import { Amostra, Estado, Sessao, corDoAnel, quandoLeu, useEstado } from "../estado";

export function Resumo() {
  const estado = useEstado();
  const [serie, setSerie] = useState<Amostra[]>([]);
  const [sessoes, setSessoes] = useState<Sessao[]>([]);
  const [saude, setSaude] = useState<DadosDeSaude | null>(null);
  const [sessao, setSessao] = useState<Sessao | null>(null);

  useEffect(() => {
    invoke<Amostra[]>("serie_do_historico").then(setSerie).catch(() => {});
    invoke<Sessao[]>("sessoes_do_controle").then(setSessoes).catch(() => {});
    invoke<DadosDeSaude | null>("saude_da_bateria").then(setSaude).catch(() => {});
  }, [estado?.key, estado?.percent]);

  if (!estado) return null;

  return (
    <>
      <h1 className="titulo-da-pagina">Resumo</h1>

      <section className="cartao estado">
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado)}
          espessura={38}
          tamanho={96}
          girando={estado.girando}
        >
          {estado.temNumero ? (
            <span className="numero grande">{estado.percent}%</span>
          ) : (
            <Glifo tamanho={36} cor="var(--text-secondary)" />
          )}
        </Anel>

        <div className="leitura">
          <div className="dispositivo">
            {estado.mode === "Offline" ? "Desconectado" : estado.deviceName}
          </div>
          <div className="detalhe">{detalhe(estado)}</div>
          <div className="rodape">{quandoLeu(estado)}</div>
        </div>

        <button className="ciclo" onClick={() => invoke("ler_agora")}>
          Atualizar
        </button>
      </section>

      <section className="cartao">
        <Historico
          serie={serie}
          trocadaEm={saude?.trocadaEm}
          janela={
            sessao
              ? { inicio: sessao.inicio, fim: sessao.fim, titulo: rotuloDaSessao(sessao) }
              : null
          }
          aoSairDaJanela={() => setSessao(null)}
        />
        <Saude saude={saude} />
        <Sessoes
          sessoes={sessoes}
          escolhida={sessao?.inicio}
          aoEscolher={(s) => setSessao((atual) => (atual?.inicio === s.inicio ? null : s))}
        />
      </section>

      <ListaDeControles principal={estado.key} sempre />
    </>
  );
}

function detalhe(estado: Estado) {
  if (estado.mode === "Offline") return estado.deviceName;
  if (estado.conectadoSemCarga)
    return `conectado ${estado.textoDaLigacao} · não informa bateria`;
  if (estado.precisao === "Aproximada")
    return `${estado.textoDaCarga} · sem percentual neste controle`;
  return estado.autonomia ?? estado.textoDaLigacao;
}
