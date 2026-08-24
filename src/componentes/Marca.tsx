/**
 * A marca do app: o mesmo desenho do icone, em tamanho de interface.
 *
 * Repetir aqui o desenho que o Rust gera para o icone e proposital -- sao dois meios
 * diferentes, e o unico jeito de o app na barra de tarefas e o app na tela serem a mesma
 * coisa e as duas versoes sairem da mesma geometria. Os numeros vem de
 * `src-tauri/src/geometria.rs`; mexer la pede mexer aqui.
 */
export function Marca({ tamanho = 16 }: { tamanho?: number }) {
  return (
    <svg
      width={tamanho}
      height={tamanho}
      viewBox="0 0 512 512"
      aria-hidden="true"
      style={{ display: "block", flex: "0 0 auto" }}
    >
      <defs>
        <linearGradient id="marca-kontro" x1="120" y1="80" x2="400" y2="440" gradientUnits="userSpaceOnUse">
          <stop offset="0" stopColor="#5FE083" />
          <stop offset="1" stopColor="#35D7A8" />
        </linearGradient>
      </defs>
      <circle cx="256" cy="256" r="256" fill="#0F1318" />
      <circle
        cx="256"
        cy="256"
        r="202"
        fill="none"
        stroke="#FFFFFF"
        strokeOpacity="0.13"
        strokeWidth="30"
      />
      {/* 72% da volta, comecando no topo -- o mesmo arco gravado no icone */}
      <circle
        cx="256"
        cy="256"
        r="202"
        fill="none"
        stroke="url(#marca-kontro)"
        strokeWidth="30"
        strokeLinecap="round"
        strokeDasharray="913.8 355.4"
        transform="rotate(-90 256 256)"
      />
      <g transform="translate(256 274) scale(0.6) translate(-256 -288)">
        <path
          d="M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 372C146 400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z"
          fill="#F4F7F9"
        />
        <circle cx="180" cy="238" r="29" fill="#0F1318" />
        <circle cx="332" cy="238" r="29" fill="#0F1318" />
      </g>
    </svg>
  );
}
