import {
  APP,
  BANDEJA,
  CAIXA,
  CORES,
  GRADIENTE,
  PAD,
  PAD_CENTRO_Y,
  PAD_COM_STICKS_VAZADOS,
} from "../estilo/geometria.gerada";

const CENTRO = CAIXA / 2;

export function Marca({ tamanho = 16 }: { tamanho?: number }) {
  const miudo = tamanho < 24;
  const perfil = miudo ? BANDEJA : APP;
  const volta = 2 * Math.PI * perfil.anelRaio;
  const cheio = (volta * APP.anelVarredura) / 360;
  const assento = `translate(${CENTRO} ${perfil.padCentroY}) scale(${perfil.padEscala}) translate(${-CENTRO} ${-PAD_CENTRO_Y})`;

  return (
    <svg
      width={tamanho}
      height={tamanho}
      viewBox={`0 0 ${CAIXA} ${CAIXA}`}
      aria-hidden="true"
      style={{ display: "block", flex: "0 0 auto" }}
    >
      <defs>
        <linearGradient
          id="marca-kontro"
          x1={GRADIENTE.x1}
          y1={GRADIENTE.y1}
          x2={GRADIENTE.x2}
          y2={GRADIENTE.y2}
          gradientUnits="userSpaceOnUse"
        >
          <stop offset="0" stopColor={CORES.verde} />
          <stop offset="1" stopColor={CORES.teal} />
        </linearGradient>
      </defs>

      <circle cx={CENTRO} cy={CENTRO} r={CENTRO} fill={CORES.fundo} />
      <circle
        cx={CENTRO}
        cy={CENTRO}
        r={perfil.anelRaio}
        fill="none"
        stroke={CORES.branco}
        strokeOpacity={perfil.trilhoOpacidade}
        strokeWidth={perfil.anelLargura}
      />
      <circle
        cx={CENTRO}
        cy={CENTRO}
        r={perfil.anelRaio}
        fill="none"
        stroke="url(#marca-kontro)"
        strokeWidth={perfil.anelLargura}
        strokeLinecap="round"
        strokeDasharray={`${cheio} ${volta - cheio}`}
        transform={`rotate(-90 ${CENTRO} ${CENTRO})`}
      />

      {miudo ? (
        <g transform={assento}>
          <path d={PAD_COM_STICKS_VAZADOS} fill={CORES.branco} fillRule="evenodd" />
        </g>
      ) : (
        <g transform={assento}>
          <path d={PAD} fill={CORES.glifoClaro} />
          <circle
            cx={APP.stickEsq[0]}
            cy={APP.stickEsq[1]}
            r={APP.stickRaio}
            fill={CORES.fundo}
          />
          <circle
            cx={APP.stickDir[0]}
            cy={APP.stickDir[1]}
            r={APP.stickRaio}
            fill={CORES.fundo}
          />
        </g>
      )}
    </svg>
  );
}
