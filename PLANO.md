# Plano · "Kontro que dá vontade de abrir"

O `TAREFAS.md` é o registro do que quebrou e por quê. Este arquivo é o oposto: o que ainda
não existe e vale construir, em ordem, com critério de pronto. Cada tarefa é pequena o
bastante para caber num commit e grande o bastante para dar para ver a diferença na tela.

Regra que atravessa tudo: **nada aqui pode inventar número**. Brilho, cor e animação
descrevem estado medido — quando não há medida, não há efeito.

## Situação

**As quinze tarefas foram entregues na 2.13.0.** O que fica de pé para conferir com uso
real, porque depende de dado que ainda não existe:

- **T5, a projeção até o zero.** Só desenha quando há consumo medido. Com o controle
  desligado não há linha nenhuma — que é a regra, não uma falha. Precisa de uma sessão
  para ser vista.
- **T6 a T10, a cadeia do jogo.** O código grava a partir desta versão; as sessões
  anteriores não têm nome. A primeira partida em tela cheia é o teste de verdade.
- **T12, o ranking.** Exige 3 sessões por jogo. Até lá mostra quantos estão em medição.

---

## Emenda ao DESIGN.md

Três tarefas abaixo colidem de propósito com o doc atual. A colisão é consciente e a regra
nova é esta:

- §6 diz "nenhuma animação em loop". Continua valendo para **movimento**. Não vale para
  **luz**: um halo que muda de cor quando a carga muda de faixa é estado desenhado, não
  animação. Segue proibido tudo que se mexe sozinho sem o dado ter mudado.
- §2 fixa a paleta. A cor de destaque do Windows entra como tema opcional, nunca
  substituindo os tokens de carga — verde, âmbar e vermelho são semântica, não decoração.
- §7 descreve o flyout como "fundo Ink". Passa a ser Acrylic; o Ink vira o fallback de
  quando o sistema não entrega o efeito.

---

# Movimento 1 — o app para de parecer uma página web escura

Hoje toda janela é um retângulo opaco com `border-radius`. O Windows 11 tem material de
verdade e o Tauri 2.11 expõe ele. Este é o movimento de maior retorno por linha escrita.

## T1. Material do sistema nas quatro janelas

**Onde:** `src-tauri/src/janelas.rs`, `src/telas/painel.css`, `principal.css`,
`sobreposicao.css`, `aviso.css`.

`WebviewWindowBuilder` tem `.effects(EffectsBuilder::new().effect(...).build())`, com
`Effect::MicaDark` e `Effect::Acrylic` disponíveis na versão que já está no `Cargo.lock`.

- Janela principal: `MicaDark`.
- Painel (flyout), sobreposição e aviso: `Acrylic` — as três já nascem
  `transparent(true)`, então falta só o efeito e tirar o fundo opaco do CSS.

**A pegadinha:** `.app` e `.painel` pintam `background: var(--ink)` chapado. Enquanto isso
existir, o material fica atrás de uma parede e não aparece. Trocar por uma camada
semitransparente e manter o `--ink` sólido como fallback.

**Pronto quando:** arrastar a janela por cima de um papel de parede colorido muda o tom do
fundo, e desligar transparência no Windows não deixa o app ilegível.

## T2. O anel ganha luz

**Onde:** `src/componentes/Anel.tsx`, `anel.css`, `src/estilo/tokens.css`.

O anel hoje é um `stroke` de cor plana. Três camadas novas, todas derivadas da cor que
`corDoAnel()` já devolve:

1. **Conic-gradient no arco** — do tom da faixa para uma versão 12% mais clara, na direção
   do preenchimento. Dá volume sem virar arco-íris.
2. **Halo** — `drop-shadow` na cor da faixa, raio 12, opacidade 0.35. É o que faz parecer
   que o anel emite luz em vez de estar pintado.
3. **Eco no cartão** — o `.cartao.estado` ganha um `radial-gradient` fantasma da cor do
   estado no canto do anel, a 6% de opacidade. Quando a carga cai para âmbar, o cartão
   inteiro esquenta um grau. Ninguém percebe conscientemente e todo mundo sente.

**Pronto quando:** a transição verde → âmbar → vermelho é visível pelo canto do olho, com a
mesma duração de 180ms que o arco já usa, e nada pisca em estado parado.

## T3. Marcas de limiar no anel e no gráfico

**Onde:** `Anel.tsx`, `src/componentes/Historico.tsx`, `historico.css`.

O `DESIGN.md` §2 é categórico: a cor segue o limiar configurado. Mas o usuário não vê
*onde* o limiar está — só descobre quando a cor muda.

- **No anel:** dois traços de 2px na trilha, nas posições de `WarnThreshold` e
  `CriticalThreshold`, na cor da faixa correspondente a 40%.
- **No gráfico:** duas faixas horizontais de fundo, âmbar e vermelha, a 8% de opacidade,
  abaixo de cada limiar. A linha entrando na faixa conta a história sozinha.

**Pronto quando:** mudar o limiar nas configurações move o traço e a faixa na hora, sem
recarregar.

## T4. Área com gradiente sob a curva

**Onde:** `Historico.tsx`, `historico.css`.

Cada trecho contínuo ganha um `path` fechado até a base, preenchido com um gradiente
vertical da cor da faixa a 22% até transparente. Custa um path por segmento e muda a
percepção de "diagrama" para "gráfico".

**Pronto quando:** o modo compacto do flyout continua legível em 70px de altura, sem a área
virar mancha.

## T5. Projeção até o zero

**Onde:** `Historico.tsx`, e `historico::autonomia_em_minutos` no Rust, que já existe.

A série termina no agora. A partir do último ponto, desenhar uma continuação **pontilhada**
descendo na taxa medida até cruzar 0%, com o horário do cruzamento em Caption/Mono na
ponta. É a mesma conta que já alimenta o texto de autonomia — só que vista.

**Regra dura:** sem taxa medida, sem linha. Nada de extrapolar de dois pontos. Se
`autonomia` é `None`, o gráfico termina onde sempre terminou.

**Pronto quando:** com autonomia disponível a linha aparece e a hora bate com o texto do
cartão de estado; sem ela, não há nenhum traço a mais.

---

# Movimento 2 — o app passa a saber o que você jogou

Aqui o produto muda de categoria. Deixa de ser um medidor de bateria e vira o diário do que
você jogou, com a bateria como unidade de medida. Todo o Movimento 3 depende deste.

## T6. Descobrir o processo em primeiro plano

**Onde:** `src-tauri/src/tela.rs`.

O módulo já chama `GetForegroundWindow` em `centro_da_janela_em_foco`. Falta encadear
`GetWindowThreadProcessId`, `OpenProcess` e `QueryFullProcessImageNameW`, e devolver o
caminho do exe com um nome legível.

O nome bonito sai do `FileDescription` do recurso de versão do exe ("ELDEN RING", não
"eldenring.exe"), com o nome do arquivo como fallback.

**Só conta como jogo** quando `em_tela_cheia()` já é verdade. Sem isso o app começa a
registrar o Explorer e o navegador, e a lista de sessões vira lixo.

**Ignorar sempre:** o próprio Kontro, `explorer.exe`, `ApplicationFrameHost.exe`,
`SearchHost.exe` e o shell. Lista curta e explícita, nunca heurística.

**Pronto quando:** um teste cobre a extração do nome a partir de um caminho, e rodar o app
com um jogo aberto registra o nome certo no diagnóstico.

## T7. A amostra guarda o jogo

**Onde:** `src-tauri/src/historico.rs`, `monitor.rs`.

`Amostra` hoje é `{ t, p, via }` — chaves curtas porque o arquivo é gravado a cada 30s.
Entra `j: Option<u16>`, um índice para uma tabela de jogos no topo do `history.json`.
Índice em vez de string repetida: 30 dias de amostras não podem carregar "ELDEN RING"
trinta mil vezes.

`#[serde(default)]` no campo novo — os `history.json` que já existem têm que continuar
abrindo.

**Pronto quando:** um `history.json` da versão anterior carrega sem erro e sem jogo, e um
novo grava o índice.

## T8. A sessão herda o jogo

**Onde:** `historico::sessoes()`.

A sessão já é um corte da série por quebra de continuidade. Ganha `jogo: Option<String>`: o
jogo que ocupa a maior parte das amostras do intervalo, exigindo pelo menos 60% delas.
Sessão dividida entre dois jogos fica sem nome — melhor nada que errado, que é a posição do
app em todo o resto.

**Pronto quando:** testes cobrem sessão de um jogo só, sessão dividida e sessão sem jogo
nenhum.

## T9. O ícone do jogo vira arte

**Onde:** `src-tauri/src/icones.rs`, comando novo, `src/componentes/Sessoes.tsx`.

`SHGetFileInfoW` com `SHGFI_ICON` devolve o `HICON` do exe. Converter para PNG de 32px pelo
mesmo caminho que o `bandeja.rs` já usa para desenhar, e guardar em
`%APPDATA%/Kontro/jogos/<hash>.png`. Extrai uma vez, reusa sempre.

Servir pelo protocolo `asset:` — a CSP do `tauri.conf.json` já libera
`img-src 'self' data: asset: http://asset.localhost`.

**Pronto quando:** a lista de sessões mostra o ícone do jogo, e um jogo desinstalado, com o
exe sumido, cai no glifo do controle sem quebrar a linha.

## T10. A sessão vira cartão

**Onde:** `Sessoes.tsx`, `sessoes.css`.

Hoje é uma linha de três colunas: quando, `de% → ate%`, duração. Vira um cartão de 56px:

    [ícone 32]  ELDEN RING                          -18%
                ontem, 21:14 · 2 h 40 min       6,8 %/h

O `%/h` daquela sessão é o número que ninguém tem e todo mundo quer: mostra que um jogo come
mais bateria que outro — vibração, áudio no controle, alto-falante ligado.

**Pronto quando:** clicar continua abrindo a sessão no gráfico, como já faz hoje.

---

# Movimento 3 — o ano do seu controle

Com T6 a T10 no lugar, os dados já estão gravados. Daqui para frente é desenho.

## T11. Página "Diário"

**Onde:** aba nova em `src/telas/Principal.tsx`, arquivo `src/telas/Diario.tsx`.

Um heatmap de 30 dias no estilo do GitHub: uma célula por dia, tamanho fixo, opacidade pelas
horas jogadas e **cor pela faixa de carga em que o dia terminou**. Abaixo, os jogos do
período em barras horizontais, ordenados por horas.

Isso responde "joguei muito esse mês?" numa olhada, e é a primeira tela do app que dá
vontade de mostrar para alguém.

**Pronto quando:** passar o mouse num dia mostra data, horas e jogos; dia sem dado é célula
vazia com borda, não célula preta.

## T12. Ranking de consumo por jogo

**Onde:** `historico.rs`, função nova, e `Diario.tsx`.

Média de `%/h` por jogo, sobre as sessões que tenham nome e pelo menos 30 minutos. Ordenado
do mais faminto para o mais econômico, com a contagem de sessões ao lado — sem contagem, uma
média de uma sessão só vira mentira estatística.

**Exigir 3 sessões** antes de mostrar um jogo no ranking. Abaixo disso, agrupar em "ainda
medindo", igual o cartão de saúde já faz.

**Pronto quando:** o ranking bate com a conta feita à mão sobre o `history.json` desta
máquina.

## T13. Cartão para compartilhar

**Onde:** `Diario.tsx`, comando novo em `lib.rs`.

Botão que gera um PNG de 1200x630 com o anel grande, as horas do mês, o top 3 de jogos com
ícone e o consumo médio. Marca do Kontro no rodapé.

Sem dependência nova: serializar o SVG que já está na tela, desenhar num `canvas` e
`toBlob`. A CSP não deixa buscar biblioteca de fora, e não precisa.

O Rust salva em Downloads e abre o Explorer com o arquivo selecionado — exatamente o que
`salvar_diagnostico` já faz no `lib.rs`.

**Pronto quando:** o PNG sai legível em 100% e em miniatura de rede social.

---

# Movimento 4 — a pílula merece mais

## T14. A sobreposição vira vidro

**Onde:** `src/telas/Sobreposicao.tsx`, `sobreposicao.css`.

Com o Acrylic da T1, a pílula deixa de ser um retângulo escuro por cima do jogo e passa a ser
vidro sobre a cena. Junto:

- **Aparecer e sumir com deslize**, 240ms, em vez de aparecer seca.
- **Modo discreto:** depois de 8s parada, cai para 45% de opacidade. Volta ao cheio quando o
  número muda ou quando a carga cruza um limiar. É reação a dado, não animação em loop —
  respeita a emenda do topo.
- **Contorno na cor da faixa** quando estiver em crítico. Em jogo, a pessoa não está lendo
  texto; está vendo cor na periferia.

**Pronto quando:** com o jogo em tela cheia exclusiva a pílula continua aparecendo, e o modo
discreto não engole a transição para vermelho.

---

# Movimento 5 — o chao muda, a semantica nao

## T15. Temas

**Onde:** `src-tauri/src/configuracoes.rs`, `sistema.rs`, `janelas.rs`, `src/estilo/tokens.css`,
`src/estado.ts`, `src/telas/Configuracoes.tsx`.

O app inteiro já roda em cima de tokens, então um tema é um bloco de override e nada mais.
Quatro opções: **Do Windows** (lê `AppsUseLightTheme`), **Noite**, **Preto** (OLED, chao em
`#000`) e **Dia**.

**A regra que separa tema de decoração:** o tema troca o chão — `ink`, `surface`, `stroke`,
os textos e a trilha do anel. **Verde, âmbar e vermelho são semântica de carga e continuam
significando a mesma coisa em todo tema.** No Dia eles escurecem só o quanto o contraste
exige (§8 do `DESIGN.md` pede 4,5:1), nunca mudam de família.

A pílula em jogo fica de fora: ela precisa ser legível sobre qualquer cena, então segue
escura em todo tema.

O Mica acompanha — `MicaLight` no Dia, `MicaDark` nos outros — e é reaplicado com
`set_effects` na hora em que o tema muda, sem reiniciar.

**Pronto quando:** trocar o tema muda a janela na hora, o texto passa no contraste nos
quatro, e a pílula continua legível sobre um jogo claro.

## T16. Auditar os temas, e fazer tema de verdade

**Onde:** `src/estilo/tokens.css`, `base.css`, `principal.css`, `configuracoes.rs`.

A T15 entregou os temas, mas com um buraco: **a pílula tem fundo escuro fixo e usava
`--text-primary`**. No tema Dia esse token vira quase preto — texto preto sobre pílula
preta, ilegível em jogo, que é justamente onde ela existe. O mesmo valia para o botão de
prender.

**A regra que faltava:** quem tem fundo próprio precisa de paleta própria. A janela da
sobreposição agora fixa a paleta escura inteira, independente do tema — inclusive as cores
de carga, que no Dia são escurecidas para fundo claro e ficariam lambuzadas sobre a pílula.

Junto: o polegar da barra de rolagem era `#3a444d` cravado, invisível no Dia; virou token.

**E os temas passaram a ter identidade.** Entrou `--realce`, separando a cor de *interface*
— aba ativa, foco, chave ligada, botão primário — da cor de *carga*, que continua sendo
semântica em todos. Com isso o Ardósia pode ser azul sem que verde deixe de significar
bateria boa.

Seis temas: **Do Windows**, **Noite**, **Preto** (OLED), **Ardósia** (frio, azulado),
**Brasa** (quente, noturno) e **Dia**.

**Pronto quando:** cada tela — resumo, sessões, diário, configurações — e a pílula foram
vistas nos seis temas, sem nada ilegível. Conferido por captura, tema a tema.

## T17. A landing page falava de protocolo, não de produto

**Onde:** `docs/index.html`, `docs/tela-*.png`.

A página descrevia GATT Battery Service, enumeração HID e extração de endereço Bluetooth
antes de dizer o que o app faz por quem usa. E estava parada na 1.5.0 — sem diário, sem
sessões com jogo, sem temas, sem pílula.

**A ordem virou:** primeiro a pergunta que o app responde ("quanto ainda dá pra jogar"),
depois o que ele mostra, com captura de tela de verdade em cada seção. O trecho técnico
sobrou em **uma** seção, e reenquadrado como confiança: por que o número é confiável, e
por que no cabo ele prefere não responder.

De quebra, a página deixou de ser um bundle de 347 KB que só renderiza com JavaScript.
Virou HTML estático de 26 KB — o único `<script>` que restou é o JSON-LD, que não executa
nada. Abre sem JS.

**Pronto quando:** a página foi percorrida de cima a baixo no navegador, sem texto
ilegível nem seção com metade vazia.

## T18. Clicar em atualizar travava o app

**Onde:** `src-tauri/src/atualizacao.rs`, `lib.rs`, `src/telas/Configuracoes.tsx`.

Dois problemas somados davam a sensação de travamento.

**O que travava de verdade:** `procurar_atualizacao` era um comando síncrono. No Tauri, o
padrão de `#[tauri::command]` é `ExecutionContext::Blocking` — ele roda na thread
principal, que é a thread da interface. E lá dentro havia um
`tauri::async_runtime::block_on` de uma requisição HTTP. Enquanto o GitHub não respondia,
**nenhuma janela repintava**. Numa rede lenta isso é o app inteiro congelado.

Agora o comando é `async fn` e usa `.await`: a requisição vai para o runtime assíncrono e a
interface segue viva. Medido: 260 ms depois do clique a tela já mostra "Procurando…" com a
requisição ainda em voo.

**O que parecia travamento:** a barra de progresso mentia. Quando o servidor não manda
`Content-Length`, `porcento` vem nulo e a barra ficava **parada em 8%** com um texto fixo —
indistinguível de um app pendurado. Pior: o passo era marcado como "baixando" *antes* do
`check()`, então a barra dizia "baixando" enquanto ainda era uma consulta.

Agora há um passo **preparando** para a consulta, e sem `Content-Length` a barra vira
indeterminada — uma faixa que percorre o trilho — com os megabytes já recebidos no texto.
Movimento aqui não contraria a emenda do topo: barra indeterminada é a forma honesta de
dizer "está andando, não sei o tamanho".

**Pronto quando:** clicar em Procurar não congela a janela, e a barra nunca fica parada num
número fixo enquanto o download acontece.

---

# Ordem

    T1 -> T2 -> T3 -> T4 -> T5        material e luz, na ordem de retorno
    T6 -> T7 -> T8 -> T9 -> T10       a cadeia do jogo, cada uma depende da anterior
    T11 -> T12 -> T13                 só faz sentido com dado de jogo gravado
    T14                               depende só da T1
    T15 -> T16 -> T17 -> T18          independentes de todas

T1 e T6 são independentes: dá para tocar o visual e a detecção em paralelo. Tudo de T11 para
frente precisa de duas semanas de dados gravados com jogo para ser visto de verdade — vale
começar a gravar cedo, com T6 a T8, mesmo que a tela venha depois.

# O que este plano recusa

- **Widget na Xbox Game Bar.** Exige empacotamento UWP e um projeto separado. O ganho não
  paga a segunda cadeia de build.
- **Detectar jogo por lista de títulos conhecidos.** Vira manutenção eterna e erra em jogo
  indie. Tela cheia mais nome do exe é honesto e não envelhece.
- **Estimar quanto falta para carregar.** Continua recusado pelo mesmo motivo da tarefa 24
  do `TAREFAS.md`: no cabo não existe percentual, e a conta seria chute com cara de conta.
