import { createContext, useContext, useEffect, useId, useRef, useState } from "react";

interface DaLinha {
  titulo: string;
  descricao: string;
  desabilitada: boolean;
}

export type Texto = React.ReactNode;

const ContextoDaLinha = createContext<DaLinha | null>(null);

export function useLinha(): DaLinha | null {
  return useContext(ContextoDaLinha);
}

export function Linha({
  titulo,
  descricao,
  erro,
  classe,
  desabilitada = false,
  viva = false,
  children,
}: {
  titulo: string;
  descricao: Texto;
  erro?: boolean;
  classe?: string;
  desabilitada?: boolean;
  viva?: boolean;
  children: React.ReactNode;
}) {
  const id = useId();
  const ids = { titulo: `${id}-t`, descricao: `${id}-d`, desabilitada };
  return (
    <ContextoDaLinha.Provider value={ids}>
      <div
        className={["linha", classe, erro && "erro", desabilitada && "desabilitada"]
          .filter(Boolean)
          .join(" ")}
      >
        <div className="rotulo">
          <div className="titulo" id={ids.titulo}>
            {titulo}
          </div>
          <div
            className="descricao"
            id={ids.descricao}
            role={viva ? "status" : undefined}
            aria-live={viva ? "polite" : undefined}
          >
            {descricao}
          </div>
        </div>
        <div className="controle">{children}</div>
      </div>
    </ContextoDaLinha.Provider>
  );
}

export function useNomeDoControle(proprio?: string) {
  const linha = useLinha();
  if (!linha) return {};
  return {
    "aria-labelledby": proprio ? `${linha.titulo} ${proprio}` : linha.titulo,
    "aria-describedby": linha.descricao,
  };
}

export function Chave({ ligado, aoTrocar }: { ligado: boolean; aoTrocar: (v: boolean) => void }) {
  const linha = useLinha();
  const nome = useNomeDoControle();
  return (
    <>
      <span className="estado-da-chave" aria-hidden="true">
        {ligado ? "Ativado" : "Desativado"}
      </span>
      <button
        role="switch"
        aria-checked={ligado}
        className={`chave${ligado ? " ligada" : ""}`}
        disabled={linha?.desabilitada}
        onClick={() => aoTrocar(!ligado)}
        {...nome}
      >
        <span className="bolinha" />
      </button>
    </>
  );
}

export function Deslizante({
  min,
  max,
  passo,
  valor,
  aoMudar,
  rotulo = (v) => `${v}%`,
}: {
  min: number;
  max: number;
  passo: number;
  valor: number;
  aoMudar: (valor: number) => void;
  rotulo?: (valor: number) => string;
}) {
  const linha = useLinha();
  const id = useId();
  const nome = useNomeDoControle(`${id}-valor`);
  const [local, setLocal] = useState(valor);
  const ultimo = useRef(valor);

  useEffect(() => {
    setLocal(valor);
    ultimo.current = valor;
  }, [valor]);

  const confirmar = () => {
    if (local === ultimo.current) return;
    ultimo.current = local;
    aoMudar(local);
  };

  const preenchido = max > min ? ((local - min) / (max - min)) * 100 : 0;

  return (
    <span className="deslizante">
      <input
        type="range"
        min={min}
        max={max}
        step={passo}
        value={local}
        disabled={linha?.desabilitada || max <= min}
        style={{ "--preenchido": `${preenchido}%` } as React.CSSProperties}
        onChange={(e) => setLocal(Number(e.target.value))}
        onPointerUp={confirmar}
        onKeyUp={confirmar}
        onBlur={confirmar}
        {...nome}
      />
      <span className="deslizante-valor" id={`${id}-valor`}>
        {rotulo(local)}
      </span>
    </span>
  );
}

export interface Opcao<T> {
  valor: T;
  rotulo: string;
}

export function Seletor<T extends string | number>({
  opcoes,
  valor,
  aoEscolher,
}: {
  opcoes: Opcao<T>[];
  valor: T;
  aoEscolher: (valor: T) => void;
}) {
  const linha = useLinha();
  const id = useId();
  const nome = useNomeDoControle(`${id}-botao`);
  const [aberto, setAberto] = useState(false);
  const [marcado, setMarcado] = useState(0);
  const raiz = useRef<HTMLSpanElement>(null);
  const botao = useRef<HTMLButtonElement>(null);
  const atual = opcoes.find((o) => o.valor === valor);

  useEffect(() => {
    if (!aberto) return;
    setMarcado(Math.max(0, opcoes.findIndex((o) => o.valor === valor)));
    const fora = (e: PointerEvent) => {
      if (!raiz.current?.contains(e.target as Node)) setAberto(false);
    };
    document.addEventListener("pointerdown", fora, true);
    return () => document.removeEventListener("pointerdown", fora, true);
  }, [aberto, opcoes, valor]);

  const escolher = (opcao: Opcao<T>) => {
    setAberto(false);
    botao.current?.focus();
    if (opcao.valor !== valor) aoEscolher(opcao.valor);
  };

  const aoTeclar = (e: React.KeyboardEvent) => {
    if (!aberto) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        setAberto(true);
      }
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      setAberto(false);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setMarcado((i) => Math.min(opcoes.length - 1, i + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setMarcado((i) => Math.max(0, i - 1));
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      escolher(opcoes[marcado]);
    } else if (e.key === "Tab") {
      setAberto(false);
    }
  };

  return (
    <span className="seletor" ref={raiz} onKeyDown={aoTeclar}>
      <button
        ref={botao}
        id={`${id}-botao`}
        className="botao seletor-botao"
        aria-haspopup="listbox"
        aria-expanded={aberto}
        aria-controls={`${id}-lista`}
        disabled={linha?.desabilitada}
        onClick={() => setAberto(!aberto)}
        {...nome}
      >
        <span>{atual?.rotulo ?? ""}</span>
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M3 4.5 6 7.5 9 4.5"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.4"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>
      {aberto && (
        <ul
          className="seletor-lista"
          id={`${id}-lista`}
          role="listbox"
          aria-activedescendant={`${id}-op-${marcado}`}
        >
          {opcoes.map((o, i) => (
            <li
              key={String(o.valor)}
              id={`${id}-op-${i}`}
              role="option"
              aria-selected={o.valor === valor}
              className={i === marcado ? "marcada" : undefined}
              onPointerEnter={() => setMarcado(i)}
              onClick={() => escolher(o)}
            >
              {o.rotulo}
            </li>
          ))}
        </ul>
      )}
    </span>
  );
}

const LARGURA_DA_MINI_TELA = 48;
const ALTURA_DA_MINI_TELA = 28;
const LARGURA_DA_MINI_PILULA = 10;
const ALTURA_DA_MINI_PILULA = 4;

const PASSO_DA_POSICAO = 0.05;
const PASSO_FINO_DA_POSICAO = 0.01;

export function MiniTela({
  x,
  y,
  solta,
  escala = 1,
  escalaDaPilula = 1,
  opacidade = 1,
  aoMover,
}: {
  x: number;
  y: number;
  solta: boolean;
  escala?: number;
  escalaDaPilula?: number;
  opacidade?: number;
  aoMover?: (x: number, y: number) => void;
}) {
  const largura = LARGURA_DA_MINI_TELA * escala;
  const altura = ALTURA_DA_MINI_TELA * escala;
  const pilula = {
    largura: LARGURA_DA_MINI_PILULA * escala * escalaDaPilula,
    altura: ALTURA_DA_MINI_PILULA * escala * escalaDaPilula,
  };
  const classe = solta ? "mini-tela solta" : "mini-tela";
  const estilo = { width: largura, height: altura, borderRadius: "var(--radius-control)" };
  const miolo = (
    <span
      className="mini-pilula"
      style={{
        width: pilula.largura,
        height: pilula.altura,
        opacity: opacidade,
        left: 1 + x * Math.max(0, largura - 2 - pilula.largura),
        top: 1 + y * Math.max(0, altura - 2 - pilula.altura),
      }}
    />
  );

  if (!aoMover) {
    return (
      <span className={classe} style={estilo} aria-hidden="true">
        {miolo}
      </span>
    );
  }

  const limitar = (v: number) => Math.round(Math.min(1, Math.max(0, v)) * 100) / 100;
  const aoTeclar = (e: React.KeyboardEvent) => {
    const passo = e.shiftKey ? PASSO_FINO_DA_POSICAO : PASSO_DA_POSICAO;
    const destino: Record<string, [number, number]> = {
      ArrowLeft: [x - passo, y],
      ArrowRight: [x + passo, y],
      ArrowUp: [x, y - passo],
      ArrowDown: [x, y + passo],
      Home: [0, 0],
      End: [1, 1],
      PageUp: [1, 0],
      PageDown: [0, 1],
    };
    const alvo = destino[e.key];
    if (!alvo) return;
    e.preventDefault();
    aoMover(limitar(alvo[0]), limitar(alvo[1]));
  };

  return (
    <button
      type="button"
      className={classe}
      style={estilo}
      aria-label={`Posição da pílula: ${Math.round(x * 100)}% da largura, ${Math.round(y * 100)}% da altura`}
      onKeyDown={aoTeclar}
    >
      {miolo}
    </button>
  );
}
