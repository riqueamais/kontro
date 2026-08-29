# Kontro · sistema de design "Anel"

Um app de bandeja é lido em 16px e em meio segundo. Todo este sistema existe para
que a mesma forma — controle dentro de um anel de carga — funcione de 16px a 512px,
sobre barra clara ou escura, sem virar mancha.

## 1. Marca

A forma é um **anel de progresso** ao redor de um **controle sólido**. O anel é o dado
(carga); o controle é a identidade. Nada mais entra no ícone: sem número, sem raio,
sem sombra interna.

Geometria canônica, viewBox `0 0 512 512`:

| elemento        | valor |
| --------------- | ----- |
| Fundo (app)     | círculo r=256, `#0F1318` |
| Borda do fundo  | círculo r=252, stroke `#FFFFFF` 10%, largura 8 |
| Trilha do anel  | círculo r=202, stroke 13% do glifo, largura 30 |
| Anel de carga   | círculo r=202, largura 30, `stroke-linecap: round`, início em -90° (12h), sentido horário |
| Controle        | path abaixo, `scale(0.6)` centrado em (256, 274) |
| Sticks          | círculos r=29 em (180,238) e (332,238), pintados na cor do fundo |

Path do controle (use direto em `Geometry.Parse`):

    M168 158H344C392 158 424 186 436 232L462 330C476 382 452 418 414 418C386 418 366
    400 352 372L330 328C322 312 308 304 292 304H220C204 304 190 312 182 328L160 372C146
    400 126 418 98 418C60 418 36 382 50 330L76 232C88 186 120 158 168 158Z

## 2. Cor

| token | hex | uso |
| ----- | --- | --- |
| Ink | `#0B0E11` | fundo da janela, splash |
| Surface | `#0F1318` | fundo do ícone, cartões |
| SurfaceAlt | `#151A20` | linhas de lista, hover |
| Stroke | `#1E252C` | bordas de cartão |
| StrokeStrong | `#2A333B` | bordas de controle focado |
| TextPrimary | `#E8ECEF` | título, valor |
| TextSecondary | `#9AA4AD` | descrição |
| TextTertiary | `#6B757D` | rótulo, timestamp |
| AccentGreen | `#5FE083` | carga saudável, ação primária |
| AccentTeal | `#35D7A8` | fim do gradiente do anel |
| Amber | `#F2C14E` | bateria baixa |
| Red | `#F2564E` | bateria crítica |
| Gray | `#8D979F` | no cabo / sem leitura |

Gradiente do anel no ícone do app: `#5FE083` -> `#35D7A8`, diagonal (120,80) -> (400,440).
Na bandeja **não** use gradiente: cor plana, porque em 16px o gradiente vira lama.

Limiares de cor do anel: **são os mesmos que o usuário configurou para os avisos**, e
não uma segunda escala paralela. O anel fica âmbar exatamente quando o app diz "carga
baixa", e vermelho exatamente quando ele diz "carga crítica".

    carga >= WarnThreshold          AccentGreen
    CriticalThreshold <= carga < W  Amber
    carga < CriticalThreshold       Red

Com o padrão (avisar em 20%, crítico em 10%) o anel fica verde até 20%. Quem quiser o
verde só acima de 60% sobe o limiar de aviso — a cor acompanha. Em Rust isso é
`Settings::limiares()`; no front, `useLimiares()`. Nenhum dos dois crava número.

## 3. Ícones da bandeja

Mesma marca, sem o fundo circular — a bandeja é o fundo.

| ajuste | valor |
| ------ | ----- |
| Anel | r=194, largura 56 (bem mais grosso que no app: precisa existir em 16px) |
| Trilha | mesma cor do glifo a 22% |
| Controle | `scale(0.5)` centrado em (256, 268), sticks vazados |
| Glifo em barra escura | `#FFFFFF` |
| Glifo em barra clara | `#1B1F24` |

Estados:

- **level-N** — trilha + arco na cor do limiar. É o estado normal.
- **cable** — anel inteiro em `#8D979F`, sem arco de carga. Diz "estou plugado e não
  tenho número", que é a posição do app: melhor nada que errado.
- **off** — sem anel; controle a 45% de opacidade e uma barra diagonal
  (`M120 392 L392 120`, largura 46, ponta redonda) por cima.

Escolha do tema pela chave do registro
`HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\SystemUsesLightTheme`
(1 = barra clara -> use `tray/light`). Reaja a `WM_SETTINGCHANGE` e a
`SystemEvents.UserPreferenceChanged`.

DPI: carregue o PNG do tamanho igual a `GetSystemMetrics(SM_CXSMICON)` — 16 a 100%,
20 a 125%, 24 a 150%, 32 a 200%. Nunca escale um 16 para 32.

## 4. Tipografia

`Segoe UI Variable Display` para títulos, `Segoe UI Variable Text` para corpo, com
fallback `Segoe UI`. `Consolas` só para número medido e timestamp.

| estilo | tamanho / peso | uso |
| ------ | -------------- | --- |
| Display | 34 / SemiBold, tracking -0.5 | porcentagem no flyout |
| Title | 22 / SemiBold | título de janela |
| Subtitle | 16 / Regular | nome do controle |
| Body | 14 / Regular | texto de configuração |
| Caption | 12 / Regular, TextTertiary | rótulo, "há 4 min" |
| Mono | 12 / Regular, Consolas | mAh, horários, diagnóstico |

## 5. Espaço, raio, elevação

Escala de espaço: 4, 8, 12, 16, 24, 32. Nada fora dela.
Raios: 8 (campo, botão), 12 (cartão), 16 (janela/flyout), pill (anel de status).
Elevação: apenas duas. Cartão = sem sombra, borda `Stroke`. Flyout = sombra
`0 18 40 rgba(0,0,0,0.55)`, blur 32, opacidade 0.55.

## 6. Movimento

120ms para hover e pressed, 180ms para troca de estado do anel, 240ms para abrir o
flyout. Easing `CubicEase EaseOut` sempre. O anel **anima** entre percentuais em
180ms — nunca salta. Nenhuma animação em loop: um app de bandeja que pisca é um
app que o usuário desliga.

## 7. Telas

**Flyout (clique no ícone)** — 320x220, raio 16, fundo Ink, borda Stroke.
Ancorado ao canto da bandeja com 12px de folga. Conteúdo, de cima para baixo:
anel 96px com a porcentagem em Display no centro · nome do controle em Subtitle ·
estimativa de tempo restante em Body · última leitura em Caption/Mono ·
linha divisória · dois botões ghost: "Configurações" e "Atualizar".

Sem número quando estiver no cabo: mostre "No cabo" em Subtitle e a última leitura
com horário em Caption. Nunca invente percentual.

**Janela principal** — 480 de largura, cabeçalho de 56px com o ícone 20px + "Kontro"
em Title, corpo em cartões de raio 12 e 16px de padding: *Estado atual*,
*Histórico* (sparkline de 1px em AccentGreen 40%, sem eixos, sem grid),
*Configurações* em linhas de 44px de altura mínima com rótulo à esquerda e
controle à direita.

**Menu da bandeja** — largura mínima 200, itens de 32px, padding 12/8, fundo
SurfaceAlt, borda Stroke, raio 8. Primeiro item é o estado (desabilitado, serve de
cabeçalho), depois separador, depois as ações.

**Notificação de bateria baixa** — toast nativo do Windows, ícone `level-25` ou
`level-10`, uma linha de título e uma de corpo, sem botão. Não repita o mesmo
limiar duas vezes na mesma sessão de carga.

## 8. Acessibilidade

Contraste mínimo 4.5:1 para texto. AccentGreen sobre Ink passa (>=7:1); AccentGreen
como *texto* sobre Surface só acima de 14px SemiBold. Todo estado do ícone tem
tooltip com texto explícito — quem não distingue verde de âmbar lê o número.
