import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

import { Anel } from "../componentes/Anel";
import { BotaoLerAgora } from "../componentes/BotaoLerAgora";
import { Glifo } from "../componentes/Glifo";
import { Historico } from "../componentes/Historico";
import { Leitura } from "../componentes/Leitura";
import { ListaDeControles } from "../componentes/ListaDeControles";
import { Saude } from "../componentes/Saude";
import type { Saude as DadosDeSaude } from "../componentes/Saude";
import { Sessoes, rotuloDaSessao } from "../componentes/Sessoes";
import {
  Amostra,
  Estado,
  Sessao,
  corDoAnel,
  useAoMudarOHistorico,
  useEstado,
  useLimiares,
} from "../estado";
import { duracao, rotuloDoAnel, taxa } from "../formato";

interface DoControle {
  serie: Amostra[];
  sessoes: Sessao[];
  saude: DadosDeSaude | null;
}

const VAZIO: DoControle = { serie: [], sessoes: [], saude: null };

export function Resumo() {
  const estado = useEstado();
  const limiares = useLimiares();
  const [{ serie, sessoes, saude }, setDoControle] = useState<DoControle>(VAZIO);
  const [sessao, setSessao] = useState<Sessao | null>(null);

  useAoMudarOHistorico(() => {
    invoke<DoControle>("resumo_do_controle").then(setDoControle).catch(() => {});
  }, [estado?.chave, estado?.via, estado?.lidoEm]);

  if (!estado) return null;

  return (
    <>
      <h1 className="titulo-da-pagina">Resumo</h1>

      <section
        className={estado.leituraAntiga ? "cartao estado antiga" : "cartao estado"}
        aria-live="polite"
        style={{ "--cor-do-estado": corDoAnel(estado, limiares) } as React.CSSProperties}
      >
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado, limiares)}
          espessura={30}
          tamanho={96}
          girando={estado.girando}
          marcas={estado.temNumero ? limiares : null}
          rotulo={rotuloDoAnel(estado, limiares)}
        >
          {estado.temNumero ? (
            <span className="numero grande">{estado.percentual}%</span>
          ) : (
            <Glifo tamanho={36} cor="var(--text-secondary)" />
          )}
        </Anel>

        <div className="coluna">
          <Leitura
            estado={estado}
            comAutonomia={false}
            renomeavel={estado.quantosConhecidos < 2}
          />
          <Indicadores estado={estado} cargaCheiaMinutos={saude?.cargaCheiaMinutos ?? null} />
        </div>

        <div className="acoes-do-estado">
          <BotaoLerAgora estado={estado} className="botao" />
          {estado.quantosConhecidos === 1 && estado.via === "Desligado" && (
            <Esquecer chave={estado.chave} />
          )}
        </div>
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
          autonomiaMinutos={estado.autonomiaMinutos}
          aoSairDaJanela={() => setSessao(null)}
        />
        <Saude saude={saude} />
        <Sessoes
          sessoes={sessoes}
          escolhida={sessao?.inicio}
          aoEscolher={(s) => setSessao((atual) => (atual?.inicio === s.inicio ? null : s))}
        />
      </section>

      {estado.quantosConhecidos >= 2 && <ListaDeControles principal={estado.chave} sempre />}
    </>
  );
}

function Indicadores({
  estado,
  cargaCheiaMinutos,
}: {
  estado: Estado;
  cargaCheiaMinutos: number | null;
}) {
  const blocos = [
    estado.autonomiaMinutos !== null && {
      numero: `~${duracao(estado.autonomiaMinutos)}`,
      rotulo: "restam de jogo",
    },
    cargaCheiaMinutos !== null && { numero: duracao(cargaCheiaMinutos), rotulo: "por carga cheia" },
    estado.consumoPorHora !== null && { numero: taxa(estado.consumoPorHora), rotulo: "de consumo" },
  ].filter((b): b is { numero: string; rotulo: string } => !!b);

  if (blocos.length === 0) return null;
  return (
    <div className="indicadores">
      {blocos.map((b) => (
        <div className="indicador" key={b.rotulo}>
          <span className="indicador-numero">{b.numero}</span>
          <span className="indicador-rotulo">{b.rotulo}</span>
        </div>
      ))}
    </div>
  );
}

function Esquecer({ chave }: { chave: string }) {
  const [confirmando, setConfirmando] = useState(false);
  if (!confirmando) {
    return (
      <button className="botao fantasma" onClick={() => setConfirmando(true)}>
        Esquecer
      </button>
    );
  }
  return (
    <div className="confirmar">
      <button
        className="botao perigo miudo"
        onClick={() => void invoke("esquecer_controle", { chave })}
      >
        Esquecer
      </button>
      <button className="botao miudo" onClick={() => setConfirmando(false)}>
        Cancelar
      </button>
    </div>
  );
}
