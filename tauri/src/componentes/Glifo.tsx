const PAD =
  "M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 " +
  "366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 " +
  "372C146 400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z";

const RAIO = 48.3;
const ESQ = { x: 129.3, y: 228 };
const DIR = { x: 382.7, y: 228 };

// Circulo escrito como subcaminho para viver dentro do mesmo path do controle: e o
// que permite vazar os analogicos com regra par-impar em vez de pintar por cima.
const circulo = (c: { x: number; y: number }) =>
  `M${c.x - RAIO} ${c.y} a${RAIO} ${RAIO} 0 1 0 ${RAIO * 2} 0 a${RAIO} ${RAIO} 0 1 0 -${RAIO * 2} 0 Z`;

const CAMINHO = `${PAD} ${circulo(ESQ)} ${circulo(DIR)}`;

/**
 * A silhueta do controle com os analogicos vazados.
 *
 * Sem os recortes ela vira uma mancha que so lembra a marca; com eles, se le como
 * controle mesmo em 16 pixels. E a mesma geometria do icone da bandeja.
 */
export function Glifo({ tamanho, cor }: { tamanho: number; cor: string }) {
  return (
    <svg
      width={tamanho}
      height={tamanho}
      viewBox="50 158 412 260"
      aria-hidden="true"
      style={{ display: "block" }}
    >
      <path d={CAMINHO} fill={cor} fillRule="evenodd" />
    </svg>
  );
}
