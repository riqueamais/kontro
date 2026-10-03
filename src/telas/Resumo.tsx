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
import { Amostra, Sessao, corDoAnel, useAoMudarOHistorico, useEstado, useLimiares } from "../estado";

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
  }, [estado?.chave]);

  if (!estado) return null;

  return (
    <>
      <h1 className="titulo-da-pagina">Resumo</h1>

      <section
        className={estado.leituraAntiga ? "cartao estado antiga" : "cartao estado"}
        style={{ "--cor-do-estado": corDoAnel(estado, limiares) } as React.CSSProperties}
      >
        <Anel
          valor={estado.preenchimento}
          cor={corDoAnel(estado, limiares)}
          espessura={30}
          tamanho={96}
          girando={estado.girando}
          marcas={estado.temNumero ? limiares : null}
        >
          {estado.temNumero ? (
            <span className="numero grande">{estado.percentual}%</span>
          ) : (
            <Glifo tamanho={36} cor="var(--text-secondary)" />
          )}
        </Anel>

        <Leitura estado={estado} />

        <BotaoLerAgora estado={estado} className="botao" />
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

      <ListaDeControles principal={estado.chave} sempre />
    </>
  );
}
