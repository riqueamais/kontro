const NOME_DA_TECLA: Record<string, string> = {
  Ctrl: "Ctrl",
  Alt: "Alt",
  Shift: "Shift",
  Super: "Win",
  Space: "Espaço",
  Escape: "Esc",
  Delete: "Del",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backslash: "\\",
  PageUp: "Page Up",
  PageDown: "Page Down",
};

export function nomeDaTecla(parte: string): string {
  if (NOME_DA_TECLA[parte]) return NOME_DA_TECLA[parte];
  if (parte.startsWith("Key")) return parte.slice(3);
  if (parte.startsWith("Digit")) return parte.slice(5);
  if (parte.startsWith("Numpad")) return `Num ${parte.slice(6)}`;
  return parte;
}

export function Teclas({ combinacao }: { combinacao: string }) {
  return (
    <>
      {combinacao.split("+").map((parte, i) => (
        <kbd className="tecla" key={i}>
          {nomeDaTecla(parte)}
        </kbd>
      ))}
    </>
  );
}
