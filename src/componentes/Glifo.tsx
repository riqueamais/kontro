import { GLIFO_CAIXA, PAD_COM_STICKS_VAZADOS } from "../estilo/geometria.gerada";

export function Glifo({ tamanho, cor }: { tamanho: number; cor: string }) {
  return (
    <svg
      width={tamanho}
      height={tamanho}
      viewBox={GLIFO_CAIXA}
      aria-hidden="true"
      style={{ display: "block" }}
    >
      <path d={PAD_COM_STICKS_VAZADOS} fill={cor} fillRule="evenodd" />
    </svg>
  );
}
