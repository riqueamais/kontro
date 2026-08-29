export const CAIXA = 512;
export const PAD_CENTRO_Y = 288;
export const GLIFO_CAIXA = "50 158 412 260";

export const PAD = "M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 372C146 400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z";

export const PAD_COM_STICKS_VAZADOS = "M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 366 400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 372C146 400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z M81 228 a48.3 48.3 0 1 0 96.6 0 a48.3 48.3 0 1 0 -96.6 0 Z M334.40002 228 a48.3 48.3 0 1 0 96.6 0 a48.3 48.3 0 1 0 -96.6 0 Z";

export const CORES = {
  fundo: "#0F1318",
  verde: "#5FE083",
  teal: "#35D7A8",
  branco: "#FFFFFF",
  glifoClaro: "#F4F7F9",
} as const;

export const APP = {
  anelRaio: 202,
  anelLargura: 30,
  anelVarredura: 259.2,
  padEscala: 0.6,
  padCentroY: 274,
  stickRaio: 29,
  stickEsq: [180, 238],
  stickDir: [332, 238],
  trilhoOpacidade: 0.13,
} as const;

export const BANDEJA = {
  anelRaio: 194,
  anelLargura: 56,
  padEscala: 0.5,
  padCentroY: 268,
  trilhoOpacidade: 0.22,
} as const;

export const GRADIENTE = { x1: 120, y1: 80, x2: 400, y2: 440 } as const;
