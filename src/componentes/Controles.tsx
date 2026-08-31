export function Linha({
  titulo,
  descricao,
  children,
}: {
  titulo: string;
  descricao: string;
  children: React.ReactNode;
}) {
  return (
    <div className="linha">
      <div className="rotulo">
        <div className="titulo">{titulo}</div>
        <div className="descricao">{descricao}</div>
      </div>
      <div className="controle">{children}</div>
    </div>
  );
}

export function Chave({ ligado, aoTrocar }: { ligado: boolean; aoTrocar: (v: boolean) => void }) {
  return (
    <button
      role="switch"
      aria-checked={ligado}
      className={`chave${ligado ? " ligada" : ""}`}
      onClick={() => aoTrocar(!ligado)}
    >
      <span className="bolinha" />
    </button>
  );
}

const LARGURA_DA_MINI_TELA = 48;
const ALTURA_DA_MINI_TELA = 28;
const LARGURA_DA_MINI_PILULA = 10;
const ALTURA_DA_MINI_PILULA = 4;

export function MiniTela({
  x,
  y,
  solta,
  escala = 1,
}: {
  x: number;
  y: number;
  solta: boolean;
  escala?: number;
}) {
  const largura = LARGURA_DA_MINI_TELA * escala;
  const altura = ALTURA_DA_MINI_TELA * escala;
  const pilula = { largura: LARGURA_DA_MINI_PILULA * escala, altura: ALTURA_DA_MINI_PILULA * escala };

  return (
    <span
      className={solta ? "mini-tela solta" : "mini-tela"}
      style={{ width: largura, height: altura, borderRadius: 4 + escala }}
      aria-hidden="true"
    >
      <span
        className="mini-pilula"
        style={{
          width: pilula.largura,
          height: pilula.altura,
          left: 1 + x * (largura - 2 - pilula.largura),
          top: 1 + y * (altura - 2 - pilula.altura),
        }}
      />
    </span>
  );
}
