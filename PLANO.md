# Plano · "Kontro que dá vontade de abrir"

O `TAREFAS.md` é o registro do que quebrou e por quê. Este arquivo é o oposto: o que ainda
não existe e vale construir, em ordem, com critério de pronto. Cada tarefa é pequena o
bastante para caber num commit e grande o bastante para dar para ver a diferença na tela.

Regra que atravessa tudo: **nada aqui pode inventar número**. Brilho, cor e animação
descrevem estado medido — quando não há medida, não há efeito.

## Situação

**A Parte 2 entrou em 26/09/2026: 107 tarefas de interface, T23 a T129.** Entregues: T23 a
T71 — os movimentos da tela cheia, de desempenho e fluidez, de janelas, bandeja e atalhos,
de sistema de design e temas e de textos e estados inteiros, e as sete primeiras do Resumo —,
mais a T89, que a T32 tornou obrigatória.
Desvios: na T38 o `botao.css` não foi criado, porque o botão não tem folha própria até a
T55; o estado `aria-busy` mora no `principal.css` e no `painel.css`, junto das regras que
ele sobrepõe. Na T42 o Rust não esconde o painel direto: ao perder o foco ele grava o
carimbo e manda `kontro://painel-fechar`, e a página faz a saída de 120 ms da T35 antes de
esconder — esconder do Rust cortaria a animação. Na T43 o painel é assentado pela área do
cliente, descontando a borda invisível de redimensionar que o Windows põe na janela com
sombra: sem isso a margem de 12 px virava 3 px na lateral e 10 px embaixo. Na T44 o
ícone da bandeja foi conferido pixel a pixel contra os PNGs de referência do pacote de
assets (`tray/dark` e `tray/light`, 16 a 32 px, sete estados): além de tirar o disco, o
desligado passou a 45% de opacidade, o cabo virou anel cinza a 90% sem trilha, e o controle
da bandeja ficou cheio — os PNGs não vazam os sticks, e o §3 do DESIGN.md foi corrigido para
dizer isso. Sobra só antisserrilhado: no máximo 4 pixels por ícone diferem mais que 32/255.
Na T45 cada recusa carrega também qual atalho foi recusado (`Mostrar` ou `Mover`), porque o
campo volta ao anterior e a combinação recusada deixa de ser a que está na tela. Na T46
`modelo::resumo_do_estado`, `EstadoDoControle::faixa` e `tempo::quando`, que são da T48,
vieram antes, porque a primeira linha do menu é feita deles; a T48 só os usou na dica. Na
T48 o "--" saiu de `texto_da_carga`, e a lista de controles e a pílula passaram a escrever
"sem leitura" onde antes mostravam o traço. Na T49 o `janela.json` guarda a posição em pixel
físico e o tamanho em lógico: posição lógica é ambígua entre monitores de escalas
diferentes, e `monitor_from_point`, que decide se ela ainda vale, já trabalha em físico. Na
T55 o botão passa a usar `--text-primary` no texto, como o do WinUI, em vez do
`--text-secondary` de antes; e o cartão de versão nova também passou a usar `.leitura`, que
é o mesmo bloco de nome e rodapé. Na T56 a porcentagem em Display (34px) só coube no anel de 96 com o
rastreio de -0,02em que o §4 já previa: sem ele "100%" encostava no traço. Na T58 ficaram
fora da escala, de propósito, a pílula (`sobreposicao.css`) e a réplica dela no passo a
passo, que têm medida própria conferida na tela cheia, e o `diario.css`, que a tarefa do
calendário refaz; o topo do aviso foi de 28 para 24px, e `SANGRIA_SUPERIOR_DO_AVISO`
acompanhou para o cartão não sair do lugar. Na T57 as transições de várias propriedades
ficaram em várias linhas, cada uma com a curva. Na T61 o ponto de leitura velha e o rodapé em duas
fontes moram no `<Leitura>` da T55, que é o mesmo nos dois lugares, em vez de repetidos em
`principal.css` e `painel.css`. Na T63 a duração da sessão no rodapé do gráfico sai da
janela da sessão, a mesma conta da lista, e não do primeiro e do último ponto. Na T67, quando
dois rótulos do eixo cairiam a menos de 12px, a prioridade é aviso, crítico, 100, 50 e 0:
com aviso em 30 e crítico em 20 os dois ficam a 11,8px no gráfico de 150, e só o "30" sai —
a linha tracejada vermelha continua lá. Na T66 o bloco "por carga cheia" aparece também com o
controle desligado, porque é medida da bateria e não da sessão; os que dependem de taxa somem. Na T68 o
relógio de minuto e o rótulo "zera amanhã às 02:30" já existiam desde a T33 e a T63; o
Resumo já recarregava a cada `kontro://historico`, e `via`/`lidoEm` entraram nas dependências
mesmo assim. Na T70 o nome editável vai onde o nome aparece: no título com o controle
ligado e na segunda linha com ele desligado, onde o título é "Desconectado"; e "Esquecer" é
um `.botao.fantasma` abaixo de "Ler agora", já que o `.ciclo` não existe desde a T55. Falta a parte que só um jogo responde — o `QUNS`
num DX12 com otimizações, num DX11 com "Desativar otimizações de tela inteira" marcado e
num sem borda, tirados do diagnóstico e anotados no `TAREFAS.md`. É esse dado que libera a
frase de "tela cheia exclusiva" em Configurações, que a T26 deixou de fora de propósito: por
ora a partida só grava o valor bruto em `T`. A Parte 1 está toda na
2.13; o que ficou de pé dela:

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

## T19. O banner de compartilhamento ficou na versão antiga

**Onde:** `assets/banner.html` (fonte), `docs/banner.png`, `docs/index.html`.

Quando alguém colava o link do Kontro no WhatsApp, no Discord ou no LinkedIn, a imagem que
aparecia dizia "Bateria do seu controle, em tempo real, na bandeja do sistema" e listava
`WINDOWS · .NET 8 · WPF · MIT`. Duas coisas erradas de uma vez: a stack mudou para Tauri e
Rust faz tempo, e a linha técnica era exatamente o que a T17 tirou da página — o cartão
continuava vendendo protocolo enquanto o site já vendia produto.

O banner novo repete a promessa da página: a pergunta "quanto ainda dá pra jogar" em
destaque, a carga na bandeja em horas e minutos logo abaixo, e o selo de grátis e código
aberto. No lugar da lista de tecnologias, a pílula de verdade — o mesmo anel, o mesmo
desenho da seção "Em jogo" — em tamanho grande.

A fonte virou um HTML de 1280×640 versionado em `assets/`, renderizado por navegador sem
interface. Antes o PNG existia sozinho, sem fonte nenhuma de onde refazê-lo.

Também foram corrigidos no `index.html` o `softwareVersion` do JSON-LD, parado na 2.13.1, e
o `og:image:alt`. As três referências à imagem ganharam `?v=2`, porque as redes guardam o
cartão por URL e sem isso continuariam mostrando o antigo.

**Pronto quando:** o link colado numa rede mostra o cartão novo, e o banner abre em
`assets/banner.html` para ser refeito na próxima mudança de discurso.

## T20. A pílula se apagava sozinha durante o jogo

**Onde:** `src/telas/Sobreposicao.tsx`.

Passados 8 s sem a assinatura do estado mudar (`preenchimento|via|girando|carregando`), a
pílula caía para metade da opacidade configurada — 45% no padrão de 90%. A intenção era não
atrapalhar; o efeito era o contrário do que a emenda do topo manda: **luz mudando sozinha
sem o dado ter mudado**.

E o gatilho estava invertido. Carga é justamente o dado que fica parado: com o controle em
80%, o número não muda por meia hora. Quer dizer que o estado normal da pílula era o
apagado, e o brilho cheio virava a exceção dos oito segundos seguintes a cada leitura nova.

Agora a opacidade é só a das Configurações. Quem quer discreto escolhe 55% e ela fica em
55% o tempo todo.

**Pronto quando:** a pílula tem o mesmo brilho no primeiro segundo e dez minutos depois.

## T21. O Diário não sabia o nome de nenhum jogo

**Onde:** `src-tauri/src/historico.rs`, `monitor.rs`.

Dois dias jogando com a 2.13 e o ranking do Diário continuava em "Nenhuma sessão trouxe o
nome do jogo ainda". O `history.json` desta máquina explicava: das 160 amostras, uma só
carregava jogo.

**Causa:** a T7 pendurou o jogo na amostra de bateria, e amostra de bateria é rara. O GATT só
avisa quando a carga muda, e ela muda em degraus de 5 pontos — gastando entre 5 e 9 %/h, é
uma amostra a cada meia hora ou mais. O jogo em foco só era consultado nesses instantes. A
T8 então contava amostras, exigindo 60% delas com o mesmo jogo, e toda sessão tem duas que
nunca levam jogo: a leitura da conexão, feita antes de o jogo abrir, e o marcador de
desligado. A sessão de 08/09 tinha quatro amostras e uma com o FC 26 — 25%, sem nome,
embora o jogo tenha ocupado 87 dos 124 minutos.

**Correção:** o jogo virou uma linha do tempo própria, `partidas` no `history.json`, com
início, fim e jogo. A cada ciclo de 2 s, se algum controle está ligado, o monitor consulta
o jogo em foco e estende a partida em curso. Sair do jogo por até 5 minutos não a encerra —
um alt-tab para o Discord ainda é jogar.

A sessão agora é batizada pelo tempo, não pela contagem: leva o jogo que ocupou pelo menos
60% da duração dela. As amostras gravadas com jogo pela 2.13 continuam valendo, cada uma
pelo trecho até a amostra seguinte, e é assim que a sessão de 08/09 ganha o nome. A amostra
de bateria deixa de guardar o jogo, porque mantê-la gravando contaria o mesmo tempo duas
vezes.

Uma linha à parte, e não amostras extras na série de bateria, porque uma amostra repetida
no começo de um patamar muda o ponto onde `descargas` começa a contar a queda — a projeção
de autonomia pularia a cada troca de jogo.

**Pronto quando:** a sessão de 08/09 aparece como EA SPORTS FC 26, o Diário mostra um jogo
em medição, e um teste reproduz as quatro amostras dela.

## T22. Versões beta, antes da final

**Onde:** `.github/workflows/release.yml`, `src-tauri/src/atualizacao.rs`, `lib.rs`,
`src/telas/Configuracoes.tsx`.

Até a 2.13.4 havia um canal só: a versão saía para todo mundo no instante em que a tag era
empurrada, e não havia como rodar uma mudança em uso real antes de ela virar a versão de
todos.

Agora são dois canais, um por branch. A `develop` publica uma beta a cada push, numa
release rolante `beta` com o próprio `beta.json`; a `main` publica a estável quando o
número do `tauri.conf.json` ainda não saiu. *Receber versões beta*, em Configurações, faz o
app consultar os dois manifestos e ficar com o maior. A consulta saiu do front para o
Rust, que é quem sabe o canal.

A beta nunca fica abaixo da última estável publicada — com a `develop` num número que já
saiu, ela é da correção seguinte. Sem isso quem liga a opção não recebe nada.

A instalação consulta de novo antes de baixar, em vez de reaproveitar o achado da última
busca: a release `beta` é recriada a cada push, o instalador leva a versão no nome, e o
endereço guardado vira 404 na build seguinte. Trocar a opção também refaz a busca, para o
cartão de versão nova nunca oferecer o canal que a pessoa acabou de desligar.

**Pronto quando:** um push na `develop` publica `X.Y.Z-beta.N` acima da última estável, o
app com a opção ligada oferece essa beta e instala, e um push na `main` com número novo
publica a estável — que quem está na beta recebe.

---

---

# Parte 2 · A interface a 100%

A Parte 1 fez o app parar de parecer uma página web escura e passar a saber o que você
jogou. Esta parte é o resto: tudo que, olhando tela por tela, ainda separa o Kontro de um
app Windows 11 de primeira linha. O levantamento foi feito sobre `98aa8bd` (2.13.4 mais a
T22), área por área, com cada achado conferido no código antes de virar tarefa e as
duplicatas entre áreas fundidas numa só. Saíram 107 tarefas, T23 a T129, em catorze
movimentos.

O primeiro movimento é o pedido que abriu esta parte: a pílula por cima da tela cheia. O
desenho técnico inteiro — o que o Windows permite, o que não permite, o que se recusa e o
roteiro de teste — está em `TELA-CHEIA.md`; aqui entram só as tarefas.

A regra da Parte 1 continua atravessando tudo: **nada aqui pode inventar número**. E ganha
uma irmã: **nada aqui muda o dado, só o jeito de mostrá-lo**. Onde uma tarefa precisa de um
campo novo vindo do Rust, é um campo que o Rust já calcula e ainda não entrega.

## Emenda ao DESIGN.md, segunda

- §7 descreve o flyout com raio 16 e sombra própria. Passa a raio 8 com a sombra do sistema,
  como todo flyout do Windows 11.
- §7 usa AccentGreen como cor de ação. Toda cor de interface — aba ativa, foco, chave ligada,
  botão primário, ponto de progresso — é `--realce`; verde continua sendo carga.
- §7 chama o botão do flyout de "Atualizar". O app tem uma ação de reler a bateria e outra de
  procurar versão nova, e elas passam a ter nomes diferentes: "Ler agora" e "Procurar".
- §7 fixa a janela principal em 480 de largura com cabeçalho de 56. A janela tem mínimo de
  720 e moldura de 32; o número certo é o que está no código.
- §2 ganha `--realce` (cor de interface, uma por tema), `--atencao` (falha e erro, que não é
  a cor de carga crítica) e o laranja do Brasa.
- §3 pede reação a `WM_SETTINGCHANGE` para o tema da barra. Vale como está; a sondagem de
  30 s que existe hoje é o que sai.
- T14 prometia "com o jogo em tela cheia exclusiva a pílula continua aparecendo". Não é
  alcançável sem injetar código no jogo, e o plano recusa isso. O critério passa a ser "em
  tela cheia com otimizações do Windows ou em janela sem borda a pílula continua
  aparecendo"; o porquê está em `TELA-CHEIA.md`.

---

# Movimento 6 — A pílula por cima da tela cheia

"A pílula some" são dois problemas de naturezas opostas. Um é ordem de empilhamento — o
jogo passa por cima e nada a traz de volta — e se resolve com poucas dezenas de linhas. O
outro é tela cheia exclusiva legada, em que o compositor do Windows não desenha aquele
monitor e nenhuma janela de nenhum processo aparece ali; só quem desenha dentro do próprio
jogo aparece, e isso o plano recusa. A primeira metade se resolve aqui. A segunda pede
honestidade, não mais código.

Cinco tarefas, T23 a T27, na ordem das dependências: primeiro medir, depois higiene, depois
voltar para cima, depois dizer quando não dá, por fim o DPI. O `TELA-CHEIA.md` cita cada
uma pelo nome curto entre aspas angulares.

**Trade-off que fica explícito:** qualquer janela por cima de um jogo em flip model pode tirá-lo do independent flip (§1.2 B). O Kontro já paga isso sempre que a pílula está visível; «Higiene» e «Voltar ao topo» não acrescentam custo, mas não o tiram. O único jeito de não pagar é não mostrar — é por isso que o padrão continua `EmJogo`, e não `Sempre`. E o A3 do roteiro decide se "Sempre" precisa de uma frase a mais na descrição.

**UIAccess, recusado com condição de reabertura registrada:** o que ele compra sobre «Voltar ao topo» é o jogo que reafirma topmost a cada quadro; o que custa está na §2.3. Reabrir só se, com «Medir», «Higiene» e «Voltar ao topo» no lugar, um jogo real cobrir a pílula **com o diagnóstico dizendo `Composta` e `alguem_por_cima` apontando o jogo como janela opaca** — aí é D de verdade e a banda é a única saída que resta. Mesmo assim, a assinatura para pessoa física no Brasil hoje não existe pelo Artifact Signing, então a condição é necessária, não suficiente.

## T23. Medir antes de mexer: `tela.rs` distingue o que a API distingue

**Onde:** `src-tauri/src/tela.rs`, `jogo.rs`, `diagnostico.rs`, `lib.rs`, `janelas.rs`, `src/telas/Configuracoes.tsx`.

`em_tela_cheia()` passa a ser uma casca sobre `Tela::atual()`, com `Tela { Livre, Composta, Exclusiva, Apresentacao, Outro(i32) }` construído do `QUNS` bruto (`QUNS_BUSY` → `Composta`, `QUNS_RUNNING_D3D_FULL_SCREEN` → `Exclusiva`, `QUNS_PRESENTATION_MODE` → `Apresentacao`, o resto em `Outro`, inclusive `QUNS_APP`, que se registra e não se age) e `Tela::bruto() -> i32`. `Tela::conta_como_jogo()` mantém **exatamente** a regra de hoje (os três valores), para o Diário e a T21 não mudarem por tabela; a pílula continua usando a mesma regra, porque, sem esconder em `Exclusiva` (tarefa «Dizer»), não há divergência a codificar — um segundo predicado idêntico seria ficção.

`jogo::executavel_em_foco` vira `pub(crate)` e ganha uma irmã `executavel_de(janela: HWND)`, para o ocluidor; os dois passam por `batizar` antes de sair do módulo. `janelas.rs` ganha `hwnd_de(&WebviewWindow) -> Option<HWND>` (extraída de `arredondar_cantos`) e o `Compartilhado` guarda o HWND cru da pílula num `OnceLock<isize>` preenchido no `setup` (§2.7).

`diagnostico::AoVivo` ganha `pilula: Option<isize>`; `salvar_diagnostico` passa a receber `app: AppHandle` além do estado e preenche o campo; no `--diagnose` fica `None`. A seção nova `=== Tela ===` registra: `QUNS` bruto e o nome do variante; **nome batizado** do programa em foco (o `FileDescription` ou o nome do arquivo, o que `jogo::batizar` já produz — **nunca** o caminho completo, que carrega o nome de usuário do Windows, e **nunca** o título da janela, que é conteúdo de terceiro: uma aba, um documento, uma conversa); retângulo da janela em foco contra o monitor dela; e, com a pílula, `GWL_EXSTYLE` lido de `GetWindowLongPtrW`, se está visível, e o resultado de `alguem_por_cima()` com o nome batizado de quem está por cima. Sem pílula, a linha "(sem pílula: modo --diagnose)" e só os três primeiros itens. Em Configurações, o texto do botão "Salvar diagnóstico" no estado parado passa a dizer que o arquivo inclui o nome do programa em primeiro plano na hora do clique — hoje ele não contém nada disso, e ninguém deve mandar um arquivo sem saber o que vai nele.

**Nenhuma mudança de comportamento nesta tarefa.**

**Pronto quando:** o `diagnostico.txt` desta máquina registra o `QUNS` num jogo DX12 com FSO, num DX11 com "Desativar otimizações de tela inteira" marcado e num sem borda — as três primeiras linhas do roteiro — e os valores vão para o `TAREFAS.md`; o arquivo não contém caminho nem título de janela de outro programa; e o `--diagnose` continua funcionando sem janela. É esse dado que decide a frase da tarefa «Dizer».

## T24. Higiene: a pílula não rouba foco, sai do Alt+Tab e volta ao topo ao aparecer

**Onde:** `src-tauri/src/janelas.rs`, `orquestra.rs`, `atalho.rs`, `lib.rs` (`soltar_sobreposicao`).

`.focusable(false)` no builder da sobreposição e do aviso. `vestir_estilos(&janela)` em `janelas.rs`: lê `GWL_EXSTYLE`, grava com `WS_EX_TOOLWINDOW`, e faz **um** `SetWindowPos(HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_FRAMECHANGED)` — o `FRAMECHANGED` que a doc de `SetWindowLong` exige e a reafirmação de Z, numa chamada. Chamada depois de todo `show()` e de todo `set_ignore_cursor_events`, porque o `tao` reescreve o `GWL_EXSTYLE` a cada mudança de flag (§1.1) — e **sempre na thread da interface, depois do `apply_diff`** (§2.8): inline em `alternar` e `soltar_sobreposicao`; no orquestrador, `show()` e `vestir_estilos` juntos num `app.run_on_main_thread`. Emenda ao `PLANO.md`: o "pronto quando" da T14 passa a "em tela cheia com otimizações do Windows ou em janela sem borda a pílula continua aparecendo".

**Pronto quando:** a pílula e o aviso não aparecem no Alt+Tab depois de dez mostra/esconde vindos do ciclo (E1); soltar, arrastar e prender continuam funcionando, com `GetForegroundWindow()` anotado antes e depois de cada um (E2 — se o WebView2 ativar a pílula solta ao clique, isso vira registro no desenho, e `focusable(false)` fica mesmo assim); e uma janela topmost que ficou por cima na sessão anterior está atrás da pílula no primeiro `show()` da sessão seguinte (D4).

## T25. Voltar ao topo: a pílula volta para cima de quem passa na frente

**Onde:** `src-tauri/src/tela.rs`, `orquestra.rs`, `lib.rs`, `Cargo.toml`, `src-tauri/examples/janela_teimosa.rs`, `.github/workflows/verificar.yml`, `src/telas/Configuracoes.tsx`.

`alguem_por_cima(hwnd) -> Option<HWND>` em `tela.rs`, com o filtro de camadas (§2.2 item 2). No orquestrador, `reafirmar_topo(&app)`: teste de oclusão, relógio próprio de uma reafirmação por segundo, contador de três, e `SetWindowPos` via `run_on_main_thread`. Chamado do ciclo normal e de `Ok(Pedido::PrimeiroPlanoMudou)`, tratado no `match` sem reiniciar o laço nem passar por `monitor.ciclo()` — laço interno com prazo até o próximo ciclo. Hook de `EVENT_SYSTEM_FOREGROUND` numa thread própria com message loop, assinaturas da §2.2 item 3, `Pedido` em `pub(crate)`. Ao desistir, `coberta_por: Option<String>` no `Compartilhado` recebe o nome batizado do ocluidor (nunca título, nunca caminho) e chega ao front pelo mesmo caminho do estado; limpa quando o primeiro plano muda, a pílula se esconde ou a partida troca de jogo. `Cargo.toml`: `Win32_UI_Accessibility` e `Win32_System_LibraryLoader`. O exemplo `janela_teimosa.rs` — janela sem borda do tamanho do monitor, `SetWindowPos(HWND_TOPMOST)` a cada 500 ms numa variante e a cada quadro na outra — é versionado **e compilado no CI**: `verificar.yml` roda hoje `cargo test --manifest-path src-tauri/Cargo.toml --lib` (linha 37), que não compila exemplos; entra um passo `cargo build --manifest-path src-tauri/Cargo.toml --examples`.

**Pronto quando:** a `janela_teimosa` a 500 ms cobre a pílula e ela volta em menos de 1 s e fica (D1); a cada quadro, o Kontro para depois de três tentativas, sem piscar, e Configurações mostra quem está por cima (D2); Alt+Tab de volta ao jogo recoloca a pílula sem esperar o ciclo, e uma rajada de dez trocas de foco em um segundo produz **uma** reafirmação e **nenhuma** leitura de bateria extra (D3); com o overlay novo do Discord ligado e a Game Bar aberta, nada é reafirmado, `coberta_por` fica vazio e nenhuma frase aparece (D5); e o CI falha se o exemplo deixar de compilar.

## T26. Dizer quando não dá

**Onde:** `src-tauri/src/historico.rs`, `monitor.rs`, `lib.rs`, `src/telas/Configuracoes.tsx`, `docs/index.html`.

A parte "está por cima" entra com «Voltar ao topo», porque se apoia no teste de oclusão. A parte "tela cheia exclusiva" é só registro e frase — **a pílula não se esconde em `Exclusiva`**: onde o DWM não compõe, mostrar não custa nada, esconder não compra nada visível, e esconder com base num sinal medido numa máquina só seria errar para o lado que mais dói em qualquer outra (§1.2 A e B).

- `Partida` ganha `tela: Option<i32>` (em disco `"T"`, `#[serde(default)]`, ao lado de `I`/`F`/`J`): o `QUNS` bruto da primeira leitura da partida; se em algum ciclo vier `QUNS_RUNNING_D3D_FULL_SCREEN`, passa a ele e não volta. Valor bruto, não um bool chamado `exclusiva` — para que o dado exista antes da interpretação, e para que uma segunda máquina possa contradizer a primeira.
- A linha em **Configurações › Sobreposição** (§4.2) com partida `tela == 3` nos últimos 30 dias **só entra se «Medir» tiver registrado, nesta máquina, que jogo com FSO não devolve 3** (medição B1). Se devolver, a frase não entra em lugar nenhum e fica só o dado bruto no diagnóstico e na partida — uma frase que contradiz uma pílula visível ao lado é pior que nenhuma.
- O rótulo "tela cheia exclusiva" no cartão da sessão do Resumo **não entra agora**: só depois de mais de uma máquina confirmar que o valor 3 separa A de B.
- `docs/index.html`, seção "Em jogo": uma frase (§4.2).

**Pronto quando:** com um DX9 ou DX11 em exclusivo legado, a partida aparece no Diário com o nome certo e `T: 3` no `history.json`, a pílula continua sendo mostrada (e `is_visible()` diz `true`), e — se B1 liberou a frase — Configurações explica o que o Windows informou; desmarcando a caixa de compatibilidade (ou trocando para janela sem borda), a partida seguinte grava outro valor e a frase some, sem outra mudança.

## T27. Posição em pixel físico

**Onde:** `src-tauri/src/janelas.rs` (`posicionar_sobreposicao`, `redimensionar_sobreposicao`, `palco`, `onde_a_sobreposicao_parou`).

Passam a calcular em físico (`monitor.position()` e `monitor.size()` já são) e a chamar `set_position(PhysicalPosition)`, para o `tao` não converter com a escala da janela (§1.1).

**Pronto quando:** com um monitor a 100% e outro a 150%, "Monitor 2" põe a pílula no canto pedido do monitor 2 no primeiro ciclo, e arrastar entre os dois no modo solta grava o monitor certo (G1).

---

# Movimento 7 — Desempenho e fluidez

O app já tem material, luz e diário. O que falta é o que se sente antes de ler qualquer coisa: a janela nasce em branco e troca de tema depois de aparecer, o anel gira por horas com o controle no cabo, trocar de aba pisca vazio, a cor salta em vez de transitar, o gráfico sai esticado, e o segundo controle e as sessões param de acompanhar o que o Rust já sabe. Por baixo, cinco cópias do mesmo listener por janela, um flyout que redesenha escondido e um ciclo de 2 s que relê o executável do jogo. Este movimento não acrescenta tela nenhuma: faz cada frame nascer pronto, cada mudança de dado chegar à tela em até um ciclo, e nada se mexer sem o dado ter mudado.

## T28. No cabo, o anel para de girar, e cinza não emite luz

**Onde:** `src/componentes/Anel.tsx`, `src/componentes/anel.css`, `src/estado.ts`, `src/telas/Passos.tsx`.

Com o controle plugado sem percentual, `girando` (`modelo.rs:168`, `via == Cabo && preenchimento.is_none()`) é um estado que dura a carga inteira, e o front o trata como spinner: `.anel .giro` roda `kontro-girar 1.7s linear infinite` em teal (`corDoAnel`, `estado.ts:102`) no Resumo, no painel, na lista, no aviso e na pílula, enquanto o ícone da bandeja (`bandeja.rs:57`, `(Via::Cabo, None)`) mostra o anel inteiro cinza e parado, o estado *cable* do DESIGN.md §3. É a única animação em loop que sobrou, contra o §6 e a emenda do PLANO.md, e as webviews compõem um frame a cada 16 ms sem dado novo. Em jogo, é um objeto girando na periferia da visão a noite inteira. E `.anel .carga` aplica `drop-shadow` em qualquer cor: o anel cinza do cabo, do desligado e da leitura velha brilha como carga viva.

`corDoAnel` devolve `var(--gray)` para `girando`, como já faz para desligado. Em `Anel.tsx`, o ramo `girando` desenha o mesmo `<circle>` do ramo `cheio` (`r={raio}`, `strokeWidth={espessura}`) em `var(--gray)`, sem gradiente, com o glifo no miolo como hoje; `.giro`, `@keyframes kontro-girar` e `ARCO_DO_GIRO` saem. O `<g className="carga">` ganha a classe `apagada` quando `cor === "var(--gray)"`, e `.anel .carga.apagada { filter: none }`. O passo 2 de `Passos.tsx` (`Passos.tsx:153`) troca `cor="var(--accent-teal)"` por `var(--gray)` e mostra o mesmo anel cheio. O campo `girando` continua existindo: decide o texto "cabo" da pílula e esconde as marcas; só deixa de mover alguma coisa.

**Pronto quando:** plugar o controle sem número mostra, em todas as janelas, um anel cinza cheio, parado e sem halo, com o glifo dentro, idêntico ao da bandeja; duas capturas da pílula com 10 s de intervalo são idênticas pixel a pixel; no Gerenciador de Tarefas o processo do WebView2 fica em 0% de GPU com o controle no cabo e as janelas paradas.

## T29. A janela nasce pronta

**Onde:** `src-tauri/src/lib.rs` (`setup`, `Compartilhado`, comando novo), `src-tauri/src/janelas.rs` (`criar_todas`, `criar_principal`), `src-tauri/src/configuracoes.rs`, `src/main.tsx`, `src/App.tsx`, `src/estado.ts` (`useTema`), `src/telas/Principal.tsx`, `src/telas/Painel.tsx`.

O `setup` chama `j.show()` (linha 203, e `painel.show()` na 190 com `--painel`) antes de o WebView2 carregar o `index.html`. Sem material, a janela não é `transparent(true)` e `criar_principal` não define `background_color`: o WebView2 pinta branco por algumas centenas de ms. Com Mica, aparece uma moldura vazia. Depois vem a barra com os botões sem estilo, porque toda regra do `principal.css` é escopada por `body[data-janela="principal"]` e `data-janela`, `data-tema` e `data-material` só são carimbados nos `useEffect` do `App.tsx`, depois do primeiro paint. O fundo pula de Ink opaco para 55% translúcido quando `material_da_janela` responde; quem usa Dia vê tudo nascer escuro (`useTema` começa em `"noite"`); e na primeira abertura o trilho aparece antes do passo a passo, porque `Principal.tsx` cai na UI normal enquanto `passos === null`.

Três partes. O Rust põe na URL o que já sabe: `janelas.rs` monta `index.html?janela=principal&tema=<noite|preto|ardosia|brasa|dia>&material=<sim|nao>`, com o tema de um `Settings::tema_efetivo()` novo (`Sistema` vira `dia` ou `noite` por `sistema::windows_no_claro()`) e o material de `sistema::material_disponivel()`; `main.tsx` carimba `document.body.dataset.janela/tema/material` lendo a URL antes do `createRoot().render`, os `useEffect` que carimbavam saem do `App.tsx`, `useTema` nasce com `document.body.dataset.tema`, e o comando `material_da_janela` sai. Ninguém mostra janela antes de ela ter o que mostrar: no lugar dos `show()` do setup, `Compartilhado.abrir_ao_carregar: Mutex<Vec<String>>` guarda os rótulos a abrir, e o comando `janela_pronta(rotulo)` tira o rótulo da lista e, se estava lá, faz `unminimize()`, `show()` e `set_focus()`; `Principal.tsx` o chama num `useEffect` quando `passos` deixa de ser `null`, `Painel.tsx` quando `estado` chega. Os `show()` do clique na bandeja e da instância única ficam, porque ali a página já carregou. Enquanto `passos === null`, `Principal` renderiza só `<div className="app"><BarraDeTitulo/></div>`. Na janela principal que não nasce `transparent(true)`, `criar_todas` recebe `claro: bool` e `criar_principal` chama `.background_color(tauri::window::Color(11, 14, 17, 255))`, ou `(238, 241, 245, 255)` com `cfg.tema_claro()`, o `--ink` do Noite e do Dia; com material a cor não entra, porque o wry a ignora em janela transparente e o Mica sumiria. Nada de `.theme(Some(...))` no construtor: com tema fixado o tao para de emitir `ThemeChanged`, que a tarefa do tema usa.

**Pronto quando:** gravando a abertura a 60 fps (Xbox Game Bar), o primeiro frame visível já tem o fundo do tema escolhido, o layout final e a translucidez definitiva; com a transparência do Windows desligada, dez aberturas a frio seguidas nunca mostram quadro branco nem moldura vazia; na primeira abertura o trilho nunca aparece antes do passo a passo; `--painel` abre o flyout já desenhado.

## T30. O segundo controle acompanha

**Onde:** `src-tauri/src/monitor.rs`.

Com dois controles ligados, o que não é o principal congela em "Seus controles" e nos acompanhantes da pílula: cai de 80% para 60% e a tela continua em 80% até o principal (o de menor carga, por `escolher_principal`) mudar alguma coisa. `Monitor::ciclo()` guarda só `ultimo: Option<EstadoDoControle>` e devolve `Some` quando o principal mudou (`igual_a`) ou `todos.len()` mudou; `kontro://controles` sai do `lib.rs` apenas nesse caso.

O que muda: `ultimos: Vec<EstadoDoControle>` no lugar de `ultimo`, e `mudou = ultimos.len() != todos.len() || todos.iter().zip(&ultimos).any(|(a, b)| !a.igual_a(b))` — o principal está em `todos`, então a comparação antiga fica coberta. `ler_agora`, `renomear` e `esquecer`, que zeravam `ultimo` para forçar o próximo panorama, esvaziam o vetor. Nada muda no front: `useControles` já escuta o evento, e `atualizar_bandeja` tem assinatura própria, então o ícone não é redesenhado à toa.

**Pronto quando:** com dois controles, o de maior carga perde 1% e a linha dele na lista e o acompanhante na pílula atualizam em até 2 s, com o principal parado.

## T31. Sessões, saúde e Diário acompanham o histórico, e o Resumo nasce na altura final

**Onde:** `src-tauri/src/lib.rs`, `src/telas/Resumo.tsx`, `Painel.tsx`, `Diario.tsx`, `src/componentes/historico.css`.

Desligar o controle fecha a sessão no Rust (`marcar_desligado`), mas "Últimas sessões" não ganha a linha, o gráfico não mostra a quebra e a saúde não recalcula até a carga mudar de número: `Resumo` e `Painel` só refazem os `invoke` quando `estado.chave` ou `estado.percentual` mudam, e desligar muda `via`, não percentual. O `Diario` busca `sessoes_do_controle` uma vez e nunca mais. E ao abrir o Resumo, três `invoke` independentes chegam em três momentos: o cartão do histórico nasce baixo com "sem histórico nesta janela" (16 px de padding sobre 12 px de texto), salta para 150 px quando a série chega, depois ganha a saúde, depois as sessões, depois a lista de controles — cada chegada empurra o que está abaixo.

O que muda: no laço do `lib.rs`, o bloco que renova `serie`, `sessoes` e `saude` em `Compartilhado` emite `kontro://historico` (sem payload) — é exatamente o momento em que o que o front pode buscar mudou, e cobre desligar, amostra nova e sessão batizada. Um comando novo `resumo_do_controle` devolve `{ serie, sessoes, saude }` de uma vez; `Resumo` faz um único `setState` com os três, disparado por `estado.chave` e pelo evento; `Painel` faz o mesmo com a série (gated pela abertura, na tarefa do flyout) e `Diario` escuta o evento em vez de buscar uma vez. `historico.css`: `.historico.vazio { min-height: 150px; display: grid; place-items: center }` e `.historico.compacto.vazio { min-height: 64px }`, para o cartão já ter a altura do gráfico antes de a série chegar.

**Pronto quando:** desligar o controle no meio de uma sessão faz "Últimas sessões" ganhar a linha em até 2 s e o gráfico mostrar a quebra, sem a carga ter mudado; a mesma sessão aparece no heatmap do Diário já aberto; abrir o Resumo mostra o cartão do histórico já na altura final, e nada abaixo dele desce depois.

## T32. As abas viram abas: trocar não pisca, o traço desliza, as setas andam e a página lembra onde estava

**Onde:** `src/telas/Principal.tsx`, `src/telas/principal.css`.

Clicar em Resumo mostra a página vazia, nem o título, por um round-trip de IPC, e então tudo aparece de uma vez: `Principal` renderiza `{pagina === "resumo" && <Resumo/>}`, cada troca desmonta e remonta a página, todo hook refaz seu `invoke`, e `Resumo` devolve `null` até `estado` chegar. Voltar ao Resumo reinicia "7 dias", desfaz a sessão escolhida no gráfico e volta a rolagem ao topo. A aba ativa troca sem movimento: `.aba.ativa` é um `box-shadow: inset 2px 0 0 var(--realce)` de altura cheia que aparece num item e some do outro, não o traço curto do NavigationView. Passar o mouse numa aba inativa a deixa idêntica à ativa, as duas em `--surface-alt`. Com teclado, Tab passa pelas três uma a uma, setas não fazem nada e Ctrl+Tab não troca de página; o Narrador lê "Resumo, botão" sem dizer que é a página atual, porque `aria-current={pagina === p.id}` gera `"true"`/`"false"` e a `nav` não tem papel nem nome.

As três páginas ficam montadas, cada uma num `<section className="folha" role="tabpanel" id="painel-{id}" aria-labelledby="aba-{id}" hidden={pagina !== id}>` dentro de `.pagina`; `overflow-y: auto` e o padding saem de `.pagina` e vão para `.pagina > .folha`, e `Principal` guarda o `scrollTop` de cada folha num `useRef` antes de trocar e o devolve num `useLayoutEffect` ao voltar, porque `display: none` não preserva rolagem. Entrada: `.pagina > .folha { animation: kontro-entrar var(--motion-base) var(--curva) }` com `@keyframes kontro-entrar { from { opacity: 0; transform: translateY(6px) } }`; uma animação CSS recomeça toda vez que o elemento sai de `display: none`, então dispara só na troca, sem JS. Trilho: `nav role="tablist" aria-orientation="vertical" aria-label="Páginas"`; botões `role="tab"`, `aria-selected`, `aria-controls="painel-{id}"`, `id="aba-{id}"`, `tabIndex={ativa ? 0 : -1}`; `onKeyDown` com ArrowUp, ArrowDown, Home e End movendo foco e seleção, e Ctrl+Tab e Ctrl+Shift+Tab ciclando no nível da janela. O `box-shadow` de `.aba.ativa` sai e entra um único `<span className="indicador">` absoluto de 3×16px, `border-radius: 2px`, em `--realce`, com `.trilho { position: relative }`, movido por `transform: translateY(var(--indicador-y))` com `transition: transform var(--motion-base) var(--curva)`; `--indicador-y` é `offsetTop + (offsetHeight - 16) / 2` do botão ativo, medido por ref. Hover em `color-mix(in srgb, var(--text-primary) 6%, transparent)`, `:active` 3%, `.ativa` 10%.

**Pronto quando:** ir ao Diário e voltar mantém "30 dias", a sessão escolhida e a posição de rolagem; a troca não mostra frame vazio; o conteúdo entra em fade de 180 ms e o traço da aba desliza entre os itens; Tab entra no trilho uma vez e as setas trocam de página; o Narrador diz "aba, 2 de 3, selecionada"; hover, ativa e pressionada são três tons distintos.

**Entregue com a T89 junto.** Com as páginas montadas o tempo todo, a cópia da config que Configurações guarda envelheceria pela vida inteira da janela, e não só enquanto a aba está aberta: soltar a pílula no Resumo e ligar qualquer chave dias depois a devolveria ao lugar antigo. O que o remontar renovava de graça — a versão nova achada em segundo plano e o número de telas — passa a ser perguntado ao entrar na aba, e o Diário refaz a grade de dias ao entrar. O trilho é `div role="tablist"`, não `nav`: o ARIA em HTML não aceita outro papel em `nav`. E o gravador de atalho passa a parar ao sair da aba: sem desmontar, ele seguia engolindo teclas no Resumo e gravava Ctrl+Tab como atalho global.

## T33. O gráfico desenha em pixel e para de recalcular

**Onde:** `src/componentes/Historico.tsx`, `src/componentes/historico.css`.

O SVG tem `viewBox="0 0 520 150"` (70 no compacto) com `preserveAspectRatio="none"` e CSS `width: 100%; height: 150px` (64 no compacto): o eixo X escala por largura/520 e o Y fica em 1. Em 720px tudo comprime 17%; maximizado em 1920px o cartão tem ~1630px para 520, os rótulos "03/09" e "100" ficam três vezes mais largos que altos, a linha de 2px vira 6px na horizontal e o ponto do hover (`r=4`) vira elipse. No painel a anisotropia é 0,57 × 0,91, e o `BAIXO.altura` de 70 nem casa com os 64 px do CSS. Os rótulos 0/50/100, as datas, "bateria nova" e "zera" saem em 9 e 10px. Por baixo, `agora = Date.now()` é lido por render, então `inicio` e `fim` mudam sempre, o `useMemo` de `segmentar` nunca acerta, `caminho()` roda duas vezes por trecho (área e linha), e cada `mousemove`, e cada `kontro://estado`, reconstrói todos os `<path>` de 30 dias; `aoMover` varre as amostras uma a uma.

A largura real do wrapper é medida com `ResizeObserver` (o padrão de `Painel.tsx:45-62`), em estado, com 520 só até a primeira medida; largura 0, de aba escondida, mantém a última. O `viewBox` vira `0 0 <largura medida> <ALTURA>` sem `preserveAspectRatio`, e `BAIXO.altura` passa a 64. `x()`, o `LARGURA - M.direita` das linhas de grade, o `LARGURA - 90` do rótulo de troca e o limite do rótulo de projeção usam a largura medida. `.rotulo-y`, `.rotulo-x`, `.rotulo-troca` e `.rotulo-projecao` sobem para 12px, e `ALTO.esquerda` vai de 32 para 36 para o "100" caber. `agora` vira estado quantizado ao minuto (`useState(() => Math.floor(Date.now() / 60_000) * 60_000)` com `setInterval` de 60 s, limpo no cleanup: relógio, não animação); `caminhos = useMemo(() => trechos.map((t) => caminho(t, x, y)), [trechos, inicio, fim, largura, compacto])`, e a área reaproveita a string da linha; `aoMover` acha a amostra por busca binária em `t`; a mira, o ponto e o rodapé vão para um filho que recebe `sob`, e o SVG estático fica num componente com `React.memo`, para o hover repintar só o `<g>` da mira.

**Pronto quando:** com a janela em 720, 840 e maximizada, os rótulos do gráfico têm a mesma largura de glifo que o texto "Carga" ao lado, o ponto do hover é um círculo e a linha tem 2px em qualquer inclinação; no painel a linha tem espessura uniforme; o "100" do eixo fica inteiro à esquerda; no Performance do DevTools (`npm run tauri dev`), varrer o gráfico com o mouse só altera o `<g>` da mira, e o evento de estado a cada leitura não recria nenhum `<path>`.

## T34. A cor do anel transita, e o halo tem o tamanho do anel

**Onde:** `src/componentes/Anel.tsx`, `src/componentes/anel.css`, `src/estilo/tokens.css`, `src/telas/principal.css`, `src/telas/painel.css`, `src/telas/sobreposicao.css`.

Quando a carga cruza um limiar, ou o limiar muda nas Configurações, o arco muda de cor num frame, o halo num frame, o fundo do cartão de estado num frame, e o contorno vermelho da pílula aparece seco. A T2 dava como pronta a transição verde → âmbar → vermelho em 180 ms, e o código não a entrega: os `<stop>` recebem `stopColor` como atributo, sem `transition`; `.carga` só transiciona `filter`, então `color`, que alimenta o `drop-shadow`, salta; `.cartao.estado::before` e `.painel .topo::before` têm `transition: background` sobre um `radial-gradient`, e gradiente não interpola; `.sobreposicao .pilula` só transiciona `opacity`. O halo é `drop-shadow(0 0 18px currentColor)` em todo tamanho: calibrado para o anel de 96px, onde é 19% do diâmetro, vira 60% no anel de 30px da pílula e 90% nos acompanhantes de 20px, um borrão que atravessa os 7px de padding do `.pilula` e flutua fora do contorno por cima do jogo; no aviso, anel de 44px, a mesma desproporção, e na lista (26px) o brilho sangra na linha vizinha. No Resumo passa dos 12px a 0,35 que a T2 fixou.

`tokens.css` registra `@property --cor-do-anel` e `@property --cor-do-estado` (`syntax: "<color>"; inherits: true; initial-value: #8d979f`): propriedade registrada interpola, e o gradiente que a lê é reavaliado por frame. `Anel.tsx` passa `style={{ width, height, "--cor-do-anel": cor, "--anel-tamanho": tamanho + "px" }}` na raiz `.anel` e para de escrever `stopColor` e `color` inline. `anel.css`: `.anel { transition: --cor-do-anel var(--motion-base) var(--curva) }`, `.anel stop:first-child { stop-color: color-mix(in srgb, var(--cor-do-anel) 84%, #fff) }`, `.anel stop:last-child { stop-color: var(--cor-do-anel) }`, `.anel .carga { color: var(--cor-do-anel); filter: drop-shadow(0 0 calc(var(--anel-tamanho) * 0.125) color-mix(in srgb, currentColor 35%, transparent)) }`, o que dá 12px em 96, 5,5px em 44, 3,75px em 30 e 2,5px em 20, todos mortos antes do padding de quem os contém; o `.carga.apagada { filter: none }` da tarefa do cabo continua valendo. `principal.css` e `painel.css`: a `transition: background` dos `::before` vira `transition: --cor-do-estado var(--motion-base) var(--curva)` em `.cartao.estado` e `.painel .topo`, que são quem define a variável. `sobreposicao.css`: `.sobreposicao .pilula` ganha `box-shadow var(--motion-base) var(--curva)` na transição.

**Pronto quando:** com o controle em ~35%, subir o limiar de carga baixa de 20% para 40% faz arco, halo, fundo do cartão e, na pílula, contorno passarem de verde a âmbar num fade contínuo de 180 ms, sem salto; no Resumo o halo tem ~12px a 35%; na captura da pílula o brilho não ultrapassa o contorno arredondado, no aviso não ultrapassa a borda do cartão, e na lista não passa da borda da linha.

**Entregue com dois ajustes.** O diagnóstico do halo supunha o `drop-shadow` em px de tela, e num filho de `<svg>` ele não é: o Chromium mede o filtro no espaço do `viewBox`. Medido: os 18 do código antigo davam ~3,4 px de desfoque no anel de 96, e o `calc(var(--anel-tamanho) * 0.125)` daria ~2 px. O que entrega a conta da tarefa é 12,5% da caixa de 512, `--raio-do-halo` de 64 unidades, que sai idêntico, pixel a pixel, a um `drop-shadow` de `tamanho × 0,125` px em HTML — 12 px em 96, 3,75 em 30 — sem o anel precisar saber o próprio tamanho. E `box-shadow` só interpola entre listas com o mesmo `inset` em cada posição: a normal, a crítica e a solta da pílula passaram a ter quatro sombras alinhadas, senão o contorno vermelho continuaria aparecendo seco.

## T35. O flyout desliza a cada abertura, cresce sem pular e descansa quando está escondido

**Onde:** `src/telas/Painel.tsx`, `src/telas/painel.css`, `src-tauri/src/lib.rs` (setup do `--painel`, clique na bandeja, `ajustar_altura_do_painel`).

O DESIGN §6 pede 240 ms para abrir o flyout, e a entrada `kontro-subir` só acontece uma vez por processo: a animação está no mount de `.painel`, e `hide()`/`show()` da janela não remontam nada. Nas aberturas seguintes o painel aparece seco, e fechar não tem saída. Quando a lista de controles ou o gráfico entra com o painel aberto, `ajustar_altura_do_painel` faz `set_size` e depois `posicionar_painel`: a janela cresce para baixo num frame e sobe para a âncora no seguinte. E escondido ele continua trabalhando: o `useEffect` de `Painel` depende de `estado.percentual` sem olhar visibilidade, então cada 1% busca a série inteira, re-renderiza o gráfico, mede a altura e manda o Rust fazer `set_size` + `posicionar_painel` numa janela invisível; `medir()` direto mais o primeiro callback do `ResizeObserver` chamam o comando duas vezes por efeito.

Os dois pontos que mostram o painel no `lib.rs`, o clique na bandeja e o `janela_pronta` do `--painel`, viram `abrir_painel(app)`: `posicionar_painel`, `show()`, `set_focus()` e `app.emit("kontro://painel-abriu", ())`. `Painel.tsx` guarda `aberto` (verdadeiro no evento, falso ao fechar) e `aberturas` (incrementado no evento) e usa `key={aberturas}` no `.painel`, o mesmo padrão de `kontro://pilula-apareceu` na `Sobreposicao`; o `Anel` nasce em `valor`, então remontar não refaz o preenchimento do zero. `kontro-subir` passa a `from { opacity: 0; transform: translateY(8px) }` em `var(--motion-slow) var(--curva)`, o deslize de baixo para cima dos flyouts do Windows 11. Saída: `fechar()` põe `.saindo`, `@keyframes kontro-descer` em `var(--motion-fast)` até `opacity: 0; transform: translateY(8px)`, e esconde a janela no `animationend`. Buscar a série e medir só acontecem com `aberto`; ao abrir ele busca e mede uma vez; a chamada direta a `medir()` sai e o `ResizeObserver`, que dispara ao observar, faz o resto. No Rust, `ajustar_altura_do_painel` retorna cedo se `!janela.is_visible()` e, em vez de `set_size` + `set_position`, calcula x e y para a altura nova e aplica posição e tamanho numa chamada só de `SetWindowPos` (via `janela.hwnd()`, como `arredondar_cantos` já faz, com `SWP_NOZORDER | SWP_NOACTIVATE`).

**Pronto quando:** a terceira abertura do flyout tem o mesmo deslize de 240 ms da primeira, e fechar pelo Esc tem os 120 ms de saída; quando a lista aparece, a borda de baixo do painel não se mexe e a de cima cresce num frame só; com o painel fechado, o processo do WebView2 dele não mostra picos de CPU a cada leitura no Gerenciador de Tarefas.

**Entregue sem o `key={aberturas}`.** Remontar o painel a cada abertura remontava junto a lista de controles, que nasce vazia até o `invoke` voltar: medido, toda abertura mandava duas alturas, 275 e 422, e a borda de cima pulava duas vezes — o contrário do critério. A troca de classe já recomeça a animação, porque `animation-name` sai de `none` (guardado) ou de `kontro-descer` para `kontro-subir`; o painel só remonta no caso raro de o Rust ter escondido a janela sem a página saber. E o `SetWindowPos` recebe o tamanho da janela, não o do conteúdo: o painel nasce com `shadow(true)`, que põe borda invisível em volta, e sem somar essa moldura o WebView encolheria uns 16 × 9 px a cada medida. O X, que o `.topo` cobria desde a T2, ganhou `z-index`.

## T36. Uma fonte por dado, não uma por hook

**Onde:** `src/estado.ts`.

Ao abrir o Resumo há cinco `invoke("configuracoes")` e cinco `listen("kontro://config")` — `App` (via `useTema`), `Principal`, e `Resumo`, `Historico` e `ListaDeControles` (via `useLimiares`) — e cada salvamento de configuração dispara cinco `setState` em cascata; `useEstado` e `useControles` duplicam entre `Resumo`/`Sobreposicao` e `ListaDeControles`/`Sobreposicao`. `useLimiares` devolve um objeto novo por render, então `marcas` do `Anel` e `limiares` do `Mapa` mudam de identidade sempre. A causa é que `useConfig`, `useEstado`, `useControles`, `usePilulaSolta` e `useAtalhosRecusados` são cinco cópias do mesmo `useState` + `invoke` + `listen` por instância.

O que muda: uma fábrica `criarFonte<T>(comando, evento, inicial)` em `estado.ts`, com o último valor em cache, um único `listen` por janela e um `Set` de assinantes, exposta por `useSyncExternalStore`; os cinco hooks viram `useFonte(fonteX)`; `useLimiares` passa a `useMemo(() => ({ critico, aviso }), [cfg?.CriticalThreshold, cfg?.WarnThreshold])`. Com o cache, uma página remontada já nasce com dado e o `return null` do primeiro render some.

**Pronto quando:** com um `console.count` no listener, abrir o Resumo registra um único `configuracoes` e um único `estado_atual`, e salvar uma configuração dispara o handler de `kontro://config` uma vez por janela.

## T37. O tema e a transparência acompanham o Windows na hora, em todas as janelas, e o Preto é preto

**Onde:** `src-tauri/src/lib.rs` (`on_window_event`, `iniciar_ciclo`, `Compartilhado`), `src-tauri/src/janelas.rs` (`criar_principal`, `criar_painel`, `vestir_material`), `src-tauri/src/sistema.rs`, `src/estado.ts` (`useTema`), `src/App.tsx`, `src/estilo/tokens.css`.

Com o tema "Do Windows", trocar o Windows de escuro para claro deixa o Kontro no tema antigo por até 30 s: `useTema` faz `setInterval(windows_no_claro, 30_000)` em cada uma das quatro janelas, lendo o registro. Quando o CSS enfim vira Dia, o Mica continua `MicaDark` para sempre, porque `vestir_material` só roda no setup e em `salvar_configuracoes` quando `theme` muda, e `on_window_event` só trata `CloseRequested`: Ink claro a 55% sobre Mica escuro vira um cinza sujo. O painel nunca recebe tema: sem `set_theme`, o Acrylic segue o claro e escuro do Windows, e Kontro em Noite com Windows claro tem um painel leitoso. `material_disponivel()` é lido uma vez na criação e uma vez no `App.tsx`: desligar "Efeitos de transparência" com o app aberto deixa a janela transparente a 55% mostrando a área de trabalho através do texto, e religar não devolve o Mica até reiniciar. E o Preto, vendido como "OLED, chão em #000", com transparência ligada é 55% de `#000` sobre Mica, um cinza que muda com o papel de parede. O §3 do DESIGN.md manda reagir a `WM_SETTINGCHANGE`, e o tao já o traduz em `WindowEvent::ThemeChanged` para janela sem tema preferido.

Tema: `vestir_material` passa a receber a `Settings` inteira e chama `set_theme` nas quatro janelas, `None` com `Sistema` e `Some(Light | Dark)` nos outros; é o `DWMWA_USE_IMMERSIVE_DARK_MODE` que dá o tom ao Acrylic do painel. Na principal, `Theme::Preto` faz `set_effects(None)` e os demais reaplicam `MicaLight` ou `MicaDark`; o painel reaplica o `Acrylic`. `on_window_event` vira um `match` e ganha o braço `ThemeChanged(_)` da janela `PRINCIPAL`: `let claro = sistema::windows_no_claro()`, emite `kontro://tema-do-sistema` com `claro` e, se `cfg.theme == Theme::Sistema`, chama `vestir_material`. `useTema` mantém a consulta inicial (ou o `data-tema` da URL, da tarefa da abertura), escuta o evento e perde o `setInterval`.

Transparência: `sistema` separa `tem_material()` (build ≥ 22000) de `transparencia_ligada()`. `criar_principal` e `criar_painel` nascem `transparent(true)` sempre que há build para isso e só aplicam o efeito com a transparência ligada: uma janela transparente sem efeito e com `--fundo-janela` no Ink sólido é opaca, o fallback que o `tokens.css` já descreve. O ciclo de 2 s guarda `ultimo_material: bool`; quando `transparencia_ligada()` muda, chama `set_effects(None)` nas duas janelas ou `vestir_material`, e emite `kontro://material`; `App.tsx` escuta e troca `data-material`, que já leva `--fundo-janela` de volta ao Ink.

Preto: em `tokens.css`, `body[data-tema="preto"][data-material="sim"]` devolve `--fundo-janela: var(--ink)`, `--fundo-barra: var(--ink)` e `--fundo-trilho: var(--surface)`.

**Pronto quando:** com "Do Windows", alternar o modo do Windows muda tokens e Mica juntos, em todas as janelas, em menos de 240 ms e sem passar por estado misto; nenhuma consulta ao registro se repete a cada 30 s; com Kontro em Noite e Windows claro, o painel aberto tem o chão escuro do tema, sem véu leitoso; alternar a transparência do Windows com a janela aberta a deixa sólida no Ink do tema ou de volta ao Mica em até 2 s, sem nunca mostrar o que está atrás; captura da principal em Preto com transparência ligada tem `#000000` no chão da página e do trilho, e voltar para Noite mostra o Mica de novo.

## T38. "Ler agora" diz que está lendo

**Onde:** `src/componentes/BotaoLerAgora.tsx` (novo), `src/telas/Resumo.tsx`, `src/telas/Painel.tsx`, `src/componentes/botao.css`, `DESIGN.md` §7.

Clicar em "Atualizar" no Resumo ou no flyout não muda nada: o botão fica igual, e quando a leitura devolve o mesmo número a tela fica idêntica; a pessoa clica de novo, três vezes, achando que não pegou, a tarefa 7 do `TAREFAS.md` do lado da percepção. No flyout é pior, porque é a única ação. `ler_agora` só enfileira o `Pedido::LerAgora`, mas o monitor zera `ultimo` em `ler_agora()`, então um `kontro://estado` sempre chega depois do pedido: o front tem como saber que terminou e não usa. E o nome mente duas vezes: a bandeja chama a mesma ação de "Ler a bateria agora", e a seção Versão tem "Atualizar agora", que baixa e instala; logo depois do toast "Kontro 2.14 disponível", o "Atualizar" do flyout lê como a outra coisa. Com o controle desligado, o botão também não diz que vai revarrer (`monitor.ler_agora` chama `descobrir`).

`BotaoLerAgora` com estado `lendo`, ligado no clique e desligado no próximo `kontro://estado` ou, por segurança, após 5 s, porque a descoberta mais a releitura do GATT passam de 1,5 s com facilidade. Enquanto `lendo`: rótulo "Lendo…", `disabled` e `aria-busy="true"`; `.botao[aria-busy="true"]` mantém a cor e só tira o hover, em vez dos 60% de `:disabled`. Rótulo "Ler agora"; com `estado.via === "Desligado"`, "Procurar" e `title="Procura o controle e relê a bateria"`. Usado no Resumo e no Painel no lugar dos dois `<button>`. "Atualizar agora" fica exclusivo da seção Versão, e no `DESIGN.md` §7 o botão do flyout passa de "Atualizar" a "Ler agora". A leitura que volta com `lidoEm` novo já muda o rodapé "atualizado às" sozinha: esse é o retorno.

**Pronto quando:** clicar em "Ler agora" deixa o botão em "Lendo…" desabilitado até o estado chegar, no máximo 5 s, com o rodapé mostrando o horário novo quando a leitura chegou; dois cliques seguidos não enfileiram duas leituras; a palavra "Atualizar" só aparece na seção Versão.

## T39. A barra de rolagem só aparece com o mouse em cima, segue o tema e não muda a largura dos cartões

**Onde:** `src/estilo/base.css`, `src/estilo/tokens.css`, `src/telas/principal.css`, `src/telas/passos.css`.

No Resumo os cartões terminam em x≈806, porque há barra; no Diário, em x≈816, porque não há: trocar de aba faz todo o conteúdo mudar de largura, como `docs/tela-resumo.png` e `tela-diario.png` mostram. A barra desenhada é a fina padrão do Chromium, não a dos `::-webkit-scrollbar`: `base.css` põe `scrollbar-width: thin` e `scrollbar-color` em `*`, e desde o Chromium 121 qualquer valor fora de `auto` faz o navegador ignorar `::-webkit-scrollbar*`; as 20 linhas são CSS morto, e o `--polegar` da T16 nunca chega à tela. O Windows 11 usa barra fina que só aparece com o mouse em cima. E nada declara `color-scheme`, então o campo de renomear tem cursor e seleção claros dentro do tema escuro, e a rolagem das notas da versão nova também.

Sai o bloco `::-webkit-scrollbar*` inteiro, e sai a linha 15 de `base.css` (`-webkit-font-smoothing` é regra de macOS); fica `scrollbar-width: thin`. No contêiner que rola, `.pagina > .folha` depois da tarefa das abas, entram `scrollbar-gutter: stable; scrollbar-color: transparent transparent` e `:hover { scrollbar-color: var(--polegar) transparent }`, com o token que já existe por tema; o mesmo par em `.cartao.novidade .notas` e `.passos .folha`. `tokens.css` declara `:root { color-scheme: dark }` e `body[data-tema="dia"] { color-scheme: light }`; Preto, Ardósia e Brasa herdam.

**Pronto quando:** os cartões têm a mesma largura no Resumo, no Diário e nas Configurações, conferido com as capturas sobrepostas; a barra só fica visível com o mouse sobre a página; não resta seletor `::-webkit-scrollbar` no bundle; no Noite, selecionar texto no campo de renomear mostra seleção escura e a rolagem das notas é escura sem regra própria, e no Dia é clara.

## T40. O anel e o download respeitam "Efeitos de animação" desligado

**Onde:** `src/movimento.ts` (novo), `src/componentes/Anel.tsx`, `src/telas/principal.css`.

Com Configurações > Acessibilidade > Efeitos visuais > Efeitos de animação desligado, o bloco global de `prefers-reduced-motion` do `base.css` leva toda animação CSS a 0,01 ms com `animation-iteration-count: 1`. Duas coisas escapam. O arco do anel é interpolado em JS, por `requestAnimationFrame` durante 180 ms, em todas as janelas: o único movimento que sobra é o que a pessoa pediu para tirar. E `kontro-vaivem` para no keyframe final, `translateX(280%)`, então a faixa indeterminada do download some do trilho e parece que nada está acontecendo.

`useMovimentoReduzido()` em `movimento.ts`, com `matchMedia("(prefers-reduced-motion: reduce)")` e listener de `change`; em `Anel.tsx`, se reduzido, `setSuave(alvo)` direto, sem rAF. Toda animação em JS daqui para frente passa por esse hook, inclusive a entrada dos anéis do passo a passo. O bloco global do `base.css` fica como está: tirar o `animation-iteration-count: 1` faria as animações infinitas rodarem para sempre a 0,01 ms. Em `principal.css`, dentro do mesmo media, `.cartao.novidade .trilho.indefinido span { animation: none; transform: none; width: 100%; opacity: 0.4 }`. O anel no cabo não precisa de estado parado próprio: deixou de girar na tarefa do cabo.

**Pronto quando:** com "Efeitos de animação" desligado, o anel vai de 80% para 60% num frame, como o resto da interface, e religado volta a animar em 180 ms; durante um download sem tamanho conhecido o trilho mostra uma faixa cheia e apagada em vez de ficar vazio.

## T41. Durante o jogo, o ciclo de 2 s para de refazer o que já sabe

**Onde:** `src-tauri/src/monitor.rs`, `jogo.rs`, `orquestra.rs`, `janelas.rs`, `lib.rs`.

Com o controle ligado e um jogo em tela cheia, a cada ciclo o Kontro abre o `.exe` do jogo e lê o bloco de versão (`GetFileVersionInfoSizeW` + `GetFileVersionInfoW`, em `jogo::descricao`) para descobrir o mesmo nome de sempre — `em_foco()` → `de()` → `batizar()` sem cache, e `anotar_jogo` recebe o `Jogo` pronto todo ciclo, no momento em que a máquina está mais ocupada. `tela::em_tela_cheia()` (`SHQueryUserNotificationState`) é consultada duas vezes por ciclo, em `jogo::em_foco` e em `orquestra.sobreposicao`. E com a pílula visível e parada, `sobreposicao()` chama `posicionar_sobreposicao` a cada ciclo: enumera monitores, lê `outer_size` e faz `set_position` no mesmo ponto.

O que muda: `Monitor` ganha `nomes: HashMap<PathBuf, String>`; `jogo::executavel_em_foco()` fica público e o `Monitor` batiza pelo cache, chamando `jogo::batizar` só na primeira vez que vê um caminho. `Monitor::ciclo()` consulta `tela::em_tela_cheia()` uma vez, guarda em `self.tela_cheia`, e o `lib.rs` passa `monitor.tela_cheia()` para `orquestrador.reavaliar`, que deixa de consultar. `posicionar_sobreposicao` lê `outer_position()` e só chama `set_position` quando `|dx|` ou `|dy|` passa de 0,5 px lógico.

**Pronto quando:** no Process Monitor filtrado por `kontro.exe`, durante um jogo em tela cheia só aparece uma leitura do executável do jogo, na primeira detecção; com a pílula visível e a janela do jogo parada, um contador de `set_position` (log em dev) fica em zero por minutos, e arrastar o jogo para outro monitor continua movendo a pílula em até 2 s.

---

# Movimento 8 — Janelas, bandeja e atalhos

O Rust já sabe criar as quatro janelas, desenhar o ícone e registrar os atalhos, mas cada uma dessas peças ainda se comporta como se o app fosse a única coisa na tela: a pílula e o cartão de conexão ativam a própria janela por cima do jogo, o painel nasce sempre no canto inferior direito e fecha-e-reabre quando a pessoa clica no ícone para fechá-lo, o ícone é um disco preto em qualquer barra, o tema do Windows é sondado a cada 30 s e o Mica nunca acompanha, e gravar um atalho dispara o atalho que está sendo trocado. O que vira: janelas que aparecem sem tomar o foco de ninguém, um painel ancorado no ícone em qualquer lado da barra e em qualquer DPI, um ícone que segue a barra clara ou escura sem fundo próprio, uma janela principal que reage ao Windows no instante em que ele muda, atalhos que se deixam regravar e recusam sem mentir, e um menu de bandeja que diz a carga antes de qualquer clique.

## T42. Clicar no ícone com o painel aberto fecha o painel

**Onde:** `src-tauri/src/lib.rs` (`on_window_event`, `montar_bandeja`, `Compartilhado`), `src/telas/Painel.tsx`.

Painel aberto, clique no ícone para fechar: ele pisca e volta. Só fecha por Esc, pelo X ou clicando em outro lugar. O botão do mouse descendo sobre a barra de tarefas tira o foco do painel antes de o clique terminar; `onFocusChanged(false)` e o `blur` do `window` em `Painel.tsx` mandam `hide()`. O `TrayIconEvent::Click` com `MouseButtonState::Up` chega em seguida, `painel.is_visible()` já é falso, e o handler cai no ramo que posiciona e mostra.

O fechar ao perder foco sai do front e vai para o Rust, que é quem recebe o clique da bandeja. `Compartilhado` ganha `painel_escondido_em: Mutex<Option<Instant>>`. O `on_window_event`, que hoje só olha `CloseRequested`, trata também `WindowEvent::Focused(false)` da janela `janelas::PAINEL`: esconde e grava `Instant::now()`. No handler do clique, antes do ramo que mostra, se o carimbo tem menos de 300 ms o clique é o mesmo gesto que acabou de esconder e nada acontece. `Painel.tsx` perde o `onFocusChanged` e o listener de `blur`; fecham o Esc e o X, enquanto o X existir.

**Pronto quando:** clique no ícone abre; segundo clique fecha e o painel fica fechado; clicar fora continua fechando.

## T43. O painel abre junto do ícone, em qualquer lado da barra e em qualquer DPI

**Onde:** `src-tauri/src/janelas.rs` (`posicionar_painel`, `MARGEM_INFERIOR_DO_PAINEL`), `src-tauri/src/lib.rs` (`montar_bandeja`, `ajustar_altura_do_painel`, o `--painel` do `setup`, `Compartilhado`).

O flyout nasce sempre no canto inferior direito da área útil. Com a barra no topo ele aparece no canto oposto ao ícone; com a barra à esquerda, idem. A folga inferior é 8 px e a lateral 12 px; o DESIGN §7 diz 12. E em DPI misto o primeiro clique na tela de escala diferente abre o painel deslocado, às vezes com um terço para fora: `outer_size()` devolve pixels físicos no DPI do monitor onde a janela está agora, e `posicionar_painel` converte com a `escala` do monitor do cursor — 328 px físicos a 100% viram 218 lógicos quando o destino está a 150%. O `TrayIconEvent::Click` já carrega `rect` do ícone, e o handler descarta com `..`.

`posicionar_painel(app, ancora: Option<Rect>)`. O lado da barra sai da diferença entre `work_area()` e o monitor: `area.y > monitor.y` é barra em cima, painel em `y = area.y + 12`; `area.x > monitor.x` é barra à esquerda, `x = area.x + 12`; `area.width < monitor.width` é barra à direita; senão embaixo, como hoje. No eixo livre o painel centra no `rect` do ícone (físico, convertido com a escala do monitor onde o ícone está) e é limitado a `[area + 12, area + util - tamanho - 12]`. `MARGEM_INFERIOR_DO_PAINEL` vira 12. O tamanho da janela deixa de ser medido: largura é `LARGURA_DO_PAINEL` e altura é a última recebida por `ajustar_altura_do_painel`, guardada em `Compartilhado.altura_do_painel`. A âncora também fica em `Compartilhado.ancora_do_painel: Mutex<Option<Rect>>`, porque `ajustar_altura_do_painel` roda depois de cada medida do `ResizeObserver` e reposiciona — sem guardar, ele devolveria o painel ao canto logo depois do clique. O `--painel` do setup passa `None` e cai no canto.

**Pronto quando:** barra embaixo: o painel abre centrado sobre o ícone, 12 px acima da barra. Barra no topo ou na lateral: abre encostado nela, do lado do ícone, sem tocar nenhuma borda. Dois monitores 100%/150%: abre a 12 px do canto nos dois já na primeira abertura.

## T44. O ícone da bandeja segue a barra: sem disco, escuro na barra clara

**Onde:** `src-tauri/src/bandeja.rs` (`montar_svg`, `desenhar`, `salvar_previa`), `src-tauri/src/sistema.rs`, `src-tauri/src/geometria.rs`, `src-tauri/src/lib.rs` (`atualizar_bandeja`, `--icon-preview`), `src-tauri/src/icones.rs` (`vetores_de_referencia`).

Com a barra de tarefas clara o ícone é um disco preto com glifo branco no meio dos ícones cinza-escuros do sistema, e o anel de carga só existe dentro desse disco. Trocar o tema da barra não muda nada. `cor_glifo` é sempre `g::BRANCO`; `montar_svg` concatena `{fundo}` — o círculo r=252 em `#0F1318` mais a borda — antes do miolo, que o DESIGN §3 proíbe na bandeja ("sem o fundo circular — a bandeja é o fundo"). `sistema.rs` só lê `AppsUseLightTheme`; `SystemUsesLightTheme`, que é a chave da barra e a que o §3 manda usar, não é lida em lugar nenhum. E a assinatura de redesenho (`via|preenchimento|tamanho|limiares`) não tem o tema, então nem redesenharia.

`sistema::barra_clara()` lê `SystemUsesLightTheme` na mesma chave `Personalize`. `montar_svg(estado, limiares, barra_clara: bool)` sai sem `{fundo}`; glifo, risco e trilha em `g::GLIFO_ESCURO = "#1B1F24"` quando clara e `g::BRANCO` quando escura, trilha a `TRILHO_OPACIDADE` (já é 0.22). O `{fundo}` continua só em `svg_do_app`, que é o ícone do app e tem disco por desenho. `atualizar_bandeja` põe `barra_clara` na assinatura — o ciclo de 2 s já relê o registro, então a troca da barra repinta sem listener novo. `salvar_previa --claro` passa o mesmo flag para o `montar_svg`, em vez de só trocar o fundo da tira; `vetores_de_referencia` gera `tray-*-claro.svg` ao lado dos três que existem.

**Pronto quando:** `kontro --icon-preview x.png 16 --claro` mostra glifo escuro sem disco sobre `#F3F3F3`; alternar a barra do Windows entre claro e escuro repinta o ícone no ciclo seguinte, em até 2 s.

## T45. Gravar um atalho não dispara o atual, diz por que recusou e devolve o anterior

**Onde:** `src-tauri/src/atalho.rs` (`aplicar`), `src-tauri/src/lib.rs` (`salvar_configuracoes`, `atalhos_recusados`, comando novo), `src/estado.ts` (`useAtalhosRecusados`), `src/telas/Configuracoes.tsx` (`Captura`, `lerCombinacao`, `descricaoDoAtalho`), `src/telas/principal.css`, `src/estilo/tokens.css`, `src/ajustes.ts`.

Clicar em "pressione a combinação" e teclar Ctrl+Shift+K mostra e esconde a pílula no meio da gravação, e pior: `RegisterHotKey` consome a tecla antes de ela chegar ao WebView, então a `Captura` nunca recebe o `keydown` da combinação atual e não dá para confirmá-la. Apertar só "K" não faz nada: `lerCombinacao` devolve `null` e o botão continua igual, sem dizer que falta modificador. Tab para desistir também é engolido: `aoTeclar` faz `preventDefault()` em tudo e só Esc sai, e alt-tab ou clique em outro lugar deixa a captura ouvindo o teclado do app inteiro. Gravar em "Soltar" o mesmo atalho de "Mostrar" é revertido em silêncio por `Settings::ajustar`. Uma combinação que não parseia recebe o mesmo "Outro programa já usa essa combinação" de uma que falhou no registro, no mesmo `--text-tertiary` de toda descrição, e parece texto normal. E `salvar_configuracoes` chama `novas.salvar()` antes de `atalho::aplicar`, sem repor nada quando falha: o atalho recusado fica gravado e o anterior, que funcionava, some no próximo boot. O `.pedindo` é um `<span>` sem `aria-live`, então entrar em escuta não é anunciado.

Rust: comando `pausar_atalhos(pausar: bool)`, com `unregister_all` quando a `Captura` entra em escuta e `aplicar` de novo quando sai, por qualquer caminho. `aplicar` devolve `Vec<Recusa { combinacao, motivo }>`, com `motivo: Invalida` quando `combinacao()` é `None` e `EmUso` quando `register` falha; o plugin achata o erro do `global_hotkey` num `Error::GlobalHotkey(String)`, então "já registrado" e "o sistema recusou" ficam no mesmo texto. Em `salvar_configuracoes`, `aplicar` roda antes de `salvar()`; para cada recusa o campo volta ao valor que ainda está em `compartilhado.config`, e só então salva e emite `kontro://config` e `kontro://atalhos`.

Front: `Captura` ganha o estado `dica`: quando `lerCombinacao` devolve `null` e a tecla não é modificador, mostra por 2 s "Junte Ctrl, Alt, Shift ou Win" dentro do botão (`setTimeout` limpo no cleanup) e continua ouvindo; recebe a prop `outra` com a combinação da outra linha e, se for igual, mostra "Já é o atalho de mostrar a pílula" ou "de soltar a pílula" e não grava. Antes do `preventDefault`, `Tab` encerra a escuta e deixa o foco seguir; `blur` da janela e `pointerdown` no `document` fora do botão também encerram. O `.pedindo` vira `role="status"` com "Pressione a combinação · Esc cancela", em maiúscula como todo botão. Token `--atencao` em `tokens.css`: `#ff8fa3` no `:root` (8:1 sobre `--surface` nos temas escuros) e `#b4235a` em `body[data-tema="dia"]` (6:1 sobre branco), um rosa fora das três famílias de carga. `descricaoDoAtalho` devolve `{ texto, erro }`, com "Outro programa já usa essa combinação. Escolha outra." e "O Windows não aceita essa combinação."; `Linha` ganha a prop `erro`, que aplica `.linha.erro .descricao { color: var(--atencao) }`, e a captura recusada leva `.captura.recusada { border-color: var(--atencao) }`. Ao lado da captura, um botão "Padrão" em 12px só quando a combinação difere de `Ctrl+Shift+KeyK` ou `Ctrl+Shift+KeyM`, constantes espelhadas de `ATALHO_DA_PILULA` e `ATALHO_DE_MOVER` em `ajustes.ts`.

**Pronto quando:** gravar Ctrl+Shift+K com a pílula visível não a esconde, e a combinação atual é aceita como confirmação; apertar "K" sozinho mostra o motivo no botão; gravar Ctrl+Shift+M em "Mostrar" mostra a colisão sem salvar; Tab sai da escuta e move o foco, e clicar em outro lugar a encerra; uma combinação recusada pelo sistema aparece em `--atencao` na descrição e na borda, e o campo volta ao atalho anterior, que continua funcionando depois de reiniciar o app; o Narrador anuncia "Pressione a combinação · Esc cancela" ao entrar.

## T46. O menu da bandeja diz a carga, e "Configurações" abre Configurações

**Onde:** `src-tauri/src/lib.rs` (`montar_bandeja`, `atualizar_bandeja`, `mostrar_janela`, o handler de instância única, o `abrir_direto` do setup, `Compartilhado`), `src-tauri/src/atalho.rs` (`alternar`), `src-tauri/src/modelo.rs`, `src/telas/Painel.tsx`, `src/telas/Principal.tsx`, `src/telas/Passos.tsx` (passo 5).

Botão direito no ícone mostra três linhas iguais: Configurações, Ler a bateria agora, Sair. Nada diz a carga sem abrir o painel, e o DESIGN §7 pede o estado desabilitado no topo, separador, depois as ações, o padrão dos menus de bateria, rede e volume do Windows 11. A ação mais usada do app, mostrar e esconder a pílula, não está no menu, e não há mnemônicos. "Configurações", no menu e no botão do painel (`Painel.tsx:117`), abre a janela na aba em que ela estava, Resumo na maioria das vezes: `Principal` guarda `pagina` num `useState` nascido em `"resumo"` e não existe canal para pedir aba. Com a janela minimizada o botão do painel não faz nada visível: `mostrar_janela` só chama `show()`; o item do menu faz `show()` e `set_focus()` sem `unminimize()`; só a instância única faz os três. E o painel, `always_on_top`, continua por cima da janela que acabou de abrir. O passo Pronto ainda promete "um clique nele abre o resumo, e o menu do botão direito leva às configurações", quando o clique esquerdo abre o painel.

`fn trazer_principal(app)` esconde o painel e faz `unminimize()`, `show()` e `set_focus()`; é usada pela instância única, pelos itens do menu, por `mostrar_janela` e pelo `abrir_direto` do setup. Comando `abrir_aba(aba: Option<String>)`: `trazer_principal` e, com aba, `app.emit("kontro://abrir-aba", aba)`; `Principal.tsx` escuta com `listen<Pagina>` e faz `setPagina`, com o cancelamento na limpeza como os hooks de `estado.ts`. O botão do painel e o item "Configurações" chamam com `"config"`; "Abrir o Kontro" chama sem aba e mantém a aba em que estava. Os handles do menu ficam num `MenusDaBandeja { estado: MenuItem, pilula: CheckMenuItem }` gerenciado como state. Itens, nesta ordem: `estado` (desabilitado, com a segunda linha da dica, `modelo::resumo_do_estado`, da tarefa da dica: "Xbox Wireless Controller · 77%, com folga · ~2 h 10 min", "· no cabo" ou "· desconectado"), separador, "&Abrir o Kontro", "&Configurações", "&Ler a bateria agora", "Mostrar a &pílula" (`CheckMenuItem` marcado conforme `is_visible()` da sobreposição, acionando `atalho::alternar`, que vira `pub(crate)`), separador, "&Sair". `atualizar_bandeja` chama `set_text` no `estado` e `set_checked` na `pilula` no ciclo em que mudam; o Tauri 2.11 leva a chamada para a thread principal sozinho. O texto do passo Pronto vira: "O ícone fica na bandeja: um clique abre o painel rápido com a carga, e o botão direito abre o menu. Configurações traz esta janela de volta, já na aba certa, onde mora tudo o que você viu aqui e o resto."

**Pronto quando:** botão direito: primeira linha cinza com nome e carga, que muda quando a carga muda; separador; ações; Sair no fim; Alt com a letra sublinhada aciona. "Configurações", no menu ou no painel, abre direto na aba Configurações, com o painel já fechado; com a janela minimizada ela é restaurada e focada, e atrás de outra vem para a frente. O passo Pronto descreve exatamente o clique e o menu.

## T47. O cartão de conexão nasce na tela do jogo, e a pílula não senta na barra de tarefas

**Onde:** `src-tauri/src/janelas.rs` (`posicionar_aviso`, `palco`, `posicionar_sobreposicao`, `onde_a_sobreposicao_parou`), `src-tauri/src/orquestra.rs` (`mostrar_aviso`, `talvez_avisar`).

Jogo na tela 2, controle conecta: o cartão aparece no topo da tela 1, atrás do jogo em foco na outra. `posicionar_aviso` usa `janela.current_monitor()`, onde a janela oculta já estava, que é o primário desde a criação sem posição, e cai para `primary_monitor()`. Usa `monitor.position()/size()` em vez de `work_area()`, então com a barra no topo o cartão nasce embaixo dela. Fora do jogo, em "Sempre visível" com a posição padrão (1.0, 1.0), a pílula cobre relógio e bandeja, justamente onde o ícone do Kontro está: `palco()` usa `monitor.size()`, enquanto o painel já usa `monitor.work_area()` desde a tarefa 12 do `TAREFAS.md`.

`posicionar_aviso` passa a receber `cfg` e usar `monitor_da_sobreposicao(app, cfg)`, a mesma função da pílula, que respeita `overlay_monitor` fixado, cai no monitor em foco e por fim no primeiro; `mostrar_aviso` e `talvez_avisar` repassam o `cfg` que `reavaliar` já tem, e aviso e pílula nascem na mesma tela. O cartão é centrado na `work_area()`, com `y = area.y + MARGEM_DO_AVISO - SANGRIA_SUPERIOR_DO_AVISO`, e o tamanho é o `inner_size` fixo de 384×180, sem medir; `set_position` vem primeiro, e o `WM_DPICHANGED` que chega depois já redimensiona no lógico certo. `palco()` recebe `tela_cheia: bool` (de `tela::em_tela_cheia()`) e usa `work_area()` quando falso e `size()` quando verdadeiro; `posicionar_sobreposicao` e `onde_a_sobreposicao_parou` passam o mesmo valor, então a fração salva continua relativa ao mesmo palco. As contas seguem em pixel físico, como a tarefa «Posição em pixel físico» da tela cheia define.

**Pronto quando:** com o jogo em foco na tela 2, ligar o controle mostra o cartão centralizado no topo da tela 2; com a barra no topo ele fica abaixo dela, não atrás; com "Sempre visível" e posição no canto inferior, a pílula encosta na borda de cima da barra de tarefas e, ao entrar num jogo em tela cheia, desce até a borda da tela.

## T48. A bandeja nasce desenhada, e a dica diz de quem é, em que faixa e de quando

**Onde:** `src-tauri/src/lib.rs` (`montar_bandeja`, `atualizar_bandeja`, `iniciar_ciclo`), `src-tauri/src/bandeja.rs`, `src-tauri/src/modelo.rs`, `src-tauri/src/tempo.rs`.

Ao subir com o Windows há um espaço em branco na bandeja, com dica "Kontro", por um a vários segundos até a primeira varredura acabar: `TrayIconBuilder::with_id("kontro")` sai sem `.icon(...)`, e o primeiro `monitor.ciclo()` inclui enumeração PnP e Bluetooth. Depois, a dica é um `format!("{} - {} - {}")` com nome, `texto_da_carga` e `texto_da_ligacao`: não diz de que app é o ícone; desconectado lê "Xbox Wireless Controller - -- - desconectado"; com leitura da semana passada lê "77% - desconectado" como se fosse agora; com 18% e aviso em 20% não diz "baixa"; a autonomia, o número que a pessoa quer, não está lá. O DESIGN §8 pede texto explícito em todo estado. E `set_tooltip` roda todo ciclo mesmo sem mudança, reemitindo `NIM_MODIFY` com a dica aberta.

`montar_bandeja` recebe o estado inicial de `Compartilhado` e sai com `.icon(bandeja::desenhar(&estado, bandeja::tamanho_do_icone(), cfg.limiares()))`, o glifo apagado com risco, e `.tooltip("Kontro · procurando controle")`. Em `modelo.rs`, `EstadoDoControle::faixa(&self, limiares) -> &'static str` com os mesmos ramos que `bandeja::desenhar` usa para a cor: "com folga", "carga baixa", "carga crítica". `texto_da_carga` deixa de devolver "--" (`_ => String::new()`; `diagnostico.rs:167`, o outro leitor, já escreve "nenhuma" quando vem vazio). `tempo::quando(ms)` devolve "hoje às 02:15", "ontem às 02:15" ou "03/09 às 02:15", com as mesmas regras do `quandoLeu` do front. `modelo::resumo_do_estado(estado, limiares)` monta a segunda linha: com número "77%, carga baixa · Bluetooth · ~2 h 10 min" (a autonomia só quando é estimativa de tempo); no cabo "No cabo, carregando" ou "No cabo"; ligado sem número "sem leitura · Bluetooth"; ligado com `leitura_antiga`, prefixo "leitura antiga · "; desligado "Desligado · 77% ontem às 21:34", ou só "Desligado" sem número. `bandeja::dica(estado)` é "Kontro · {nome}", quebra de linha, e o resumo. `iniciar_ciclo` guarda `ultima_dica` ao lado de `ultimo_icone` e só chama `set_tooltip` quando o texto muda; o texto continua refeito a cada ciclo, como a tarefa 14 do `TAREFAS.md` exige. Teste garantindo `dica().chars().count() <= 127` para os estados de demonstração, porque o `tray-icon` corta em 128 u16.

**Pronto quando:** no instante em que o ícone entra na bandeja ele já é o glifo apagado com risco, nunca um espaço vazio; a dica começa com "Kontro", diz "carga baixa" com 18% e aviso em 20%, nunca mostra "--", e desligado diz quando foi a última leitura; com o mouse parado 5 s sobre o ícone a dica não pisca.

## T49. A janela principal volta onde estava

**Onde:** `src-tauri/src/janelas.rs` (`criar_principal`), `src-tauri/src/lib.rs` (`on_window_event`, `RunEvent::Exit`, `Compartilhado`), `src-tauri/src/caminhos.rs`.

Quem usa o app na tela 2 abre pelo painel e ele surge no centro da tela 1, no tamanho padrão, toda vez: `.center()` centra no primário, nada guarda posição nem tamanho, e nada escuta `Moved`/`Resized`.

Posição e tamanho vão para um `janela.json` próprio — `{ x, y, largura, altura }` em lógico — e não para `Settings`: lá o campo entraria em `CAMPOS_DA_CONFIG` e no `config.gerada.ts` que o front nunca leria, e o teste `a_descricao_da_config_cobre_exatamente_os_campos_gravados` obriga a isso. `Moved` e `Resized` da principal atualizam `Compartilhado.geometria_da_principal` em memória; o arquivo é gravado no `CloseRequested` que esconde e no `RunEvent::Exit`, que já grava o histórico — sem debounce, porque não há escrita durante o arrasto. Na criação, se há valor e `monitor_from_point(x + 24, y + 24)` acha monitor, `.position(x, y).inner_size(largura, altura)` respeitando o mínimo 720×520; senão, centrar na `work_area()` de `monitor_do_cursor`, que é onde a pessoa está.

**Pronto quando:** fechar a janela num canto da tela 2 e reabrir traz ela no mesmo lugar e tamanho; instalação nova abre centrada na tela onde está o cursor.

## T50. O atalho da pílula vence o modo Desligada

**Onde:** `src-tauri/src/orquestra.rs` (`sobreposicao`).

Com a pílula em Desligada e os atalhos ligados, Ctrl+Shift+K não mostra nada e não avisa nada. `alternar` grava `sobreposicao_a_mao = Some(true)`, mas o ramo `Some(escolha) => ligada && escolha`, e `ligada` é falso com `OverlayMode::Desligada`.

O ramo vira `Some(escolha) => escolha`: um atalho é ação explícita e vence o modo. `salvar_configuracoes` já zera `sobreposicao_a_mao` quando o modo muda, então trocar para Desligada continua escondendo; e sem atalho ela segue nunca aparecendo sozinha.

**Pronto quando:** com Desligada, o atalho mostra a pílula; apertar de novo esconde; sem o atalho ela não aparece.

## T51. O tamanho do ícone sai do DPI da barra, não do sistema

**Onde:** `src-tauri/src/bandeja.rs` (`tamanho_do_icone`), `src-tauri/Cargo.toml`.

Primário a 100% e barra de tarefas no monitor a 150%: o Windows pede 24 px e recebe 16, esticado e borrado — o contrário do que o DESIGN §3 exige. `GetSystemMetrics(SM_CXSMICON)` responde pelo DPI do sistema num processo per-monitor.

`FindWindowW("Shell_TrayWnd")` → `GetDpiForWindow` → `GetSystemMetricsForDpi(SM_CXSMICON, dpi)`, com o valor atual como fallback quando a barra não é encontrada; entra a feature `Win32_UI_HiDpi` no `Cargo.toml`. O tamanho já está na assinatura de redesenho, então mover a barra de monitor repinta sozinho no ciclo seguinte.

**Pronto quando:** com a barra no monitor a 150%, `tamanho_do_icone()` devolve 24 e o anel fica nítido ao lado dos ícones do sistema.

---

# Movimento 9 — Sistema de design e temas

A T15 e a T16 deram ao Kontro seis temas em cima de tokens, e o `--realce` separou a cor de interface da cor de carga. O que ficou para trás é o que só aparece medindo: o `--text-tertiary` não chega a 4,5:1 em nenhum tema escuro, metade das folhas ainda pinta interface de verde, o Brasa não tem o laranja que a amostra vende, o anel gira em loop no cabo, e o tema "Do Windows" demora até 30 s para perceber que o Windows mudou. Junto vem a dívida de escala: dezessete tamanhos de fonte, quatro receitas de rótulo de seção, botão estilizado quatro vezes com quatro alturas, espaços e raios fora da escala do `DESIGN.md`. Este movimento fecha essas contas uma a uma — cada valor novo entra com a medida que o justifica, e o `DESIGN.md` passa a registrar a conta, não só o hex.

## T52. O texto terciário passa no contraste que o DESIGN.md exige, e o botão de destaque no Dia também

**Onde:** `src/estilo/tokens.css` (linhas 10, 69, 86, 102, 110, 126 e o bloco `body[data-janela="sobreposicao"]`), `DESIGN.md` §2 e §8.

Os rótulos de seção, o "lido ontem às 02:15" em mono, a descrição das linhas de Configurações, o estado do controle na lista, o "ontem, 21:14 · 2 h 40 min" das sessões, o "de jogo em 30 dias" e os eixos do gráfico saem em `--text-tertiary`, justamente o texto menor. Calculado contra os três chãos em que o token aparece: no Noite `#6b757d` dá 3,96:1 sobre `--surface` e 3,72:1 sobre `--surface-alt` (sessão escolhida, hover); Preto 4,24 e 3,79; Ardósia 4,11 e 3,66; Brasa 4,60 e 4,24; Dia `#67727f` dá 4,89 sobre branco, 4,32 sobre `--ink`, onde ficam os `h2`, e 4,01 sobre `--surface-alt`. O §8 exige 4,5:1, e nenhum tema escuro passa sobre `--surface-alt`. No Dia há um segundo furo: o hover de "Salvar cartão", "Atualizar agora" e "Avançar" põe `--ink #eef1f5` sobre `--realce #0a7f68`, 4,36:1.

Sobe o token, não o tamanho. Valores escolhidos pelo pior chão e conferidos: Noite e Preto `#7c868f` (5,03 sobre `--surface` e 4,72 sobre `--surface-alt` no Noite; 5,37 e 4,81 no Preto), Ardósia `#7d8d9e` (5,53 / 5,10 / 4,55), Brasa `#948578` (5,45 / 5,20 / 4,80), Dia `#5b6673` (5,16 sobre `--ink`, 5,84 sobre `--surface`, 4,79 sobre `--surface-alt`). O `--realce` do Dia vira `#0a6e5b`: `--ink` sobre ele dá 5,46, e como texto sobre `--surface` 6,19. O bloco fixo de `body[data-janela="sobreposicao"]` acompanha o Noite. Cada valor segue mais apagado que o `--text-secondary` do próprio tema, então a hierarquia não muda. O `DESIGN.md` §2 troca `TextTertiary #6B757D` por `#7C868F` e ganha, abaixo da tabela, a conta por tema e por chão: token novo de texto não entra sem ela.

**Pronto quando:** o medidor de contraste do DevTools do WebView2 em `.linha .descricao`, `.cartao .rodape`, `.sessao-quando` e `.diario-rotulo` dá ≥ 4,5:1 nos seis temas, inclusive numa sessão escolhida, e `--text-tertiary` sobre `--ink`, `--surface` e `--surface-alt` passa de 4,5:1 nas cinco paletas; o hover de "Salvar cartão" no Dia dá ≥ 4,5:1; na captura, "CARGA", "INICIALIZAÇÃO" e "lido ontem às…" continuam lendo como terciários, mais apagados que a descrição ao lado; o DESIGN.md diz o mesmo hex que o `tokens.css`.

## T53. Interface é --realce; verde só onde é carga

**Onde:** `src/telas/passos.css:33`, `src/telas/principal.css:293, 316, 321, 359`, `src/componentes/lista.css:56`, `src/telas/sobreposicao.css:84, 106-107`, `src/estilo/tokens.css:124`, `DESIGN.md` §7.

No Ardósia a aba ativa, a chave ligada e o botão primário são azuis, mas o ponto do passo a passo, a mini-pílula solta, a borda tracejada e o texto "pressione a combinação" da captura de atalho, o trilho de download da versão nova e a borda do campo de renomear continuam verdes ou teal. O modo solta mistura três cores para dizer a mesma coisa: contorno da pílula `rgba(53, 215, 168, 0.6)` cravado, hover do cadeado em `--accent-green`, mini-pílula da Configurações em `--accent-green` com borda em `--realce`. O bloco `body[data-janela="sobreposicao"]` fixa a paleta escura mas não fixa `--realce`, e como `App.tsx` aplica `data-tema` em toda janela, no Dia a pílula herdaria `#0a6e5b` sobre fundo escuro; por isso o CSS cravou o hex.

Os sete pontos passam a `var(--realce)`; `.pedindo` fica em `--realce` com peso 600. O bloco da sobreposição em `tokens.css` ganha `--realce: #35d7a8`, pela regra da T16: quem tem fundo próprio tem paleta própria. Em `sobreposicao.css`, o contorno de `.solta .pilula` vira `inset 0 0 0 1px color-mix(in srgb, var(--realce) 60%, transparent)` e `.prender:hover` vira `border-color: var(--realce); color: var(--realce)`. Verde fica só onde é carga: anel, heatmap, faixas e área do gráfico, ponto de saúde. A barra do ranking do Diário fica com a tarefa do ranking. No `DESIGN.md` §7, o "traço de 20px em AccentGreen" do passo a passo e o "trilho de 3px em AccentGreen" da versão nova passam a dizer `--realce`.

**Pronto quando:** no tema Noite, contorno da pílula solta, hover do cadeado e mini-pílula têm a mesma cor; no Ardósia, capturas de Resumo, Configurações, Passos e da pílula solta não mostram verde nem teal fora de anel, gráfico, heatmap e ponto de saúde; grep por `accent-green` e `accent-teal` fora de `tokens.css`, `anel.css`, `historico.css`, `diario.css` e `saude.css` não devolve nada.

## T54. O seletor de tema mostra o que cada tema é

**Onde:** `src/estilo/tokens.css:77-91`, `src/telas/principal.css:419-494`, `src/telas/Configuracoes.tsx:53-60, 222-236`, `src/componentes/Controles.tsx` (`Linha`).

São seis caixinhas de 40×28 com um pingo no canto e o nome só no `title`: para saber que "Ardósia" está ativo é preciso pousar o mouse e esperar o tooltip. Preto e Noite são dois retângulos quase iguais, Ardósia e Brasa só se distinguem pelo pingo. O pingo mente: o do Brasa é um laranja `#d98a4a` que não existe em lugar nenhum além dele, porque o tema não redefine `--realce` e aba ativa, chave e botão primário ficam teal frio sobre o chão marrom; o do Preto mostra `#5fe083`, verde de carga, e o tema usa teal. O `principal.css` duplica à mão os hex de chão e destaque dos seis temas. A descrição da linha é fixa, e o grupo usa `aria-pressed` em seis botões independentes, então o leitor de tela não sabe que é uma escolha única.

O Brasa ganha `--realce: #e0925a` em `tokens.css` (7,44:1 como texto sobre `--surface #18120f`; `--ink` sobre ele 7,81:1 no hover do botão primário). As amostras deixam de ter hex no CSS: `TEMAS` em `Configuracoes.tsx` ganha `chao`, `realce` e `texto` por tema, aplicados por `style` como `--amostra-chao`, `--amostra-cor` e `--amostra-texto`; o "Do Windows" usa `linear-gradient(115deg, #0b0e11 0 50%, #eef1f5 50% 100%)`, o Preto passa a `#35d7a8`, e o texto é o `--text-primary` de cada tema (noite e preto `#e8ecef`, ardósia `#e7edf4`, brasa `#f2ebe5`, dia `#0f151b`), desenhado como um traço de 14×2px ao lado do pingo para mostrar o contraste. Cada amostra vira coluna: retângulo de 48×32 e, embaixo, o nome em 12px `--text-tertiary`, `--text-primary` quando escolhida. `.temas` fica `flex-wrap: wrap; gap: var(--space-3)`; `Linha` ganha a prop `classe`, e `.linha.tema` usa `grid-template-columns: 1fr` para o controle descer para baixo da descrição e as seis caberem na janela mínima de 720px. `.temas` recebe `role="radiogroup"` e `aria-label="Tema"`; cada amostra `role="radio"`, `aria-checked={cfg.Theme === t.id}` e `tabIndex` 0 só na escolhida; ArrowLeft e ArrowRight trocam e gravam o tema. A descrição vira `${rotuloDoTema} · o chão da janela. As cores de carga não mudam.`.

**Pronto quando:** no Brasa a aba ativa, a chave ligada e "Salvar cartão" são laranja; sem passar o mouse, a linha diz qual tema está ativo e cada amostra tem o nome embaixo; a cor do pingo de cada amostra é a que a aba ativa assume ao escolher aquele tema, conferido nos seis; Tab entra nos temas uma vez, as setas trocam o tema na hora, e o Narrador lê "Tema, grupo de opções, Ardósia, selecionado".

## T55. Um botão só, um bloco de leitura só

**Onde:** novo `src/componentes/botao.css` e `src/componentes/leitura.css`, novo `src/componentes/Leitura.tsx`, `src/telas/principal.css:155-186, 208-261`, `src/telas/painel.css:65-85, 95-106`, `src/componentes/lista.css:110-121`, `src/telas/passos.css:171-174`, `src/telas/Painel.tsx:103-109`, `src/telas/Resumo.tsx:53-59`, `DESIGN.md` §5.

O botão fantasma tem padding 6/12 no Resumo, 7/14 no painel e 4/10 na confirmação da lista — três alturas para um botão só, porque `.ciclo` mora em `principal.css` sob `body[data-janela="principal"]` e o painel não pode usá-lo. O bloco "nome / detalhe / lido às" é o mesmo JSX em `Painel.tsx` e `Resumo.tsx`, com o detalhe a 4px do nome num e 6px no outro, o rodapé a 6 e 8px, em 11 e 11,5px. E o botão quase não existe: "Atualizar" e "Minimizar" são um retângulo transparente com borda `--stroke-strong`, 1,45:1 contra `--surface` no Noite (1,49 Brasa, 1,65 Preto, 1,69 Ardósia, 1,82 Dia); a chave desligada é `--surface-alt` com a mesma borda.

`botao.css` tem raiz `.botao` e variantes `.destaque`, `.perigo`, `.fantasma`, `.miudo`: `min-height: 32px`, `padding: 0 var(--space-3)`, raio `--radius-field`, `background: color-mix(in srgb, var(--text-primary) 5%, transparent)`, `border: 1px solid color-mix(in srgb, var(--text-primary) 12%, transparent)`, hover a 8%, `:disabled` com opacidade 0,6 — é o botão do WinUI, preenchimento sutil mais borda. `.miudo` tem `min-height: 28px` e `padding: 0 var(--space-2)`. `.ciclo` e `.painel .acoes button` viram `.botao` em todo JSX; `.ciclo.perigo` da lista vira `.botao.perigo.miudo`. `leitura.css` tem raiz `.leitura` com `.dispositivo` 16px/600, `.detalhe` `margin-top: var(--space-1)` 13px `--text-secondary` e `.rodape` `margin-top: var(--space-2)` mono 11px `--text-tertiary`; `<Leitura estado={…}/>` substitui o bloco nos dois arquivos. A `.chave` desligada ganha `border-color: var(--text-tertiary)`, que depois da tarefa de contraste já passa de 3:1 contra a linha. O `DESIGN.md` §5 registra que controle tem preenchimento sutil e cartão não.

**Pronto quando:** capturas do painel e do Resumo lado a lado mostram o botão com a mesma altura em px e o bloco de leitura com os mesmos espaçamentos; "Atualizar" se distingue do cartão sem hover; a chave desligada tem contorno ≥3:1 contra a linha, medido; `principal.css` e `painel.css` não definem mais botão nem leitura.

## T56. Uma escala de fonte, um rótulo de seção

**Onde:** `src/estilo/tokens.css`, `src/telas/principal.css`, `src/telas/painel.css`, `src/telas/aviso.css`, `src/telas/passos.css`, `src/telas/diario.css`, `src/componentes/lista.css`, `src/componentes/historico.css`, `src/componentes/sessoes.css`, `src/componentes/saude.css`, `src/telas/Painel.tsx`, `src/telas/Resumo.tsx`, `DESIGN.md` §4.

O CSS usa 34, 26, 22, 21, 20, 16, 15, 14, 13,5, 13, 12,5, 12, 11,5, 11, 10,5, 10 e 9px para uma escala que o DESIGN §4 define com cinco: 34, 22, 16, 14, 12, mais Mono 12. O mesmo papel muda de tamanho conforme a tela: o nome do controle é 15px no flyout e no aviso e 16 no Resumo; a legenda de leitura 11,5 no flyout, 11 no cartão e 11,5 na lista; a porcentagem do flyout 26 onde o §4 pede Display 34; o título da página 21 e o do passo a passo 22; as abas 13,5 e a marca 12,5. Meio pixel em Segoe UI Variable rende com métrica fracionária e fica borrado ao lado de texto inteiro: "ontem, 21:14" (11,5), "lido ontem às 02:15" (11) e "12% a 100%" (11,5) parecem três pesos. Os eixos do gráfico e "zera 23:40" saem em 9 e 10px, abaixo de qualquer texto do Windows 11. O rótulo de seção tem quatro receitas: "CARGA", "SEUS CONTROLES" e "ÚLTIMAS SESSÕES" em Consolas 10,5px (`historico.css:20-26`, `lista.css:7-14`, `sessoes.css:7-14`), que o §4 reserva para número medido e timestamp; "INICIALIZAÇÃO" e "QUEM COME MAIS BATERIA" em Segoe 11px com 0,12em; os `h3` das notas em 11px com 0,08em. E no Diário a receita errada vence: `body[data-janela="principal"] .pagina h2` (0,2,2) bate `.diario-titulo` (0,2,1), e o `h2` dentro do cartão do ranking herda `margin: 24px 0 8px`, pensado para `h2` entre cartões.

Em `tokens.css`, a escala do §4: `--fs-display: 34px`, `--fs-title: 22px`, `--fs-subtitle: 16px`, `--fs-body: 14px`, `--fs-caption: 12px`, `--fs-mono: 12px` e `--rastreio-rotulo: 0.12em`. Mapa: 9, 10, 10,5, 11, 11,5 e 12,5 vão para `--fs-caption`; 13 e 13,5 para `--fs-body`; 15 para `--fs-subtitle`; 20 e 21 para `--fs-title`; o 26 da porcentagem no anel de 96 vai para `--fs-display`, e para "100%" caber `Painel.tsx` e `Resumo.tsx` baixam a `espessura` do anel de 38 para 30, o que leva o miolo de 82 para 85 px, em vez de baixar a fonte. O rótulo de seção passa a uma receita só, `--fs-caption`, peso 600, `--rastreio-rotulo`, caixa alta, `--text-tertiary`, família `--font-text`, em `.pagina > h2`, `.diario-titulo`, `.lista-titulo`, `.historico-titulo`, `.sessoes-titulo` e `.notas h3`; `.lista-titulo` não vira `<h2>` porque a lista também vive no flyout, e `.historico-titulo.livre` continua sem caixa alta quando o título é o nome da sessão. O seletor de `principal.css:113` vira `.pagina > h2`, para o `h2` dentro de cartão não herdar a margem, e `.diario-titulo` fica com `margin: 0 0 var(--space-3)`. Os rótulos dos eixos já entram em 12px na tarefa do gráfico em pixel. O `DESIGN.md` §4 ganha a linha "Rótulo · 12 / SemiBold, caixa alta, 0.12em, TextTertiary · título de seção".

**Pronto quando:** `grep -o 'font-size: *[0-9.]*px' src/**/*.css` não devolve nada fora de `tokens.css`, e nenhum texto da janela principal, do painel ou do aviso tem `font-size` computado abaixo de 12px; o nome do controle tem o mesmo tamanho no Resumo, no painel e no aviso; "100%" cabe inteiro no anel do flyout e do Resumo; "CARGA", "SEUS CONTROLES", "QUEM COME MAIS BATERIA" e "INICIALIZAÇÃO" têm a mesma fonte, tamanho e rastreio lado a lado; o cartão do ranking tem 16px acima do rótulo, iguais ao padding.

## T57. Botão pressionado, curva em toda transição, cursor de app

**Onde:** `src/estilo/base.css:49-54`, `src/componentes/botao.css`, `src/telas/principal.css`, `src/telas/painel.css`, `src/telas/passos.css:24`, `src/componentes/lista.css`, `src/componentes/sessoes.css:100-101`, `src/componentes/historico.css:47`, `src/componentes/saude.css:57`, `DESIGN.md` §6.

Clicar em "Atualizar", numa aba, numa chave, numa sessão, em "Esquecer" ou numa amostra de tema não dá resposta entre o mousedown e o mouseup: o único `:active` do projeto é o cursor `grabbing` da pílula, embora o §6 fale em 120 ms para hover e pressed. A chave não tem nem `:hover`. Mais de dez transições usam `--motion-*` sem `var(--curva)` e caem no `ease` do navegador; `saude.css:57` escreve `240ms ease-out` por extenso. O cursor muda sem regra: `base.css` põe `cursor: pointer` em todo botão, `.sessao` e `.faixa` voltam para `default`, `.ciclo`, `.esquecer` e `.item-nome` herdam a mão; o Windows 11 usa seta em botão e mão só em link. E o valor atual de um ajuste sai em `--text-secondary`, a cor de descrição, e parece placeholder.

Em `botao.css` e em `.aba`, `.chave`, `.amostra-de-tema`, `.faixa`, `.sessao`, `.esquecer`, `.captura` e `.fechar` entra `:active:not(:disabled) { background: color-mix(in srgb, var(--text-primary) 4%, transparent); color: var(--text-secondary) }`, o pressed do WinUI, preenchimento e texto mais fracos; `.botao.perigo:active` usa `filter: brightness(0.9)`. A chave ganha resposta própria: `.chave:hover .bolinha { transform: scale(1.12) }`, na ligada `translateX(18px) scale(1.12)`; `.chave:active .bolinha { width: 20px }`, na ligada `translateX(14px)` para não sair do trilho. O botão que mostra o valor de um ajuste pinta o texto em `--text-primary`. Toda `transition` ganha `var(--curva)` depois da duração; `saude.css:57` vira `width var(--motion-slow) var(--curva)`. `cursor: pointer` sai de `base.css:53`, junto com as reversões de `sessoes.css:100`, `historico.css:47` e os `cursor: default` redundantes de `.ciclo:disabled` e `.captura:disabled`; a mão fica só no `grab` da `.sobreposicao.solta`.

**Pronto quando:** segurar o mouse em qualquer botão da janela principal e do painel muda o preenchimento antes de soltar e volta em 120 ms; passar o mouse na chave engorda a bolinha; `grep -n 'transition' src/**/*.css | grep -v curva` não devolve linha; o cursor é a seta sobre todo botão.

## T58. Espaço e raio só da escala, e uma altura de controle

**Onde:** `src/estilo/tokens.css`, `src/telas/principal.css:68, 78, 182, 274, 299-300, 324-328, 349-358`, `src/telas/painel.css:17-25`, `src/telas/aviso.css:2, 7-8`, `src/componentes/lista.css:20, 54-55`, `src/componentes/sessoes.css:24-30`, `src/componentes/historico.css:36, 40`, `src/telas/passos.css:14, 20`, `src/componentes/Controles.tsx:57`, `src/componentes/saude.css:47-55`.

A aba tem padding 9/12, o rodapé margem 6, o trilho gap 2, o item da lista 7/8, a faixa do gráfico 3/9, o aviso padding 28/32/56 e gap 14; raios de 5px (mini-tela, tecla), 6px (fechar do painel, ícone da sessão), 4px (renomear), 11px (chave), 2 e 3px nos trilhos. Botões de 30px ao lado de chaves de 22px e ícones de 28px, sem uma altura de linha comum. Os tokens existem, mas metade das regras foi escrita com número solto e não há tamanho de controle definido.

Em `tokens.css` entram `--altura-controle: 32px` (o botão da tarefa do `botao.css` já a usa), `--radius-control: 4px` (o raio de controle do Windows 11: tecla, mini-tela, campo de renomear) e `--radius-trilho: 2px` (só para trilhos de 3px). Mapa: `.aba` `padding: var(--space-2) var(--space-3)`; `.rodape` `margin-top: var(--space-2)`; `.trilho` `gap: var(--space-1)`; `.item` `padding: var(--space-2)`; `.faixa` `padding: var(--space-1) var(--space-2)`; `.aviso` `padding: var(--space-5) var(--space-6) 56px` e `gap: var(--space-3)`; `.fechar` e `.sessao-icone` raio `--radius-control`; `.tecla`, `.mini-tela` e `.renomear` idem, com o `borderRadius: 4 + escala` inline da `MiniTela` trocado pelo token; `.captura` e `.renomear` com `min-height: var(--altura-controle)`. A chave fica pill (`999px`) em vez de `11px`.

**Pronto quando:** `grep -oE '(padding|gap|margin[^:]*|border-radius): *[0-9]' src/**/*.css` só devolve valores da escala ou `var()`; botão, campo de renomear e captura de atalho têm 32px de altura na captura.

## T59. O flyout cabe na tela e fecha os cantos, e as sombras viram token do tamanho da janela

**Onde:** `src-tauri/src/lib.rs` (`ajustar_altura_do_painel`), `src/telas/painel.css`, `src/telas/Painel.tsx`, `src/telas/aviso.css`, `src/telas/sobreposicao.css`, `src/estilo/tokens.css`, `DESIGN.md` §5 e §7.

Com vários controles e histórico, a altura do flyout vai até 900px; numa tela de 768 o topo fica fora do monitor e nada rola, porque `clamp(200, 900)` não olha a área de trabalho e `.painel` não tem `max-height` nem `overflow`. O DWM arredonda a janela em 8px (`DWMWCP_ROUND`, `janelas.rs:79-97`), mas o `div.painel` é quadrado: a borda de 1px é cortada na diagonal e some nos quatro cantos. Sem material, a janela tem exatamente o tamanho do `div`, e a sombra CSS de 18/32px é cortada; a que aparece é sempre a nativa do `.shadow(true)`. O X a 10px do canto, que nenhum flyout do Windows 11 tem, recebe o `.dispositivo` em `nowrap` correndo até a borda: "Xbox Wireless Controller" encosta nele. No aviso, sobre fundo claro, aparece um degrau reto nas laterais do cartão, a 32px da borda: `box-shadow: 0 18px 32px rgba(0,0,0,0.45)` é σ=16, e a 2σ, nos 32px de padding lateral, ainda carrega ~6% de preto, uns 15 níveis sobre branco, o mesmo defeito que o commit d76dc63 mediu e corrigiu na pílula. O cartão de ~70px usa `--radius-window` (16px), que o §5 reserva para janela, e `.linha` mede 12,5px. A pílula repete `0 2px 8px rgba(0,0,0,.6)` quatro vezes e crava `rgba(242,86,78,.35)`, que é `--red` com opacidade.

Flyout: `ajustar_altura_do_painel` limita a altura a `work_area.height - 24` do monitor do cursor, o mesmo que `posicionar_painel` já consulta. `painel.css`: `.painel { max-height: 100vh; overflow-y: auto; border-radius: var(--radius-sistema) }`, com `--radius-sistema: 8px` novo em `tokens.css`, o raio do DWM; `--radius-window` de 16 fica para o que não passa pelo DWM. `--sombra-painel` sai de `tokens.css`, porque a sombra do flyout é a do sistema. `Painel.tsx` remove o botão `.fechar` e `painel.css` as regras dele: Esc, perda de foco e o clique na bandeja já fecham. Aviso: `box-shadow: var(--sombra-aviso)`, com `--sombra-aviso: 0 8px 24px rgba(0, 0, 0, 0.45)`, σ=12 e 30px de cauda, que cabe nos 32 das laterais, nos 28 de cima (30 − 8) e nos 56 de baixo (30 + 8); `border-radius: var(--radius-card)`; `.linha` em 12px. Pílula: `--sombra-flutuante: 0 2px 8px rgba(0, 0, 0, 0.6)` nas quatro ocorrências, e `sobreposicao.css:42` vira `0 0 12px color-mix(in srgb, var(--red) 35%, transparent)`. O `DESIGN.md` §5 registra que o flyout usa o raio de 8 e a sombra do sistema e que o aviso usa `0 8 24`, e o §7 passa o raio do flyout de 16 para 8.

**Pronto quando:** numa tela 1366×768 o flyout nunca passa do topo e rola por dentro; zoom no canto mostra a borda de 1px acompanhando a curva; um nome longo nunca toca o canto superior direito; captura do aviso sobre fundo branco sem degrau reto em volta do cartão, com diferença de no máximo 1 nível na borda da janela; os cantos do aviso são os mesmos dos cartões do Resumo; `grep -rn 'rgba(0, 0, 0\|rgba(242' src/**/*.css` só devolve `tokens.css`.

---

# Movimento 10 — Textos e estados

Os textos do Kontro foram escritos conforme cada tela nasceu, e hoje o mesmo fato sai de um jeito em cada janela: o Resumo diz "Desconectado" para quem nunca pareou nada, o 77% de ontem tem o mesmo peso do 77% de agora, a dica da bandeja imprime "--", a mesma hora aparece em quatro formatos e a mesma ação tem três nomes. O que muda: cada estado ganha uma frase que diz o que ele é — vazio, procurando, sem leitura, leitura velha — escrita uma vez, no Rust quando a bandeja também usa e em `formato.ts` quando é só tela; os rótulos passam a nomear o que o controle faz; e uma falha aparece ao lado do botão que a causou, em português, com um jeito de tentar de novo.

## T60. O app para de dizer "Desconectado" enquanto procura e para quem nunca ligou um controle

**Onde:** `src-tauri/src/modelo.rs`, `src-tauri/src/lib.rs`, `src/estado.ts`, `src/formato.ts`, `src/telas/Resumo.tsx`, `src/telas/Painel.tsx`.

Abrir o app, ou o flyout, antes do primeiro ciclo terminar mostra título "Desconectado", segunda linha "Procurando controle", rodapé "sem leitura ainda" e anel cinza vazio, com o controle ligado ao lado, pela primeira descoberta mais a conexão GATT: segundos. Sem controle pareado, "Desconectado" com "Nenhum controle pareado". `Resumo.tsx:55` e `Painel.tsx:105` escolhem o título só por `via === "Desligado"`, e o Rust usa o campo `nome` como recado nesses dois estados (o placeholder de `lib.rs:106-108` e `monitor.rs:427`), sempre com `quantos_conhecidos: 0` e `lido_em: None`. "Desconectado" pressupõe um controle conhecido que saiu do ar; quem acabou de instalar lê que algo caiu antes de ter ligado qualquer coisa.

`Bruto` e `EstadoDoControle` ganham `procurando: bool` (falso por padrão, verdadeiro só no placeholder de `Compartilhado`, cujo recado vira "Procurando controle…", com reticências) e `titulo`, montado em `EstadoDoControle::montar`: "Procurando controle…" com `procurando`; `nome` quando `via != Desligado`; `nome` também quando `quantos_conhecidos == 0`, porque aí ele é o recado; "Desconectado" no resto. `Estado` em `estado.ts` recebe `titulo` e `procurando`, e as duas telas trocam o ternário por `estado.titulo`. Em `formato.ts`, `detalhe()` no ramo desligado devolve "Ligue um controle por Bluetooth ou pelo cabo" quando `quantosConhecidos === 0` e `nome` no resto; com `procurando`, `detalhe()` e `quandoLeu()` ficam vazios e o anel é o cinza sem arco que `corDoAnel` já devolve para `Desligado`.

**Pronto quando:** abrir o app com o controle ligado nunca mostra "Desconectado" antes do primeiro dado: mostra "Procurando controle…" e em seguida o nome e a carga; sem controle pareado mostra "Nenhum controle pareado / Ligue um controle por Bluetooth ou pelo cabo / sem leitura ainda"; com um controle conhecido desligado continua "Desconectado / Xbox Wireless Controller".

## T61. Leitura velha e leitura nenhuma escritas como o que são

**Onde:** `src/estado.ts`, `src/formato.ts`, `src/telas/Resumo.tsx`, `src/telas/Painel.tsx`, `src/telas/principal.css`, `src/telas/painel.css`, `src/telas/Configuracoes.tsx` (`Novidade`), `src/telas/Sobreposicao.tsx`, `src/telas/sobreposicao.css`.

Com o controle desligado desde ontem, o Resumo mostra "77%" grande dentro do anel, e a única pista de que o número é velho é o verbo do rodapé em 11px: "lido" em vez de "atualizado". `estado.leituraAntiga` só é lido em `quandoLeu()`; `Resumo.tsx:47`, `Painel.tsx:97` e a pílula imprimem o percentual sem olhar o campo. O passo 2 do tutorial promete "nada de número velho fingindo ser de agora", e o cartão principal faz exatamente isso. O rodapé inteiro, "lido ontem às", sai em Consolas, que o DESIGN §4 reserva para número e timestamp. Na pílula, a única tela vista em jogo, três estados que o modelo já mede ficam invisíveis: logo depois de ligar, `monitor.rs` semeia o registro com o último ponto do `history.json` e marca `leitura_antiga`, e a pílula desenha esse valor com anel na cor da faixa e número cheio; sem leitura (`precisao === "Nenhuma"` fora do cabo) ela imprime "--" em mono 16px ocupando os 46px de `min-width`; e no cabo `resumir()` devolve "cabo" tanto plugado quanto carregando, enquanto um controle que informa número no cabo mostra "77%" com anel verde, idêntico a estar na bateria.

Tela: Resumo e Painel põem a classe `antiga` em `.cartao.estado` e `.painel .topo` quando `estado.leituraAntiga`, com `.antiga .numero { color: var(--text-secondary) }` em `principal.css` e `painel.css`, e o rodapé abre com um ponto de 6px em `--gray` (`::before`, `margin-right: var(--space-2)`, o mesmo de `.saude .veredito::before`); o anel já sai cinza e sem halo pela tarefa do cabo. `quandoLeu()` passa a devolver `{ texto, hora }`: "última leitura ontem às" e "02:15" na antiga, "atualizado às" e "02:15" na viva, `{ texto: "sem leitura ainda", hora: null }` sem leitura. As duas telas renderizam `<div className="rodape">{texto} <span className="mono">{hora}</span></div>`; `.rodape` (`principal.css:181`, `painel.css:80`) volta para `--font-text` e só `.rodape .mono` fica em `--font-mono`. No cartão de versão nova, que usa o mesmo `.rodape`, o número da versão e a porcentagem do download ganham o `.mono`, as palavras não.

Pílula: `resumir()` devolve `null` sem leitura fora do cabo, e o `<span className="valor">` só renderiza quando há texto; a pílula encolhe para anel e glifo, e o Rust já redimensiona pela medida. A raiz ganha `antiga` quando `estado.leituraAntiga && !estado.girando`, com `.sobreposicao.antiga .valor { color: var(--text-secondary) }` e `.sobreposicao.antiga .anel .carga { opacity: 0.6 }`, estado desenhado, sem animação. Com `estado.via === "Cabo"`, o miolo do anel troca o glifo por um raio, SVG de 15px, `fill: var(--accent-teal)` quando `carregando` e só `stroke` em `var(--gray)` quando apenas plugado; o texto continua "cabo" sem número e "77%" com número. Sem `title` na pílula: presa, a janela ignora o cursor (`janelas.rs:140`).

**Pronto quando:** com o controle desligado desde ontem, o 77% do Resumo e do painel está em cinza secundário e o rodapé diz "última leitura ontem às 02:15", com só o "02:15" em Consolas; ligando o controle o número volta a `--text-primary` e o rodapé a "atualizado às HH:MM". Ao ligar, a pílula mostra o número esmaecido até a primeira leitura de verdade; sem leitura ela mostra só o anel cinza com o glifo, sem traço; plugar o cabo troca o glifo pelo raio em até 2 s e desplugar o devolve. Nenhuma janela exibe "--" nos cinco estados: desligado, procurando, no cabo, Bluetooth sem leitura e Bluetooth com número.

## T62. Erros em português, com acento e num tom só

**Onde:** `src-tauri/src/lib.rs`, `src-tauri/src/atualizacao.rs`, `src/telas/Configuracoes.tsx`, `src/telas/Diario.tsx`.

Falhar ao salvar o cartão do Diário mostra "a imagem veio ilegivel" ou "nao achei a pasta de downloads" (lib.rs:609 e 613, sem acento) ou o texto bruto do `io::Error` — "Access is denied. (os error 5)" — porque 614 e 617 fazem `map_err(|e| e.to_string())`. Diario.tsx:172 guarda `String(e)`, que para o `throw new Error("sem canvas")` vira "Error: sem canvas". Na atualização, `descrever()` já traduz as falhas conhecidas, mas o ramo `outro => outro.to_string()` (atualizacao.rs:141) deixa passar o inglês do plugin. E a mesma falha tem dois registros: o Rust diz "não foi possível falar com o GitHub", o TypeScript (Configuracoes.tsx:125 e 142) diz "não deu para consultar o repositório", e o título "Não foi possível verificar" repete "não foi possível" na linha de baixo.

lib.rs: "a imagem veio ilegível", "não achei a pasta Downloads", e `map_err(|_| "não deu para gravar em Downloads")` nas duas gravações. atualizacao.rs:141: "a atualização parou no meio", com o erro original em `eprintln!`. `descrever()` adota o tom que `textoDoDiagnostico` já usa: "não deu para falar com o GitHub". Configuracoes.tsx:125 e 142 usam essa mesma frase como fallback; 170 guarda `typeof e === "string" ? e : "não deu para baixar a versão nova"`. Os títulos de `tituloDaVersao` viram "Sem resposta do GitHub" e "A atualização não terminou". Diario.tsx:172: `typeof e === "string" ? e : "não deu para desenhar o cartão"`.

**Pronto quando:** nenhum texto exibido contém "os error", palavra em inglês ou vogal sem acento onde o português pede; título e descrição de uma falha não repetem a mesma expressão.

## T63. Uma hora, uma taxa, uma duração, formatadas num lugar só

**Onde:** `src/formato.ts`, `src/componentes/Historico.tsx`, `src/componentes/Sessoes.tsx`, `src/componentes/Saude.tsx`, `src/telas/Diario.tsx`, `src/telas/Configuracoes.tsx`.

O mesmo instante aparece como "03/09 às 21:14" no hover do gráfico (`momento`), "03/09, 21:14" na lista de sessões (`momentoRelativo`) e "lido em 03/09, às 21:14" no rodapé do cartão (`quandoLeu`) — três funções em formato.ts com três separadores, mais `toLocaleTimeString` e `toLocaleDateString` inline em Historico.tsx:275 e 290. A mesma taxa sai como "7,5 %/h" nas sessões, no ranking e na autonomia do Rust, e como "7,5% por hora" na Saúde e no rodapé do gráfico; `toFixed(1).replace(".", ",")` está copiado em seis arquivos. O rodapé da sessão diz "12 pontos em 160 min" enquanto a lista ao lado diz "2 h 40 min" para a mesma sessão. E a ponta da projeção escreve "zera 02:30" (Historico.tsx:206): lido às 22:30, parece que já passou.

formato.ts fica com `dia(ms)` → "hoje" / "ontem" / "amanhã" / "03/09" (`diasAtras` já devolve negativo para o futuro), `quando(ms)` → `${dia} às ${hora}`, `decimal(n)` → uma casa com vírgula, e `taxa(porHora)` → "7,5 %/h". `momento` e `momentoRelativo` saem; Historico.tsx:224 e Sessoes.tsx usam `quando`. Historico.tsx:275 e 290 usam `hora()` e `diaEMes()`. Historico.tsx:304 vira `${pontos} pontos em ${duracao(minutos)} · ${taxa(t)}`; Sessoes.tsx:100, Diario.tsx:128 e 307 usam `taxa()`; Saude.tsx:75-77 vira "7,5 %/h esta semana, contra 6,8 %/h antes."; `megabytes` em Configuracoes.tsx:527 usa `decimal()`. A projeção escreve "zera às 23:40" no mesmo dia e `zera ${quando(zeraEm)}` — "zera amanhã às 02:30" — quando cruza a meia-noite. monitor.rs:485 já escreve "consumo de 7,5 %/h" e fica como está.

**Pronto quando:** buscar `replace(".", ",")` em src/ devolve só formato.ts; a mesma sessão mostra a mesma duração na lista e no rodapé do gráfico; nenhuma tela escreve "% por hora"; uma projeção que cruza a meia-noite escreve "zera amanhã às 02:30".

## T64. O passo a passo mostra os atalhos que a pessoa tem, com as teclas desenhadas

**Onde:** `src/componentes/Teclas.tsx` (novo), `src/telas/Configuracoes.tsx` (`NOME_DA_TECLA`, `nomeDaTecla`, `Captura`), `src/telas/Passos.tsx` (passo 3, texto da pílula presa).

O passo 3 diz "Ctrl + Shift + M solta ela de novo a qualquer momento" num `<b>` cravado no JSX (`Passos.tsx:186`). Quem trocou o atalho em Configurações, desligou "Usar atalhos" ou teve a combinação recusada (`atalhos_recusados`) lê uma instrução que não funciona, e o botão "Rever" reabre o tutorial justamente para quem já mexeu nisso. O atalho de mostrar e esconder, `Ctrl+Shift+K` por padrão, o que se usa dentro do jogo, não aparece em nenhum dos seis passos. E `NOME_DA_TECLA`, `nomeDaTecla` e o `<kbd className="tecla">` vivem privados em `Configuracoes.tsx:12` e `669`.

`NOME_DA_TECLA` e `nomeDaTecla` saem de `Configuracoes.tsx` para `src/componentes/Teclas.tsx`, junto com `Teclas({ combinacao })`, que faz o `split("+")` e renderiza um `<kbd className="tecla">` por parte; a `Captura` passa a usá-lo. O `.tecla` de `principal.css` já vale para a janela principal inteira, então serve nos passos sem CSS novo. No passo 3, com a pílula presa, o parágrafo vira: "Está presa no ponto do desenho acima. `<Teclas combinacao={cfg.OverlayMoveShortcut} />` solta ela de novo, e `<Teclas combinacao={cfg.OverlayShortcut} />` esconde e traz de volta, sem sair do jogo." O `Passos` chama `useAtalhosRecusados()`: com `!cfg.OverlayShortcutEnabled` a frase dos atalhos vira "Os atalhos estão desligados; dá para ligar em Configurações > Atalhos, e o botão Soltar de lá solta ela de novo."; se a combinação de mover está em `recusados`, "O Windows recusou esse atalho; escolha outro em Configurações > Atalhos." O botão "Soltar de novo" continua nos três casos.

**Pronto quando:** trocar o atalho de mover em Configurações e clicar em Rever mostra a combinação nova no passo 3, em teclas iguais às da tela de Atalhos; os dois atalhos aparecem no passo 3; com "Usar atalhos" desligado o passo diz isso em vez de mostrar teclas.

---

# Movimento 11 — Resumo

O Resumo é a primeira tela do app e a que mais promete: o anel com a carga, o gráfico de sete dias com a projeção até o zero, a saúde da bateria, as últimas sessões. Hoje ele entrega isso com uma série de ruídos que a pessoa sente sem saber nomear: um anel que gira para sempre no cabo, um 77% de ontem com o mesmo peso de uma leitura ao vivo, um gráfico que estica as letras conforme a largura da janela, a resposta do app ("quanto ainda dá pra jogar") em texto cinza de 13px, um botão Atualizar que não responde, e três folhas de estilo escolhendo tamanhos a olho. Este movimento não acrescenta dado nenhum: pega o que o Rust já mede e põe cada número no peso que ele merece, tira todo movimento que não vem de dado, e alinha o Resumo à escala que o DESIGN.md e as Configurações já usam.

## T65. Sem uso na semana, o gráfico ainda deixa chegar aos 30 dias, e o vazio diz o que vai acontecer

**Onde:** `src/componentes/Historico.tsx`, `src/componentes/historico.css`.

Quem ficou dez dias sem jogar abre o Resumo e vê só "sem histórico nesta janela", em minúscula, 12px, cinza, e nenhum botão. O retorno antecipado em `Historico.tsx:63-65` acontece antes de renderizar `.historico-topo` (`:89-112`), então os botões 7 dias / 30 dias somem junto com o gráfico: os 30 dias têm dado, mas não há como chegar lá, e o cartão parece vazio e quebrado. Com histórico nenhum é a mesma linha solta, sem dizer que janela é essa nem que ela se preenche sozinha; no flyout, a mesma frase para o usuário novo.

O `if (amostras.length < 2)` vai para dentro do JSX e substitui só o `<svg>` e o rodapé por um bloco `.historico .vazio` com a altura do gráfico (150px, 64 no compacto), mantendo o topo com os botões. O texto, em 14px `--text-secondary` e com inicial maiúscula, distingue os casos: série com amostras mas nenhuma na faixa, "Nada nos últimos 7 dias. Veja os 30 dias."; série vazia, "O histórico começa na primeira leitura com o controle ligado."; no compacto, "Sem leituras esta semana". Fora de uma sessão aberta e fora do compacto, quando os 7 dias estão vazios e os 30 não, `dias` vai para 30 sozinho na montagem.

**Pronto quando:** com um `history.json` cuja última amostra tem 10 dias, o Resumo abre já em 30 dias mostrando a curva; os botões 7/30 continuam visíveis em qualquer estado vazio; um perfil sem `history.json` mostra, no Resumo e no flyout, uma frase que diz o que é o histórico e que ele se preenche sozinho.

## T66. A autonomia vira o segundo maior número da tela

**Onde:** `src/telas/Resumo.tsx`, `src/telas/principal.css`, `src/formato.ts`, `src/componentes/Saude.tsx`, `saude.css`, `src-tauri/src/modelo.rs`, `src-tauri/src/monitor.rs`, `src/estado.ts`.

O que a landing page promete e o que a pessoa abre o app para saber, "~35 h de jogo", aparece como `.detalhe` em 13px `--text-secondary` embaixo do nome do controle, porque `detalhe()` (`formato.ts:58`) devolve autonomia como texto genérico e `Resumo.tsx:57` o joga na mesma classe de "conectado Bluetooth". "Uma carga cheia dura 46 h 20 min" fica a 12px no fim do cartão do gráfico, abaixo de dois divisores, por acidente de onde o dado nasce (`Saude.tsx:23-28`, `.carga-cheia` em `saude.css:60-72`). O Diário tem um número-herói de 34px; o Resumo, que é a primeira tela, não tem nenhum.

No cartão de estado, abaixo de `.leitura`, entra uma linha `.indicadores` com até três blocos, `gap: var(--space-5)`, `margin-top: var(--space-3)`: número em `--font-display` 22px 600 `letter-spacing: -0.5px` e rótulo em Caption 12 `--text-tertiary` embaixo. Os blocos: `~{duracao(autonomiaMinutos)}` com rótulo "restam de jogo", só quando `autonomiaMinutos` não é null; `duracao(saude.cargaCheiaMinutos)` com rótulo "por carga cheia"; e `{consumo} %/h` com rótulo "de consumo". A taxa hoje só chega ao front dentro de texto, então `Bruto` e `EstadoDoControle` (`modelo.rs`) ganham `consumo_por_hora: Option<f64>`, preenchido em `monitor.rs` no mesmo ponto em que a `autonomia` já chama `historico.consumo_por_hora(chave)`, comparado em `igual_a` (a tarefa 35 do TAREFAS.md registra o que acontece quando um campo fica de fora) e exposto como `consumoPorHora` em `Estado`. Sem taxa, o bloco não existe, sem espaço vazio no lugar. `detalhe()` ganha um segundo parâmetro `comAutonomia = true`; o Resumo chama com `false` e volta a mostrar só a via, e o Painel, que precisa da autonomia em Body pelo §7, continua igual. `.carga-cheia` sai de `Saude.tsx` e de `saude.css`.

**Pronto quando:** abrindo o Resumo com o controle ligado por Bluetooth, o maior número da tela depois do percentual é a autonomia; com o controle desligado ou no cabo, os blocos que dependem de taxa não aparecem, e o cartão do gráfico termina na saúde sem a linha da carga cheia.

## T67. As faixas de limiar ficam visíveis e ganham número

**Onde:** `src/componentes/Historico.tsx`, `historico.css`.

A T3 pôs as faixas âmbar e vermelha no gráfico, mas a `opacity: 0.06` e `0.07` (`historico.css:97-105`) sobre `--surface` rende menos de 2% de diferença de luminância: na captura, abaixo de 20% há uma faixa marrom quase invisível e nenhuma indicação de que ali o app avisa. O eixo Y só marca `[0, 50, 100]` (`Historico.tsx:146`), então o limiar configurado não tem número em lugar nenhum do gráfico.

`historico.css` sobe para 0.10 (aviso) e 0.12 (crítico). `Historico.tsx` desenha, além de 0/50/100, um `rotulo-y` em `limiares.aviso` com `fill: var(--amber)` e outro em `limiares.critico` com `fill: var(--red)`, pulando o rótulo que ficaria a menos de 12px de outro; e na altura de cada limiar uma `<line>` de 1px, `stroke-dasharray: 2 4`, na cor da faixa a 35% de opacidade. No compacto entra só a linha, sem rótulo, como já acontece com o eixo. Isso completa o que a T3 prometeu: a linha entrando na faixa conta a história sozinha.

**Pronto quando:** mudar o aviso de 20 para 40 nas Configurações move o rótulo "40" âmbar e a linha tracejada na hora, e a faixa é distinguível do fundo nos seis temas.

## T68. O gráfico sabe que horas são

**Onde:** `src/componentes/Historico.tsx`, `historico.css`, `src/telas/Resumo.tsx`.

Dois sintomas com a mesma raiz. Com 77% e 46 h de carga cheia a autonomia passa de 30 h; o rótulo diz "zera 14:20" (`hora()`, `Historico.tsx:206`) e a pessoa não sabe se é hoje ou depois de amanhã. O eixo se estende até o zero sem teto (`fim = Math.max(agora, zeraEm)`, `:57`), comprimindo a semana em 85% da largura, e não há marca do "agora": a linha pontilhada é a única pista de onde o dado termina e a projeção começa. E `agora = Date.now()` é calculado no render (`:52`), então com a janela aberta a noite toda o eixo fica parado no horário do último render. As três `invoke` de `Resumo.tsx:22-26` dependem só de `[estado?.chave, estado?.percentual]`: desligar o controle grava o marcador de desligado no `history.json`, mas o gráfico não quebra o trecho nem some a projeção até o percentual mudar.

`agora` vira `useState`, atualizado por `setInterval` de 60 s enquanto montado, limpo no cleanup. O rótulo usa `momentoRelativo(zeraEm)` quando `zeraEm - agora > 12h` ("zera amanhã, 14:20") e "zera às 14:20" senão. Uma `<line className="agora">` em `x(agora)` de `M.topo` a `y(0)`, `stroke: var(--stroke-strong); stroke-dasharray: 2 3`, com o rótulo "agora" em `rotulo-x` acima do eixo. A projeção ocupa no máximo um quarto do gráfico: `fim = Math.min(zeraEm, agora + (agora - inicio) / 3)`; se `zeraEm` cair fora, a pontilhada é cortada na borda direita e o rótulo ganha "→" na ponta ("zera amanhã, 14:20 →"). Em `Resumo.tsx`, `estado?.via` e `estado?.lidoEm` entram nas dependências do `useEffect`.

**Pronto quando:** com autonomia de 30 h a semana ocupa pelo menos 75% do gráfico, existe uma linha "agora" separando medido de projetado e o rótulo diz o dia; desligar o controle quebra a curva na próxima leitura de estado, sem mudança de percentual; deixar a janela aberta uma hora move o eixo uma hora.

## T69. O gráfico anda por setas, o valor nasce colado ao ponto, e o leitor de tela sabe o que ele mostra

**Onde:** `src/componentes/Historico.tsx`, `src/componentes/historico.css`, `src/telas/Resumo.tsx`, `src/formato.ts`.

Passar o mouse num vão de dois dias sem uso desenha a mira e o ponto na amostra mais próxima no tempo, às vezes a 200px do cursor, porque `aoMover` (`Historico.tsx:75-85`) não tem limite de distância. O valor sai no rodapé, embaixo do gráfico e longe do ponto, e substitui o resumo ("12% a 100%, em 17 períodos"), que some enquanto se lê. A data é "08/09 às 21:14", absoluta, enquanto a lista de sessões abaixo diz "ontem, 21:14". Com teclado não há como chegar a ponto nenhum: o `<svg>` só tem `onMouseMove`/`onMouseLeave`, sem `tabIndex`, `role` ou `aria-label`, e o leitor de tela o pula inteiro. Os botões "7 dias" e "30 dias" marcam a ativa só por classe e são anunciados como dois botões iguais. E o cartão de estado muda de 77% para 12% sem anúncio.

Depende do gráfico em pixel, porque a dica é HTML posicionada sobre o SVG. `aoMover` só prende quando `Math.abs(x(perto.t) - cursorX) <= 24`, senão `setSob(null)`. O SVG ganha em volta um `div.grafico-area` com `position: relative`, e dentro dele uma `<div className="dica" role="status" aria-live="polite">` em `left: x(sob.t)`, `top: y(sob.p) - 8`, `transform: translate(-50%, -100%)`, fundo `--surface-alt`, borda 1px `--stroke`, raio 8, padding 4px 8px, conteúdo em Mono 12: "77%" em `--text-primary` e `${quando(sob.t)} · ${via}` em `--text-tertiary`, com o `quando()` de `formato.ts`; quando `y(sob.p) < 40` a dica vai para baixo do ponto. O rodapé mantém o resumo fixo. O `<svg>` recebe `tabIndex={0}`, `role="img"`, `aria-label={janela ? resumoDaSessao(amostras) : resumo(amostras, trechos.length)}` e um `onKeyDown`: ArrowLeft e ArrowRight movem `sob` para a amostra anterior ou seguinte, Home e End para a primeira e a última, Escape limpa; `onBlur` limpa. Cada `.faixa` recebe `aria-pressed={dias === f.dias}`. Em `historico.css`, `.grafico { border-radius: 4px }` e `.grafico:focus-visible { outline: 2px solid var(--realce); outline-offset: 4px }`. Em `Resumo.tsx`, `aria-live="polite"` na `<section className="cartao estado">`.

**Pronto quando:** passando o mouse por um vão nada aparece; sobre a curva, a dica nasce colada ao ponto sem tapar a linha; Tab até o gráfico e setas para os lados fazem a mira e a dica percorrerem as amostras; o Narrador lê cada valor, lê o resumo quando não há ponto escolhido, diz "pressionado" na faixa ativa e anuncia a mudança de percentual sem mover o foco.

## T70. Com um controle só, o cartão de baixo some

**Onde:** `src/telas/Resumo.tsx`, `src/telas/principal.css`, `src/componentes/ListaDeControles.tsx`, `lista.css`.

Abaixo do gráfico aparece outro cartão com o mesmo anel, o mesmo nome e o mesmo "desconectado · 77% na última leitura" do cartão de estado. `Resumo.tsx:86` passa `sempre`, que em `ListaDeControles.tsx:19` baixa o mínimo de 2 para 1 controle, e `.lista.solta .item` (`lista.css:76-81`) o desenha como cartão cheio. O que ele acrescenta é renomear, clicando no nome sem pista visual além de um tracejado no hover, e esquecer, num X que só aparece no hover. É o dobro do peso visual para 5% de função.

`Resumo.tsx` só renderiza `ListaDeControles` quando `estado.quantosConhecidos >= 2`; o campo já existe. Com um só, renomear vai para o cartão de estado: `.dispositivo` vira o mesmo `<button className="item-nome">` de `ListaDeControles.tsx:97`, com um lápis de 12px em `--text-tertiary` à direita que aparece no hover e no foco, e ao clicar o campo `.renomear` (o input de `lista.css:52-61`, com `renomear_controle` no blur e no Enter) toma o lugar do nome. "Esquecer" vira um `.ciclo` secundário ao lado de Atualizar, visível só quando `via === "Desligado"`, com a mesma confirmação em linha (`Esquecer` em `.ciclo.perigo` e `Cancelar`). As regras de `.item-nome`, `.renomear`, `.confirmar` e `.ciclo.perigo` ganham cópia em `principal.css`, escopada por `.cartao.estado`.

**Pronto quando:** com um controle pareado o Resumo tem dois cartões, estado e gráfico, e nada abaixo; renomear e esquecer continuam possíveis a partir do cartão de estado; com dois controles a lista volta.

## T71. A sessão aberta no gráfico cabe numa linha e fala a mesma língua da lista

**Onde:** `src/componentes/historico.css`, `src/componentes/Historico.tsx`, `src/componentes/Sessoes.tsx`, `src/formato.ts`.

Clicar numa sessão com nome de jogo põe no cabeçalho do gráfico "Tom Clancy's Rainbow Six Siege X · ontem, 21:14 · 77% a 40%": `rotuloDaSessao` (`Sessoes.tsx:103-107`) concatena três partes sem prioridade, `.historico-titulo` não tem `min-width: 0` nem `white-space: nowrap` dentro do flex de `.historico-topo`, e `.faixas` não tem `flex: 0 0 auto`. Na janela mínima de 720px sobram uns 385px para o título, o texto quebra em duas linhas, o botão desce e o gráfico desce junto; na lista, `.sessao-titulo` corta com reticências, no cabeçalho não. O rodapé fala outra língua: a lista diz "2 h 40 min" e "6,8 %/h", `resumoDaSessao` diz "18 pontos em 160 min, ou 6,8% por hora", formatando por conta própria em vez de usar a `duracao()` que o item 35 do `TAREFAS.md` criou para unificar isso. E "voltar", em minúscula e sem seta, não parece um botão de sair de um modo.

`.historico-titulo { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis }`, `.historico-topo { gap: var(--space-3) }` e `.faixas { flex: 0 0 auto }`. `rotuloDaSessao` devolve só `s.jogo ?? quando(s.inicio)`; o que saiu vai para o rodapé, onde `resumoDaSessao` passa a `${quando(amostras[0].t)} · ${de}% → ${ate}% · ${duracao(minutos)} · ${taxa(porHora)}`, com `quando`, `duracao` e `taxa` de `formato.ts`, as mesmas que a linha da lista usa. O botão de sair vira "← Voltar", com SVG de seta de 12px, `aria-label="Voltar ao gráfico de 7 dias"`, na mesma classe `.faixa.ativa`.

**Pronto quando:** com um nome de 40 caracteres na janela de 720px, o cabeçalho fica numa linha, o nome termina em reticências e o botão Voltar fica no lugar dos botões 7/30; abrir qualquer sessão mostra no rodapé a mesma duração e a mesma taxa, com a mesma grafia, que a linha da lista; nenhuma tela escreve "% por hora".

## T72. As marcas de limiar sentam na trilha em vez de cortar o arco

**Onde:** `src/componentes/Anel.tsx`, `anel.css`.

A T3 pediu os traços "na cor da faixa correspondente a 40%". O código tem `.anel .marcas { opacity: 0.85 }` (`anel.css:15-17`) e `COMPRIMENTO_DA_MARCA = 0.62` (`Anel.tsx:10`) com `strokeWidth={espessura * 0.14}`: o traço atravessa 62% da espessura do arco a 85%, e sobre o arco verde lê-se como um corte ou um bug de renderização, não como "aqui vira âmbar".

`anel.css`: `opacity: 0.4`. `Anel.tsx`: `COMPRIMENTO_DA_MARCA = 0.5` e `strokeWidth={espessura * 0.1}`. O `<g className="marcas">` continua acima do arco. As linhas da lista (Anel de 26px) já não recebem marcas; ficam como estão.

**Pronto quando:** com 77% e limiares 20/10, os dois traços aparecem como marcações discretas na trilha e, onde o arco passa por cima, não parecem uma falha na linha.

## T73. A Saúde deixa de falar em primeira pessoa

**Onde:** `src/componentes/Saude.tsx`.

"Preciso de duas semanas de uso para comparar" e "Faltam 12 para eu poder comparar" (`Saude.tsx:68-72`) são as únicas frases do app em que ele diz "eu". Configurações, Diário e o próprio rodapé do gráfico são impessoais. E "Faltam 12" não diz 12 o quê.

Os três textos viram: "São necessárias duas semanas de uso para comparar."; "2 dias de histórico. Faltam 12 dias para comparar."; "Ainda não houve descarga suficiente nas duas semanas para comparar.".

**Pronto quando:** nenhum texto renderizado no Resumo contém "eu" ou "preciso", e o número de dias que faltam vem com a unidade.

---

# Movimento 12 — Pílula e aviso

A pílula e o aviso são as duas telas que a pessoa vê sem ter aberto o app: uma por cima do jogo, a outra por cima do que estiver na tela. Hoje a pílula esmaece o número junto com o fundo quando a transparência é configurada, some cortada e volta com um quadro cheio antes de deslizar, deixa o halo do anel vazar para fora do contorno, desenha cabo, leitura velha e ausência de leitura como se fossem carga viva, senta em cima da barra de tarefas no modo Sempre e aparece toda vez que a janela principal ganha foco. O aviso nasce no monitor errado, tem a sombra cortada em reta e diz a via em vez de quanto ainda dá para jogar. As tarefas abaixo consertam isso lendo o que o modelo já mede — `via`, `carregando`, `precisao`, `leituraAntiga`, `autonomia` — sem inventar dado novo, e trazem as duas telas para os tokens e as regras de movimento que o resto do app já segue.

## T74. A transparência da pílula apaga o número, e em Grande/Enorme ela entra crescendo

**Onde:** `src/telas/Sobreposicao.tsx`, `src/telas/sobreposicao.css`.

Em `Sobreposicao.tsx` a opacidade das Configurações vai inline no `.pilula` inteiro (`style={{ opacity: solta ? 1 : opacidade }}`), então texto, anel e glifo esmaecem na mesma proporção do fundo. Em "55%" sobre uma cena clara, o fundo `rgba(10,13,16,.94)` composto a 55% sobre branco dá cerca de `#828282` e o `#e8ecef` do número vira `#f2f4f6`: 3,4:1, abaixo dos 4,5:1 do DESIGN §8. O anel também perde saturação, e a cor da faixa é o dado. A transparência existe para o fundo não brigar com o HUD, não para o número sumir.

No mesmo elemento, `style={{ transform: scale(escala) }}` na raiz briga com `@keyframes kontro-pilula-entra`, cujo `from` é `transform: translateY(8px)`. Durante a entrada o `from` substitui a transform inteira, então em "Grande" e "Enorme" a pílula infla de 1 até 1,2 ou 1,45 enquanto sobe — um zoom que a T14 não pediu e que não descreve dado nenhum.

A raiz passa a receber só variáveis: `style={{ "--alfa": solta ? 1 : opacidade, "--escala": escala }}`. Em `sobreposicao.css`, `.sobreposicao { transform: scale(var(--escala, 1)); transform-origin: top left }` e o keyframe vira `from { opacity: 0; transform: translateY(8px) scale(var(--escala, 1)) }`. O gradiente do `.pilula` vira `rgb(28 34 41 / calc(0.94 * var(--alfa, 1)))` e `rgb(10 13 16 / calc(0.94 * var(--alfa, 1)))`, a sombra externa `0 2px 8px rgb(0 0 0 / calc(0.6 * var(--alfa, 1)))`; `.valor`, o anel e o glifo ficam a 100%, e `.valor` ganha `text-shadow: 0 1px 2px rgba(0, 0, 0, 0.55)` para segurar a leitura quando o fundo clareia.

**Pronto quando:** numa captura sobre fundo branco, o número e o anel têm exatamente a mesma cor em "Sólida" e em "55%", e só o fundo da pílula muda entre os dois; em "Enorme" a pílula aparece já no tamanho final, apenas subindo 8px em 240 ms.

## T75. A pílula some cortada e volta com um quadro cheio antes de deslizar

**Onde:** `src-tauri/src/orquestra.rs`, `src-tauri/src/atalho.rs`, `src-tauri/src/lib.rs` (`soltar_sobreposicao`), `src/telas/Sobreposicao.tsx`, `src/telas/sobreposicao.css`.

Só existe o evento `kontro://pilula-apareceu`. Toda saída — sair do jogo, apertar o atalho, prender depois de soltar — é `janela.hide()` seco em `orquestra.rs:99` e `atalho.rs:101`. Ao reaparecer pelo orquestrador, o `show()` vem antes do `emit`, e o DOM antigo continua desenhado dentro da janela oculta: há um quadro com a pílula inteira antes de ela voltar a opacity 0 e deslizar. Pelo atalho (`atalho.rs:99`) e ao soltar (`lib.rs`, `soltar_sobreposicao`) o `show()` vem sem evento nenhum, e a entrada nem roda. A T14 prometeu "aparecer e sumir com deslize"; a saída nunca existiu.

O Aviso já faz isso do jeito certo: classe `saindo`, `SAIDA_MS`, e só então `esconder_janela`. A pílula copia. Rust: os dois `hide()` viram `app.emit("kontro://pilula-vai-sumir", ())`, e os três `show()` passam a emitir `kontro://pilula-apareceu` sempre. Front: ao receber `vai-sumir`, `setSaindo(true)` e um `setTimeout` de 180 ms que chama `invoke("esconder_janela", { rotulo: "sobreposicao" })`; ao receber `apareceu`, `setSaindo(false)`, limpa o timeout e incrementa `entradas` (o `key` já remonta). Esconder pelo front, e não por um timer no Rust, é o que evita a corrida: um `show()` no meio dos 180 ms chega como `apareceu`, cancela o timeout e a janela nunca some. Em `sobreposicao.css`, `.sobreposicao.saindo { animation: kontro-pilula-sai var(--motion-base) var(--curva) forwards }` com `@keyframes kontro-pilula-sai { to { opacity: 0; transform: translateY(8px) scale(var(--escala, 1)) } }`; o `forwards` deixa a raiz em opacity 0, e a janela reaparece sem o quadro cheio. Para a primeira abertura do app, que nunca passou por uma saída, a raiz fica com `opacity: 0` enquanto `entradas === 0` — a medida do tamanho continua funcionando, e o primeiro `apareceu` remonta com a entrada.

**Pronto quando:** Alt-Tab do jogo faz a pílula descer 8px e sumir em 180 ms; voltar ao jogo, apertar o atalho ou soltar a pílula faz ela entrar deslizando, sem um quadro inteiro antes.

## T76. A pílula pula na tela toda vez que a janela principal ganha foco

**Onde:** `src-tauri/src/lib.rs`, `src-tauri/src/orquestra.rs`, `src/telas/Configuracoes.tsx`.

Abrir o Kontro para ver o Diário ou o Resumo faz a pílula aparecer no canto da tela, mostrando "77%" cinza se o controle está desligado. Em `orquestra.rs`, `ajustando` é `principal.is_focused()`, que não sabe qual aba está aberta. A prévia faz sentido enquanto se mexe em Tamanho, Transparência e Posição; nas outras abas é ruído.

`Compartilhado` ganha `previa_da_pilula: Mutex<bool>` e um comando `previa_da_pilula(ligada: bool)`. `Configuracoes.tsx` chama `true` num `useEffect` no mount e `false` no cleanup — `Principal.tsx` só monta a página da aba ativa, então trocar de aba desmonta. O laço em `lib.rs` lê o valor junto com `mao` e `solta` e passa para `reavaliar`; `ajustando` vira `previa && principal.is_focused()`. O foco continua na conta porque fechar a janela não desmonta o componente, e sem ele a pílula ficaria presa na tela. O passo da pílula em `Passos.tsx` não precisa de nada: ele já solta a pílula, e solta ela é mostrada por outro caminho.

**Pronto quando:** abrir o app no Resumo ou no Diário não traz a pílula; entrar em Configurações traz em até 2 s; trocar de aba ou fechar a janela esconde.

## T77. O aviso de conectar diz quanto ainda dá para jogar em frase inteira, e a pílula separa "acabando" de "baixa"

**Onde:** `src/telas/Aviso.tsx` (`legenda`), `src/telas/Sobreposicao.tsx` (`resumir`).

Com número, `legenda()` monta "conectado · Bluetooth": a via, que a pessoa já sabe, com ponto no lugar de preposição. A autonomia calculada (`estado.autonomia`, "~2 h 40 min de jogo") está no pacote e o flyout a mostra pelo `detalhe()` de `formato.ts`; o aviso não. Com nível mas sem número, a frase perde o verbo: "carga média · Bluetooth". `Aviso.tsx:82-90` escreve "agora · Bluetooth" e "agora · sem fio", enquanto o ramo do cabo já faz certo, "agora no cabo · carregando". `.nome` mostra `estado.nome` cru, e o toast nativo em `orquestra.rs` já cai em "O controle" para nome vazio. E a pílula reduz o nível 0 a "baixa": `resumir()` mapeia `["baixa", "baixa", "média", "cheia"]` (`Sobreposicao.tsx:151`), enquanto o Rust descreve o nível 0 como "quase acabando" e o 1 como "carga baixa"; em jogo, onde a diferença importa, os dois viram a mesma palavra.

Em `legenda()`, `const por = estado.via === "Bluetooth" ? "por Bluetooth" : "sem fio"`. Com número e autonomia estimada (começa por "~"): `${abertura} · ${estado.autonomia}`, "conectado · ~2 h 40 min de jogo"; com número e sem estimativa, que é o caso logo depois de conectar: `${abertura} ${por}`; com nível aproximado: `${abertura} ${por} · ${estado.textoDaCarga}`; sem leitura: `${abertura} ${por} · sem leitura de bateria`. O ramo do cabo fica como está. `.nome` mostra `estado.nome.trim() || "O controle"`. Em `Sobreposicao.tsx:151`, `["acabando", "baixa", "média", "cheia"]`.

**Pronto quando:** ligar um controle com autonomia medida mostra "conectado · ~2 h 40 min de jogo"; sem estimativa, "conectado por Bluetooth"; um controle de nível aproximado mostra "conectado por Bluetooth · carga média"; passar do cabo para o Bluetooth mostra "agora por Bluetooth"; nome vazio nunca deixa a primeira linha do cartão em branco; nível 0 na pílula escreve "acabando" e nível 1 "baixa".

## T78. Com dois controles, os anéis acompanhantes são anônimos

**Onde:** `src/telas/Sobreposicao.tsx`.

Em jogo local a pílula mostra "77% | 34%" com dois anéis de 20px e nada que diga qual é qual. O `title={c.nome}` do `.acompanhante` só funciona com a pílula solta, quando `set_ignore_cursor_events(false)` deixa o cursor chegar — e solta ninguém está jogando. Em jogo a janela ignora o cursor e o tooltip nunca abre.

O `title` sai. No miolo de cada acompanhante, no lugar do Glifo de 10px, entra o número de ordem do controle — a posição em `todos`, 1-based — em `--font-mono` 9px `--text-secondary`. `todos` vem de `useControles()`, o mesmo hook e a mesma ordem que `ListaDeControles.tsx` usa no Resumo, então o dígito bate com a lista. O principal continua com o glifo: é o que a bandeja mostra.

**Pronto quando:** com dois controles ligados, cada anel acompanhante da pílula carrega um dígito que corresponde à posição do controle na lista do Resumo.

---

# Movimento 13 — Instalação, atualização e avisos do sistema

A atualização hoje funciona: a consulta roda fora da thread da interface (T18), há dois canais (T22) e a nota da release viaja dentro do manifesto. O que falta é a conversa em volta dela. A nota chega ao cartão como uma pilha de títulos em caixa alta com asteriscos no meio; a falha do download aparece na linha de baixo, ao lado de um botão que diz "Procurar"; a rotina diária desiste por 24 h no primeiro tropeço de rede e nunca diz quando foi a última vez; a janela aberta não fica sabendo da versão que a rotina achou; o app volta da instalação sem dizer que instalou, e volta com a janela aberta sozinha quando a instalação nem chegou ao fim. Do lado do sistema, o toast de carga baixa sai sem o anel, às vezes em dobro e sem o tempo que resta, e a chave "Iniciar com o Windows" confia numa chave de registro que o Windows deixou de usar como veredito. As onze tarefas abaixo mexem quase só em `Configuracoes.tsx`, `lib.rs`, `atualizacao.rs`, `orquestra.rs` e `inicio_automatico.rs`, e nenhuma inventa número: toda frase nova sai de um dado que já existe — a versão gravada na marca, o carimbo da última checagem, a autonomia medida.

## T79. As notas da versão nova viram texto de gente

**Onde:** `src/telas/Configuracoes.tsx` (`Notas`, `emBlocos`), `src/telas/principal.css`, `README.md`.

O cartão "Versão 2.13.4 disponível" mostra a anotação da tag como nove `<h3>` seguidos de 11px, em maiúsculas e espaçados — o primeiro começando por "**CORREÇÃO:** O RANKING…", com os asteriscos literais. `emBlocos()` só conhece dois tipos de linha: marcador (`-`, `*`) vira item de lista e qualquer outra linha não vazia vira `{ tipo: "titulo" }`, renderizado com o estilo de rótulo de seção (`.notas h3`: 11px, `uppercase`, `letter-spacing 0.08em`). Ele não junta as linhas de um parágrafo, não fecha bloco em linha vazia e não tira a marcação. A nota que o `release.yml` gera sozinho ("O que entrou" mais uma lista de assuntos) cabe nesse molde; a nota escrita à mão — a tag anotada que o README convida a empurrar antes do merge, e que é o que as v2.13.2 a v2.13.4 têm — é prosa quebrada a 85 colunas com `**negrito**`, e vira isso.

`emBlocos` passa a acumular linhas não vazias consecutivas num bloco só e fechar em linha vazia. O primeiro bloco vira `{ tipo: "manchete" }` (14px/600, `--text-primary`); linha que começa por `#` vira `h3` sem o `#`; bloco de marcadores continua lista; qualquer outro vira `{ tipo: "paragrafo" }`, com `**trecho**` partido por `/\*\*(.+?)\*\*/` em `<strong>`. Em `principal.css`, `.cartao.novidade .notas .manchete { margin: 0 0 var(--space-2); font-size: 14px; font-weight: 600; color: var(--text-primary) }`, `.notas p { margin: 0 0 var(--space-2); font-size: 13px; line-height: 1.5; color: var(--text-secondary) }` e `.notas strong { font-weight: 600; color: var(--text-primary) }`. No README, a seção "Publicando uma versão" ganha uma frase com o formato que a anotação deve ter: manchete, linha em branco, parágrafos ou lista com `-`.

**Pronto quando:** com a anotação da v2.13.4 no manifesto, o cartão mostra uma manchete, três parágrafos legíveis com "Correção:" em negrito, e nenhum asterisco nem linha em maiúsculas; a nota gerada pelo `release.yml` continua saindo como rótulo "O que entrou" e lista.

## T80. O cartão de versão nova assume a falha, e o botão vira primário

**Onde:** `src/telas/Configuracoes.tsx` (`Novidade`, `atualizarAgora`, `tituloDaVersao`, `detalheDaVersao`), `src/componentes/botao.css`.

Sem rede, clicar em "Atualizar agora" some com a barra e o cartão volta a mostrar o mesmo botão, como se nada tivesse acontecido. Quem conta a falha é a linha de baixo, "Procurar atualizações": o título vira "Não foi possível atualizar" e a descrição traz o motivo, ao lado de um botão que continua dizendo "Procurar". `Novidade` recebe `passo` mas só o usa para `andando`; o ramo `falhou` é desenhado só por `tituloDaVersao` e `detalheDaVersao`. O motivo já vem em português desde a T22, porque `instalar_atualizacao` devolve `Err(motivo)` passado por `descrever()`; o que está errado é o lugar. E o botão que existe para um clique só é contorno: `.ciclo.destaque` pinta borda e texto em `--realce` e só preenche no hover, o mesmo peso de "Soltar" e "Salvar cartão", quando o `DESIGN.md` §7 pede botão primário à direita e reserva a cor de realce para ação primária.

`Novidade` passa a tratar `passo.tipo === "falhou" && passo.ao === "atualizar"`: no lugar do trilho entra um bloco `.falha` com o motivo em 14px `--atencao`, o token de erro da tarefa da captura de atalho, e não âmbar nem vermelho, que são cor de carga, e o botão vira "Tentar de novo", chamando `aoAtualizar`. `tituloDaVersao` e `detalheDaVersao` só olham falhas de `ao === "verificar"`. Em `botao.css` entra `.botao.primario { background: var(--realce); border-color: var(--realce); color: var(--ink); font-weight: 600 }`, hover em `color-mix(in srgb, var(--realce) 88%, #fff)` com transição `var(--motion-fast) var(--curva)`; "Atualizar agora" e "Tentar de novo" usam `botao primario`, e `.destaque` fica para os botões de estado, Prender e Soltar.

**Pronto quando:** desligar a rede e clicar em "Atualizar agora" mostra, dentro do cartão e sob o botão "Tentar de novo", uma frase em português, e a linha "Procurar atualizações" não muda de texto; o botão do cartão sai preenchido em `--realce` com texto escuro nos seis temas, com contraste de 4,5:1 conferido no Dia.

## T81. Um toast só por queda, com o tempo que ainda dá para jogar

**Onde:** `src-tauri/src/orquestra.rs` (`limiares`).

Com aviso em 20% e crítico em 10%, uma leitura que pula de 25% para 8%, comum porque o GATT anda em degraus de 5 e acelera no fim, como a tarefa 37 do `TAREFAS.md` mediu, gera o toast "Carga crítica" e, no ciclo seguinte, 2 s depois, "Carga baixa". O laço percorre `[critical_threshold, warn_threshold]`, faz `break` no primeiro limiar cruzado e marca só ele em `avisados`; o de aviso fica sem marca, passa no teste do ciclo seguinte e dispara sozinho, o menos urgente por último. E o corpo é "{nome} está com {pct}% de carga.": `estado.autonomia` ("~40 min de jogo") já está calculado na mesma função e aparece no flyout no mesmo instante, mas não no único aviso que interrompe o jogo. A pergunta que o app existe para responder, "quanto ainda dá pra jogar", fica de fora justamente ali.

`limiares()` separa a decisão numa `fn a_avisar(pct, avisados: &[i32], limiares) -> Option<i32>` pura: junta todos os limiares com `pct <= limite` ainda não avisados e devolve o menor; quem chama marca todos de uma vez e emite um toast só. O corpo vira `match &estado.autonomia { Some(a) if a.starts_with('~') => format!("{nome} está com {pct}% · {a}."), _ => format!("{nome} está com {pct}% de carga.") }`: só entra estimativa de tempo, com o prefixo "~"; "medindo o consumo" e "consumo de X %/h" ficam de fora, e o corpo continua uma linha, como o DESIGN §7 pede. Teste em `orquestra.rs`: 25 → 8 devolve o crítico e marca os dois; subir para 40, que limpa `avisados` pela regra dos 5 pontos, e cair de novo repete.

**Pronto quando:** simulando 25% → 8% aparece um único toast, "Carga crítica", com o mesmo "~N min de jogo" que o flyout mostra; sem autonomia estimada o toast diz o texto de hoje e não fala de tempo.

## T82. A chave "Iniciar com o Windows" diz o que o Windows faz

**Onde:** `src-tauri/src/inicio_automatico.rs`, `src-tauri/src/registro.rs`, `src-tauri/src/lib.rs` (`salvar_configuracoes`), `src/ajustes.ts`, `src/telas/Configuracoes.tsx`.

A pessoa desativa o Kontro em Configurações > Aplicativos > Inicialização do Windows 11, ou no Gerenciador de Tarefas. O app não sobe mais no boot, mas a chave continua verde e diz "Sobe junto com o sistema", e ligar e desligar não resolve. O Windows não apaga o valor em `HKCU\...\CurrentVersion\Run`: grava o veredito em `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`, valor binário "Kontro" cujo byte 0 é `0x02` (ativado) ou `0x03` (desativado). `ligado()` só pergunta se o valor em `Run` existe, e é isso que `lib.rs:103` copia para `start_with_windows` na subida. O caso inverso também mente: quando `definir()` falha (política de grupo, antivírus segurando o registro), `salvar_configuracoes` reverte `novas.start_with_windows = ligado()` e emite `kontro://config` — mas `Configuracoes.tsx` lê a config uma vez no mount, sem `listen`, e `ajustes.ts:salvar` aplica a mudança de forma otimista. A chave fica verde na tela, e só aparece cinza na próxima abertura, sem uma palavra de explicação.

`registro.rs` ganha `bytes()` (REG_BINARY, por cima do `ler()` que já devolve tipo e bytes) e `gravar_bytes()`. `inicio_automatico::ligado()` vira `linha_registrada().is_some() && aprovado_pelo_windows()`, com `fn aprovado(byte0: Option<u8>) -> bool` pura — `None` é aprovado, byte par é aprovado — e teste sobre ela; `definir(true)` reescreve o valor em `StartupApproved\Run` com `[0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]` quando ele estava desativado, para a chave do app e a página do Windows ficarem de acordo. `salvar_configuracoes` passa a devolver `Salvo { config: Settings, recusadas: Vec<String> }` (por exemplo `["StartWithWindows"]`); `ajustes.ts:salvar` vira `async` e devolve isso; `Configuracoes.tsx` aplica o `config` devolvido em vez do otimista e guarda `recusadas`, e a Linha "Iniciar com o Windows" mostra na descrição, no mesmo molde de `descricaoDoAtalho`, "O Windows não deixou gravar o início automático. Veja Configurações > Aplicativos > Inicialização." enquanto a recusa durar.

**Pronto quando:** desativar o Kontro na página de Inicialização do Windows e reabrir Configurações mostra a chave desligada, e ligar no app reativa também na página do Windows; com a chave `Run` bloqueada para escrita, ligar mostra a frase e a chave fica desligada na hora.

## T83. A marca de atualização só entra na instalação, e ao voltar o app diz que atualizou

**Onde:** `src-tauri/src/lib.rs` (`instalar_atualizacao`, `consumir_marca_de_atualizacao`, `marcar_atualizacao`, `versao_do_app`, `Compartilhado`), `src-tauri/src/atualizacao.rs` (`instalar`), `src/telas/Principal.tsx`, `src/telas/Configuracoes.tsx`.

Com "Iniciar minimizado" ligado, o app sobe no boot mostrando a janela principal em cima de tudo, uma vez, depois de qualquer tentativa de atualização que não chegou ao fim. `instalar_atualizacao` chama `marcar_atualizacao()` antes de `atualizacao::instalar(...).await?`; se o download falha, o arquivo `atualizando` fica, e na próxima subida `consumir_marca_de_atualizacao()` devolve `true` e `voltou_de_atualizacao` força `abrir_direto`. Quando a instalação dá certo, o arquivo carrega justamente a versão antiga (`marcar_atualizacao` grava `CARGO_PKG_VERSION`) e `consumir_marca` joga o conteúdo fora: o app reabre no Resumo como qualquer abertura, e para saber se deu certo a pessoa vai a Configurações procurar o número.

`marcar_atualizacao()` passa a ser chamada dentro do callback de instalação de `atualizacao::instalar` — o `move ||` que emite `Andamento::Instalando`, o último instante antes de o instalador assumir — e o comando sai do `generate_handler`, porque o front não o chama mais. `consumir_marca` devolve `Option<String>`; se a versão gravada for diferente de `CARGO_PKG_VERSION`, fica em `Compartilhado.veio_de`, e `versao_do_app` passa a devolver `{ atual, veioDe }`. `Principal.tsx` começa em `"config"` quando `veioDe` existe; `Configuracoes.tsx` ganha o passo `"chegou"`, com título "Atualizado para a 2.14.0" e descrição "Você estava na 2.13.4." em Caption/Mono, que some no próximo clique em Procurar. Só aparece quando as duas versões medidas diferem.

**Pronto quando:** desligar a rede, clicar em "Atualizar agora", fechar o app e abrir de novo com "Iniciar minimizado": a janela não aparece. Instalar uma versão nova pelo cartão: o app reabre em Configurações com "Atualizado para a X" e a versão anterior embaixo; uma abertura comum não mostra isso.

## T84. A checagem diária tenta de novo sem rede, e a linha de versão diz quando foi a última e para de trocar de nome

**Onde:** `src-tauri/src/lib.rs` (`iniciar_ciclo`, `avisar_versao_nova`, `Pedido`, `generate_handler!`), `src-tauri/src/atualizacao.rs`, `src/telas/Configuracoes.tsx` (`Novidade`, `tituloDaVersao`, `detalheDaVersao`), `src/formato.ts`.

Com "Iniciar com o Windows", o app sobe antes do Wi-Fi conectar, consulta o GitHub, falha em silêncio e só tenta de novo no dia seguinte: `proxima_checagem = agora + JANELA_MS` é definido em `lib.rs:293` antes de saber o resultado, `marcar_checagem` só roda em `Nova` e `EmDia`, e `avisar_versao_nova` roda numa thread que descarta a `Consulta`. Na primeira abertura de vida `ultima_checagem()` é 0 e a consulta roda no primeiro ciclo: se a versão instalada já estiver atrás, o toast "Kontro X disponível" cai em cima do passo a passo. Na tela, quem abre Configurações com versão nova só acha o cartão rolando até quase o fim, porque `<Novidade>` mora na seção "Versão", a penúltima, e o DESIGN §7 diz que versão nova é estado, não ajuste. A linha de baixo troca o título para "Você está na versão mais recente" ou "Não foi possível verificar" e nunca volta; a descrição diz "O app verifica sozinho uma vez por dia" com a chave "Avisar sobre versões novas" desligada logo abaixo, o que é falso porque `lib.rs:295` só consulta com `auto_check_updates`; e não diz quando foi a última vez, porque `ultima_checagem()` existe e nenhum comando a expõe. O cartão ainda escreve "você está na 2.13.4" em minúscula.

Rust: `Pedido` ganha `VersaoConsultada { deu_certo: bool }`; `avisar_versao_nova` recebe o `Sender` e manda o resultado ao fim da thread. No laço, `deu_certo` marca `proxima_checagem = agora + JANELA_MS` e falha marca `agora + REPETE_SEM_REDE_MS` (15 min, constante em `atualizacao.rs`). Na subida, `proxima_checagem = max(ultima + JANELA_MS, agora + CARENCIA_DA_SUBIDA_MS)`, com 2 min de carência, e a consulta só dispara com `cfg.first_run_done`. Comando `ultima_verificacao() -> i64`, 0 quando nunca, chamando `atualizacao::ultima_checagem()`, registrado no `generate_handler!`.

Tela: `{nova && <Novidade/>}` vai para logo depois do `<h1>`, antes de "Inicialização", com "Você está na" em maiúscula. A linha ganha título fixo "Procurar atualizações", e a descrição vira uma linha de estado em Mono 12px `--text-tertiary`, lida na montagem e de novo depois de cada `procurar()`: "Você está na 2.13.4 · verificado hoje às 09:12", com o `quando()` de `formato.ts`; "· ainda não verificado" com 0; "Consultando o repositório…" enquanto procura; "Nada novo desde esta versão" depois de em dia; "Não foi possível verificar: {motivo}" depois da falha. A frase da rotina diária sai dali e vai para a descrição da chave "Avisar sobre versões novas", onde ela pertence: "Consulta o repositório uma vez por dia, sem baixar nada sozinho."

**Pronto quando:** subir sem rede e ligar a rede 5 min depois faz o toast de versão nova aparecer em até 15 min; na primeira abertura o toast nunca aparece antes de "Começar"; abrir Configurações com versão nova mostra o cartão sob o título sem rolar; o título da linha nunca muda; clicar em Procurar mostra "verificado hoje às HH:MM", e abrir no dia seguinte "ontem às HH:MM"; desligar a chave não deixa frase dizendo que o app verifica sozinho.

## T85. Versão nova chega na janela que já está aberta, e a aba avisa

**Onde:** `src-tauri/src/lib.rs` (`avisar_versao_nova`, `procurar_atualizacao`, `instalar_atualizacao`), `src/estado.ts`, `src/telas/Configuracoes.tsx`, `src/telas/Principal.tsx`, `src/telas/principal.css`.

Com a janela aberta em Configurações, a rotina diária encontra versão nova, o toast diz "Abra as configurações do Kontro para instalar" — e a tela onde a pessoa já está continua sem cartão até ela trocar de aba e voltar. `avisar_versao_nova` só grava `Compartilhado.novidade`; `Configuracoes.tsx` chama `versao_disponivel` uma vez, no mount; o trilho de abas não recebe nada, então quem abre o app cai no Resumo e nenhuma aba sinaliza que há algo esperando.

Depois de guardar, `lib.rs` emite `app.emit("kontro://novidade", Option<VersaoNova>)` — em `avisar_versao_nova`, e também em `procurar_atualizacao` e `instalar_atualizacao` (com `None` quando a consulta diz em dia), para a fonte ser uma só. `estado.ts` ganha `useNovidade()` no molde de `usePilulaSolta`: `invoke("versao_disponivel")` mais `listen`. `Configuracoes.tsx` usa o hook e deixa de guardar `nova` em estado próprio. `Principal.tsx` põe, no botão "Configurações" do trilho, um `<span className="aviso" />` quando há novidade: ponto de 6px em `--realce`, `margin-left: auto`, classe `.aba .aviso` em `principal.css`, que some quando a novidade volta a `null`.

**Pronto quando:** com Configurações aberta, uma consulta que encontra versão nova faz o cartão aparecer sem trocar de aba; no Resumo, a aba Configurações mostra o ponto; depois de instalar, ou quando a consulta diz que está em dia, o ponto some.

## T86. O toast de carga leva o anel

**Onde:** módulo novo `src-tauri/src/avisos.rs`, `src-tauri/src/orquestra.rs`, `src-tauri/src/lib.rs` (`avisar_versao_nova`), `src-tauri/src/bandeja.rs`, `src-tauri/Cargo.toml`.

O toast do Windows mostra o ícone genérico do atalho e "Carga baixa / Xbox Wireless Controller está com 18% de carga." O `DESIGN.md` §7 manda o ícone `level-25` ou `level-10` — o anel âmbar ou vermelho, que é o que faz a pessoa reconhecer o aviso pelo canto do olho, como faz na bandeja. O builder é chamado só com `.title()` e `.body()`, e não adianta acrescentar `.icon()`: no Windows o plugin 2.3.3 passa isso ao campo `icon` do notify-rust, que o `build_toast` de `windows.rs` ignora; só `path_to_image` chega ao toast, e o builder do plugin não expõe esse campo.

`avisos.rs` monta o toast direto com o `tauri-winrt-notification` 0.7.3, que já está no `Cargo.lock` por baixo do notify-rust e passa a dependência direta: `Toast::new(app_id).title(titulo).text2(corpo).icon(&caminho, IconCrop::Circular, "").show()`, com `app_id` igual ao `identifier` do `tauri.conf.json` (`com.riqueamais.kontro`) quando o exe não está em `target\debug` ou `target\release` — o mesmo critério do plugin — e `Toast::POWERSHELL_APP_ID` no dev. `fn imagem_do_aviso(estado, limiares) -> Option<PathBuf>` rasteriza `bandeja::montar_svg()` a 96px (o `rasterizar` de `bandeja.rs` ganha uma variante que devolve o `Pixmap` para `save_png`) em `%APPDATA%\Kontro\avisos\nivel-{pct}.png`, uma vez por percentual; para versão nova, `svg_do_app(96)` em `avisos\app.png`. `orquestra::limiares` e `lib::avisar_versao_nova` chamam `avisos::mostrar(titulo, corpo, imagem)`, e o plugin de notificação sai do `Cargo.toml` e do `lib.rs`, porque esses são os dois únicos usos.

**Pronto quando:** o toast "Carga baixa" mostra o anel âmbar com o glifo do controle; o de "Carga crítica", o vermelho; o de "Kontro X disponível", a marca do app; e a central de Notificações do Windows continua agrupando os três sob "Kontro".

## T87. O README para de prometer o que a tela não faz

**Onde:** `README.md` (linhas 5, 7 e 51-53).

Quem lê "o app se atualiza sozinho a partir das releases publicadas aqui" instala e espera não fazer nada. `avisar_versao_nova` só mostra o toast e guarda a novidade; instalar é um clique no cartão, e a própria chave em Configurações diz "sem baixar nada sozinho". Na mesma página, o link de topo está rotulado "kontro.riqueamais.github.io", domínio que não existe — a URL é `riqueamais.github.io/kontro` —, e "O ícone mostra a porcentagem exata" contradiz o `DESIGN.md` §1, que tira o número do ícone: a porcentagem fica na dica da bandeja e no painel.

A linha 5 passa a rotular o link como "riqueamais.github.io/kontro". A linha 7 vira "O ícone é um anel que esvazia com a carga e muda de cor conforme ela cai; a porcentagem exata fica na dica e no painel." A frase da instalação vira "…e avisa quando há versão nova nas releases publicadas aqui; instalar é um clique em Configurações."

**Pronto quando:** nenhuma frase do README descreve comportamento que a interface não tem: os três trechos batem com `Configuracoes.tsx` e com o `DESIGN.md`.

## T88. O instalador se apresenta como instalador

**Onde:** `src-tauri/src/icones.rs`, `src-tauri/src/geometria.rs`, `.github/workflows/verificar.yml`, `README.md` (tabela "o que é gerado").

`assets/branding/setup.ico` é byte a byte igual a `src-tauri/icons/icon.ico`, porque `icone_do_instalador()` grava a mesma marca sem variação: na pasta Downloads, na barra de tarefas durante a instalação e no diálogo do SmartScreen, `Kontro_2.13.4_x64-setup.exe` se apresenta com o ícone do app já instalado. E os dois bitmaps do NSIS, `nsis-header.bmp` (150x57) e `nsis-sidebar.bmp` (164x314), exportados uma vez em 22/08, são a marca solta sobre Ink: a coluna da tela de boas-vindas tem a marca no meio e nada mais — sem "Kontro", sem a tagline —, e o cabeçalho é a marca de 40px encostada à direita numa faixa preta sobre o diálogo claro. Nenhum dos dois sai de `--gerar` nem consta da tabela do README que diz que tudo o que deriva do Rust sai de lá.

`geometria.rs` ganha `SETA_DO_INSTALADOR`, uma seta para baixo em path simples, e `icone_do_instalador` desenha a marca de sempre com um selo no quadrante inferior direito, 40% da caixa: disco em Surface com a seta em AccentGreen, fora do anel, para a marca do §1 não mudar. Os dois BMP passam a sair de `icones::gerar`, a partir de um SVG composto — marca, "Kontro" em Segoe UI Variable Display 22/600 e a tagline "Bateria do seu controle na bandeja" em Caption, sobre Ink — nos tamanhos que o MUI2 espera, 150x57 e 164x314: o NSIS estica o bitmap para o diálogo em qualquer DPI, então gerar em 2x não ganha nitidez, e o que a tarefa entrega é a composição e a origem. BMP de 24 bits com o cabeçalho de 54 bytes escrito à mão, como `montar_ico` já faz com o ICO. Texto no resvg pede `usvg::Options` com `fontdb_mut().load_system_fonts()`, só nesse caminho; como texto rasterizado depende da fonte instalada, `verificar.yml` exclui os dois BMP da conferência de "artefatos em dia". A tabela do README ganha a linha dos dois.

**Pronto quando:** na pasta Downloads o `setup.exe` mostra a marca com a seta e o app instalado continua com o ícone atual; a tela de boas-vindas mostra marca, "Kontro" e a tagline legíveis em 100% e 150% de DPI; rodar `--gerar` nesta máquina reescreve os dois BMP sem diferença no git.

---

# Movimento 14 — Configurações

A página de Configurações hoje é uma coluna de linhas com o controle à direita: sete chaves, nove botões de ciclo, duas capturas de atalho e seis amostras de tema, tudo em `src/telas/Configuracoes.tsx` sobre `Linha`, `Chave` e `MiniTela` de `src/componentes/Controles.tsx`. Ela funciona, mas guarda uma cópia própria da config que nunca ouve o Rust, esconde as opções dentro de botões que mostram uma por vez, prende os limiares de cor do anel à chave de notificação e não dá nome às chaves para quem navega por teclado. Nas tarefas abaixo a tela passa a refletir o que o Rust gravou de verdade, cada escolha vira lista ou deslizante com o valor à vista, os limiares ganham seção própria, e todo controle da coluna direita ganha nome, altura, estado de pressionado e a cor de interface do tema.

## T89. A tela passa a ouvir o que o Rust gravou

**Onde:** `src/telas/Configuracoes.tsx`, `src/estado.ts`, `src-tauri/src/lib.rs`.

A pessoa solta a pílula, arrasta para o canto novo, prende. A mini-tela continua mostrando o ponto antigo, e ao ligar ou desligar qualquer chave da página a pílula pula de volta para onde estava. `Configuracoes` enche um `useState<Config>` com um `invoke("configuracoes")` único e nunca assina `kontro://config`. `soltar_sobreposicao` em `lib.rs` grava `OverlayX`/`OverlayY` e emite o evento, mas a cópia local fica velha; o próximo `gravar()` faz `salvar(cfg, mudanca)`, que é `{...cfg, ...mudanca}` com a posição antiga, e `salvar_configuracoes` chama `posicionar_sobreposicao` com ela. O mesmo vale para tudo que o Rust corrige por baixo: `StartWithWindows` revertido quando `inicio_automatico::definir` falha, e um atalho igual ao outro que `Settings::ajustar` devolve ao padrão. O Rust emite a config corrigida e a tela ignora.

No `useEffect` de `Configuracoes`, ao lado do `invoke<Config>("configuracoes")`, entra `listen<Config>("kontro://config", ({ payload }) => setCfg(payload))` com a mesma guarda `vivo` e o mesmo cleanup que `useConfig()` já usa em `estado.ts`. `gravar()` continua otimista, com `setCfg` imediato; o evento que volta do `salvar_configuracoes` sobrescreve com o que foi de fato guardado. `trocarCanal` segue igual. `Passos.tsx` já lê por `useConfig` e não muda.

**Pronto quando:** soltar, arrastar, prender e em seguida ligar "Avisar ao conectar": a pílula não se move e a mini-tela já mostra o ponto novo sem trocar de aba. Gravar em "Soltar a pílula" a mesma combinação de "Mostrar e esconder": o botão volta a mostrar a combinação que o Rust manteve.

## T90. Os limiares saem de baixo dos avisos

**Onde:** `src/telas/Configuracoes.tsx`, `src/componentes/Controles.tsx`, `src/telas/principal.css`, `src/telas/Passos.tsx`, `src/ajustes.ts`.

Quem desliga "Avisar carga baixa" perde os dois botões de limiar: os dois têm `disabled={!cfg.NotificationsEnabled}` (`Configuracoes.tsx:253` e `271`), e a descrição diz "Notificação ao cruzar os limiares abaixo". Mas `notifications_enabled` só é consultado em `Orquestrador::limiares` para o toast: `Settings::limiares()` pinta a bandeja e `useLimiares()` pinta Resumo, Painel, Sobreposição, Aviso e Diário sem olhar a chave, como o DESIGN §2 manda, e decidem quando a pílula sobe fora de jogo com contorno vermelho. Quem não quer notificação perde o direito de escolher a cor do anel, e o rótulo "Avisar em" diz que o número só serve para avisar, enquanto o passo 4 do tutorial explica certo: "os mesmos números mandam na cor do anel". O segundo botão ainda trava: `Math.min(ciclar(...), WarnThreshold - 5)` sobre `[20, 15, 10, 5]` deixa o crítico em 5 para sempre com o aviso em 10, e com 15 só há duas opções; a regra do Rust em `Settings::ajustar` é `aviso - 1`. São duas regras para o mesmo limite, e o clique é ignorado sem texto.

Entra `<h2>Limiares de carga</h2>` antes de "Avisos", com "Carga baixa" ("Abaixo disto o anel fica âmbar na bandeja, no resumo e na pílula e, com os avisos ligados, chega a notificação.") e "Carga crítica" ("Abaixo disto fica vermelho, e a pílula aparece mesmo fora de jogo."), os mesmos títulos das notificações em `orquestra.rs:179` e do DESIGN §2, nenhum `disabled`. Os dois usam `Deslizante`, novo em `Controles.tsx`: `<input type="range">` com `min`, `max`, `passo`, `valor`, `aoMudar` e `rotulo`, o valor em Mono 12px `--text-primary` com `min-width: 40px` alinhado à direita para o número não dançar; trilho de 4px em `--stroke` com a parte preenchida em `--realce` por `linear-gradient` até uma custom property `--preenchido` posta inline; polegar de 16px redondo em `--realce` com borda 2px `--surface`; `:active` no polegar `scale(1.15)` em `var(--motion-fast) var(--curva)`; o foco é o `:focus-visible` do `base.css`. Faixas iguais às de `Settings::ajustar`: baixa de 5 a 90, de 5 em 5; crítica de 1 até `WarnThreshold - 1`, de 1 em 1. Durante o arrasto só o estado local e o número mudam; `gravar()` dispara no `pointerup` e no `keyup`, uma gravação por gesto, porque `salvar_configuracoes` reaplica os atalhos e reposiciona a pílula a cada chamada. Baixar o aviso para menos que o crítico manda `CriticalThreshold: Math.min(cfg.CriticalThreshold, aviso - 1)`, e a descrição da crítica mostra "Ajustado para {n}% para caber abaixo da carga baixa." até a próxima mudança. `LIMIARES_DE_AVISO` e `LIMIARES_CRITICOS` ficam em `ajustes.ts`, porque `Passos.tsx` ainda os usa, e as linhas do passo 4 (`Passos.tsx:222` e `236`) passam aos mesmos títulos. A seção "Avisos" fica com "Avisar carga baixa" ("Notificação do Windows ao cruzar cada limiar.") e "Avisar ao conectar".

**Pronto quando:** com "Avisar carga baixa" desligada, levar "Carga baixa" de 20 para 60 deixa o anel do Resumo âmbar ao soltar o polegar; com o aviso em 10 a crítica oferece de 1 a 9 e cada passo muda o número; os rótulos das linhas coincidem com os títulos das notificações e com os do passo a passo.

## T91. Toda linha diz seu nome, e linha desabilitada parece desabilitada

**Onde:** `src/componentes/Controles.tsx`, `src/ajustes.ts`, `src/telas/Configuracoes.tsx`, `src/telas/Passos.tsx`, `src/telas/principal.css`.

Com o Narrador, Tab pela página de Configurações lê "interruptor, ativado", "botão, 20%", "botão, Ctrl Shift M", nunca "Iniciar com o Windows" ou "Carga baixa": `Linha` renderiza título e descrição em `<div>` sem `id` e o controle em outro `<div>`; `Chave` é `<button role="switch" aria-checked>` com um `<span class="bolinha">` vazio e nenhum nome; os botões de valor só têm o valor como conteúdo. Quem enxerga também não tem o "Ativado"/"Desativado" que o Windows 11 põe ao lado de toda chave: a pista é a cor e a posição da bolinha. As linhas não sabem quando estão desabilitadas: "Quando aparecer" em "Desligada" deixa Posição, Monitor, Tamanho e Transparência clicáveis, e "Soltar" chama `soltar_sobreposicao`, que faz `janela.show()` numa pílula que `orquestra.rs` nunca vai mostrar; "Usar atalhos" desligado desabilita as duas capturas, mas `.captura:disabled` só baixa o botão a 60% e o título fica no brilho cheio. E os botões de ciclo só andam para frente: `ciclar()` conhece só `(i + 1) % opcoes.length`, então voltar de 10% para 15% custa cinco cliques, e com teclado só Enter e Espaço.

`Linha` gera `const id = useId()`, põe `id={`${id}-t`}` no `.titulo` e `id={`${id}-d`}` na `.descricao`, e entrega `{ titulo, descricao, desabilitada }` por um `ContextoDaLinha` (`createContext`). `Chave`, `Captura`, os botões `.ciclo` que restarem e o `Deslizante` da tarefa anterior e o `Seletor` da seguinte leem o contexto e põem `aria-labelledby` (o título, mais o próprio `id` do botão quando o conteúdo é o valor, para o nome sair "Mostrar e esconder a pílula, Ctrl Shift M"), `aria-describedby` (a descrição) e `disabled={desabilitada || disabled}`; a prop `desabilitado` de `Captura` sai. Os botões de ação da página (Soltar/Prender, Procurar, Rever, Salvar) mantêm o próprio texto como nome e recebem só o `aria-describedby`. Antes da chave, dentro de `.linha .controle`, um `<span className="estado-da-chave">` com "Ativado" ou "Desativado" em 12px `--text-tertiary`, `min-width: 68px`, alinhado à direita. `Linha` ganha a prop `desabilitada`, que aplica `.linha.desabilitada`: `.titulo` em `--text-tertiary`, `.descricao` e `.controle` a 60% de opacidade, `pointer-events: none` no controle; aplicada com `cfg.OverlayMode === "Desligada"` nas quatro linhas da pílula e com `!cfg.OverlayShortcutEnabled` nas duas de atalho. Os ciclos que continuam ciclo, os de "Carga baixa" e "Carga crítica" do passo 4, andam para os dois lados: `ciclar(atual, opcoes, sentido: 1 | -1 = 1)` em `ajustes.ts` vira `(i + sentido + opcoes.length) % opcoes.length`, ArrowRight e ArrowUp avançam, ArrowLeft e ArrowDown voltam, e Shift+clique volta.

**Pronto quando:** no Narrador ou no Inspect do SDK do Windows, cada chave, atalho, seletor e deslizante de Configurações tem Name começando pelo título da linha e Description igual à descrição, e Tab até a primeira chave lê "Iniciar com o Windows, botão de alternância, ativado"; ao lado de cada chave aparece Ativado ou Desativado; escolher "Desligada" esmaece as quatro linhas inteiras, título incluso, e "Soltar" não responde, e "Só em jogo" as traz de volta; no passo 4, seta para a esquerda volta "Carga baixa" de 10% para 15%.

## T92. As escolhas de lista abrem uma lista

**Onde:** `src/componentes/Controles.tsx`, `src/telas/Configuracoes.tsx`, `src/telas/principal.css`, `src-tauri/src/lib.rs`.

"Ao clicar no X", "Quando aparecer" e "Monitor" são botões `.ciclo` que giram sobre uma lista fixa: para saber quais são os modos da pílula a pessoa clica três vezes e volta ao início. "Monitor" com uma tela só vai de "Segue o jogo" para "Monitor 1" e nada muda, porque `quantidade_de_telas` devolve só a contagem e o botão cicla `-1..telas-1` mesmo com `telas === 1`. Com duas, "Monitor 1" e "Monitor 2" não dizem qual é qual.

`Seletor` em `Controles.tsx`: um `<button>` de 32px com o rótulo atual e um chevron de 12px, `aria-haspopup="listbox"` e `aria-expanded`; abre um `<ul role="listbox">` posicionado abaixo, alinhado à direita, `min-width: 200px`, fundo `--surface-alt`, borda 1px `--stroke`, raio 8, padding 4px, itens `role="option"` de 32px com padding 0 12px e `aria-selected`, o escolhido com a barra `inset 2px 0 0 var(--realce)` que `.aba.ativa` já usa. É o menu da bandeja do DESIGN.md §7. Abre em `--motion-fast` (opacidade e `translateY(-4px)` até 0), fecha com Escape, clique fora ou escolha; setas movem, Enter escolhe. Opções como `{ valor, rotulo }`. Usado em "Ao clicar no X", "Quando aparecer" (Desligada, Só em jogo, Sempre visível) e "Monitor".

Para o monitor, `quantidade_de_telas` vira `telas`, devolvendo `Vec<Tela { largura: u32, altura: u32, principal: bool }>` a partir de `app.available_monitors()`: tamanho lógico (`size` dividido por `scale_factor`) e `principal` comparando `position()` com o de `app.primary_monitor()`. `Monitor::name()` no Windows devolve `\\.\DISPLAY1`, não serve de rótulo; a opção fica "Monitor 1 · 2560×1440 · principal". A ordem é a mesma de `available_monitors()` que `monitor_da_sobreposicao` usa em `janelas.rs`, então o índice escolhido é o que o Rust usa. Com `telas.length <= 1` a linha recebe `desabilitada` e a descrição "Só há uma tela ligada; a pílula fica nela."

**Pronto quando:** um clique em "Quando aparecer" mostra as três opções juntas com a atual marcada; com um monitor a linha fica esmaecida e não oferece escolha; com dois, cada opção traz a resolução da tela.

## T93. Tamanho e opacidade viram deslizante com prévia

**Onde:** `src/telas/Configuracoes.tsx`, `src/componentes/Controles.tsx`, `src/telas/principal.css`.

"Tamanho" tem quatro degraus (0,85, 1, 1,2, 1,45) embora `Settings::ajustar` aceite 0,75 a 2,0. "Transparência" oferece "Sólida, 90%, 75%, 55%": 90 % de transparência seria quase invisível; o número é de opacidade, e um valor fora da lista (o Rust aceita 0,3 a 1,0) cai em `${Math.round(valor * 100)}%` sem a linha saber de quê. E no modo padrão "Só em jogo" mudar o tamanho não mostra nada, porque a pílula só existe durante um jogo: `MiniTela` desenha a posição, mas a mini-pílula é fixa em 10×4 com opacidade 1.

As linhas viram "Tamanho" e "Opacidade" ("Em 100 % ela é sólida. Baixe para não competir com o HUD do jogo."), as duas com o `Deslizante` da tarefa dos limiares: tamanho 75 a 200 de 5 em 5, mostrado como "100 %" e gravado como `OverlayScale: valor / 100`; opacidade 30 a 100 de 5 em 5, gravada como `OverlayOpacity: valor / 100`. `TAMANHOS`, `OPACIDADES` e `rotulo()` saem. `MiniTela` recebe `escalaDaPilula` e `opacidade`: largura `10 × escalaDaPilula`, altura `4 × escalaDaPilula`, `style.opacity`. Fica uma `MiniTela` só, com `escala={2}` (96×56), num bloco `.previa` no topo da seção Sobreposição (fundo `--surface`, borda `--stroke`, raio 8, padding 12, margin-bottom 8, mini-tela centrada), servindo de prévia para Posição, Tamanho e Opacidade; a linha Posição fica só com o botão Soltar/Prender. `Passos.tsx` continua chamando `MiniTela` sem as props novas, que têm padrão 1.

**Pronto quando:** arrastar Tamanho de 100 para 145 faz a mini-pílula crescer e o número acompanha; 55 de opacidade a deixa visivelmente mais apagada; o nome da linha e o número dizem a mesma coisa.

## T94. "Problemas" vira "Ajuda", e o diagnóstico diz onde salvou

**Onde:** `src/telas/Configuracoes.tsx`.

Rever as seis telas de apresentação mora numa seção chamada "Problemas". Depois de salvar o diagnóstico a descrição diz "Salvo como diagnostico.txt, e a pasta abriu" sem dizer onde, embora `salvar_diagnostico` devolva o caminho completo e a tela o descarte no `await invoke<string>(...)`.

A seção vira `<h2>Ajuda</h2>` com "Rever o passo a passo" (botão "Rever") e "Salvar diagnóstico". O estado `diagnostico` passa a `{ passo, caminho }`, e a descrição em pronto vira "Salvo em {caminho}. Anexe ao relatar um problema.", com o caminho num `<code>` em `--font-mono` 11,5px dentro da descrição. Não entra a linha "Sobre" que a auditoria pediu: a versão já mora na linha de versão.

**Pronto quando:** depois de salvar, o caminho completo aparece na linha; o passo a passo não mora mais em "Problemas".

## T95. Títulos e valores falam do que acontece

**Onde:** `src/telas/Configuracoes.tsx`.

"Ao clicar no X — Fechar a janela pode só esconder o app." com valores "Minimizar" e "Encerrar": minimizar não é o que acontece, a janela some para a bandeja. "Iniciar minimizado" tem o mesmo problema. "Quando aparecer — Fixa na tela por cima do que estiver aberto." descreve a pílula, não as opções. "Usar atalhos — Valem por cima do jogo" é a única linha que fala de jogo sem dizer que são atalhos de teclado. Os textos foram escritos do ponto de vista de `CloseAction` e `StartMinimized`, não de quem escolhe.

"Botão de fechar" com "O que o X da janela faz." e valores "Esconder na bandeja" e "Encerrar o Kontro". "Abrir na bandeja" com "Sobe sem mostrar esta janela.". "Quando aparecer" com "Desligada, só durante um jogo em tela cheia, ou sempre.". "Atalhos de teclado" com "Funcionam com o jogo em foco, sem sair dele.". "Carga crítica" já entra na tarefa dos limiares.

**Pronto quando:** nenhum valor de botão descreve algo diferente do que acontece ao escolhê-lo.

---

# Movimento 15 — Acessibilidade e teclado

Hoje o app é feito para mouse e para quem enxerga cor: as chaves e os seletores de Configurações não têm nome para o leitor de tela, o anel e as células do Diário existem só como desenho, "baixa" e "crítica" só existem como âmbar e vermelho, o gráfico só responde a `onMouseMove`, e não há um `@media (forced-colors)` nem tratamento honesto de `prefers-reduced-motion` em `src/`. O que vira: cada controle tem nome e descrição, a faixa de carga vira palavra em toda superfície (cartão, dica da bandeja, pílula), tudo que o mouse alcança o teclado também alcança, e o app continua legível com alto contraste, sem animação e com o texto terciário acima de 4,5:1 nos seis temas.

## T96. A faixa de carga vira palavra, não só cor

**Onde:** `src/estado.ts`, `src/componentes/Anel.tsx`, `src/telas/Resumo.tsx`, `src/telas/Painel.tsx`, `src/formato.ts`.

O leitor de tela passa pelo anel do Resumo e do painel sem dizer nada: o `<svg>` de `Anel` não tem `role`, `aria-label` nem `aria-hidden`. Ele ouve "77%" e "Xbox Wireless Controller", mas não que a carga está "com folga" ou "baixa", nem que as marcas de limiar existem. Quem não distingue verde de âmbar vê o número e não sabe se o app já considera aquilo baixo. `corDoAnel` devolve cor e não existe função irmã que devolva o nome da faixa; `detalhe()` mostra só a autonomia ou a ligação.

Em estado.ts entra `faixaDaCarga(estado, limiares)`, com os mesmos ramos de `corDoAnel`, devolvendo `"com folga" | "carga baixa" | "carga crítica" | "no cabo" | "desligado" | "sem leitura"` — as mesmas palavras que o passo 2 do passo a passo já usa em `CORES`. `Anel` ganha a prop `rotulo?: string`: com ela o `<svg>` recebe `role="img" aria-label={rotulo}`; sem ela, `aria-hidden="true"` (Passos, lista de controles, aviso, pílula, acompanhantes). Resumo e Painel passam `rotulo={`${estado.textoDaCarga}, ${faixa}. Avisa em ${limiares.aviso}%, insiste em ${limiares.critico}%`}` quando `temNumero`, e `detalhe(estado)` quando não. Em formato.ts, `detalhe()` prefixa a faixa quando há número: "carga baixa · ~2 h de jogo", "com folga · Bluetooth".

**Pronto quando:** o Narrador, ao chegar no cartão de estado, lê o percentual, a palavra da faixa e os dois limiares; o texto abaixo do nome do controle no Resumo e no painel traz a faixa mesmo com o monitor em escala de cinza.

## T97. No alto contraste do Windows a chave, a aba, o tema, o mapa e a pílula continuam mostrando o estado

**Onde:** `src/telas/principal.css`, `src/telas/diario.css`, `src/telas/sobreposicao.css`, `src/componentes/anel.css`, `src/estilo/tokens.css`.

Com um tema de contraste do Windows 11 ligado, o WebView2 troca `background-color`, `border-color` e `box-shadow` pelas cores do sistema e apaga gradientes. Não existe um `@media (forced-colors: active)` em `src/`, então: `.chave.ligada { background: var(--realce) }` some e a chave ligada fica igual à desligada; `.aba.ativa { box-shadow: inset 2px 0 0 }` some e a aba ativa desaparece do trilho; os seis `.amostra-de-tema .chao` viram retângulos iguais; as 30 `.mapa .celula` viram uma grade uniforme; a pílula, com fundo por `linear-gradient` e contorno por `box-shadow inset`, vira texto solto sobre o jogo.

Um bloco `@media (forced-colors: active)` em cada folha, escopado pela raiz dela. Em principal.css: `.chave { forced-color-adjust: none; background: Canvas; border-color: ButtonText } .chave.ligada { background: Highlight; border-color: Highlight } .chave .bolinha { background: ButtonText } .chave.ligada .bolinha { background: HighlightText }`; `.aba.ativa { forced-color-adjust: none; background: Highlight; color: HighlightText }`; `.amostra-de-tema { forced-color-adjust: none; border-color: ButtonText } .amostra-de-tema.escolhida { outline: 2px solid Highlight; outline-offset: 2px }`. Em diario.css: `.mapa .celula { forced-color-adjust: none; border: 1px solid CanvasText }` — visualização de dado é a exceção prevista pela spec. Em sobreposicao.css: `.pilula { background: Canvas; border: 1px solid CanvasText }` e `.prender { border-color: ButtonText }`. Em anel.css: `.anel > svg { forced-color-adjust: none }` e, em tokens.css, `--ring-track: color-mix(in srgb, CanvasText 30%, transparent)` dentro do mesmo media, para a trilha não virar um anel cheio.

**Pronto quando:** com o tema "Contraste #1" do Windows ativo, dá para dizer olhando qual chave está ligada, qual aba está ativa, qual tema está escolhido e quais dias do Diário tiveram jogo; a pílula tem contorno visível sobre uma cena branca e o anel continua com trilha e arco distinguíveis.

## T98. Alvos de 32px, anel de foco que acompanha a forma, e ações visíveis sem hover

**Onde:** `src/estilo/base.css`, `src/telas/principal.css`, `src/telas/painel.css`, `src/componentes/lista.css`, `src/componentes/historico.css`, `src/telas/sobreposicao.css`.

Com toque, erra-se: a chave tem 40×22, o X do painel 24×24, o X de esquecer 26×26, "7 dias"/"30 dias" uns 20px de altura, o cadeado da pílula 28×28, cada amostra de tema 40×28, e Esquecer/Cancelar `padding: 4px 10px`. O Windows 11 pede 32×32 como mínimo. Tab até a chave (raio 11px), o X de esquecer (círculo) ou uma amostra de tema (raio 8): o elemento ganha cantos de 4px enquanto está focado, porque `:focus-visible { border-radius: 4px }` em base.css sobrescreve o raio do próprio elemento — o outline do Chromium já acompanha o `border-radius` sozinho, a linha só estraga. E o X de esquecer nasce com `opacity: 0`, revelado só por `.item:hover` e `:focus-visible`; o nome do controle só ganha o sublinhado tracejado no hover. Em tela touch, onde não há hover, nada indica que dá para esquecer ou renomear.

O desenho fica e o alvo cresce. Em base.css, sai o `border-radius: 4px` de `:focus-visible`. Em principal.css, `.chave::before { content: ""; position: absolute; inset: -6px -4px }` (a chave já é `position: relative`; o alvo vira 48×34) e `.amostra-de-tema { height: 32px }`. Em painel.css, `.painel .fechar { width: 32px; height: 32px; top: 6px; right: 6px }`. Em lista.css, `.esquecer { width: 32px; height: 32px }`, `.confirmar .ciclo { min-height: 32px; padding: 6px 12px }`, `.lista.solta .esquecer { opacity: 0.5 }` para a lista do Resumo (a do painel não esquece nada), e `@media (hover: none) { .lista .esquecer { opacity: 1 } .lista .item-nome { border-bottom-color: var(--stroke-strong) } }`. Em historico.css, `.faixa { padding: 6px 10px; min-height: 32px }` — o `.historico-topo` cresce para 32px — e `.grafico { border-radius: 4px }`, que o foco da tarefa do gráfico vai usar. Em sobreposicao.css, `.prender { width: 32px; height: 32px }` com o `svg` mantido em 14px.

**Pronto quando:** medindo no DevTools, nenhum `<button>` da janela principal, do painel ou da pílula solta tem caixa de clique menor que 32×32 e a chave responde ao clique 6px acima ou abaixo do desenho; Tab pela chave, pelo X de esquecer e pelas amostras mostra o anel de foco redondo, sem o elemento mudar de forma; em modo tablet do Windows 11, o X de esquecer e o sublinhado de renomear aparecem sem toque prévio.

## T99. Procurar, Atualizar agora, Salvar diagnóstico e Salvar cartão mostram e anunciam o resultado

**Onde:** `src/componentes/Controles.tsx` (`Linha`), `src/telas/Configuracoes.tsx` (`Novidade`, linhas de versão e de diagnóstico), `src/telas/Diario.tsx` (`Exportar`, `rotuloDoBotao`), `src/telas/diario.css`.

Quem usa o Narrador clica em "Procurar", "Atualizar agora" ou "Salvar diagnóstico" e ouve silêncio: "Consultando o repositório…", "Nada novo desde esta versão", "baixando 43%", "instalando, o app reinicia sozinho" e "Salvo como diagnostico.txt" só mudam visualmente, porque o foco fica no botão e o texto que muda está no `.titulo`/`.descricao` da `Linha` ou no `.andamento .rodape`, que são `<div>` comuns, e o trilho de progresso é `aria-hidden`. No Diário, `passo` nunca volta a `parado`: depois do primeiro clique o botão diz "Salvo em Downloads" para sempre e continua clicável, gerando um arquivo por clique; em erro vira "Não deu", o rótulo do botão fazendo papel de mensagem, com o motivo embaixo. Como `.exportar` é uma coluna dentro de `.diario-cabeca`, o erro aumenta a altura do cabeçalho e empurra o heatmap.

`Linha` aceita `viva?: boolean`, que põe `role="status" aria-live="polite"` na `.descricao`; as linhas de versão e de diagnóstico passam `viva`. No cartão de versão nova, o `.andamento .rodape` e o bloco `.falha` ganham os mesmos atributos, e o trilho troca `aria-hidden` por `role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-label="Download da atualização"`, com `aria-valuenow={porcento}` só quando há porcentagem: indeterminado fica sem valor, que é o que ele é. No Diário, o botão só conhece dois rótulos, "Salvar cartão" e "Desenhando…", e `rotuloDoBotao` sai; um `useEffect` sobre `passo` agenda `setTimeout(() => setPasso("parado"), 4000)` quando ele é `salvo` ou `falhou`, e limpa no unmount. O resultado vai para um `<span className="exportar-aviso" role="status">` ao lado do botão, em 12px: "Salvo em Downloads" em `--text-secondary` ou "Não consegui salvar" em `--atencao`, com o motivo em `title`. `.exportar` vira `flex-direction: row; align-items: center; gap: var(--space-2)` e `.exportar-erro` sai.

**Pronto quando:** com o Narrador ligado, clicar em Procurar lê "Consultando o repositório" e depois o resultado; durante o download a porcentagem é lida a cada mudança; a falha dentro do cartão e o "Salvo em" do diagnóstico são lidos quando aparecem, sem mover o foco; quatro segundos depois de salvar ou falhar, o botão do Diário volta a "Salvar cartão" e o aviso some; salvar, falhar e mostrar o aviso não deslocam o heatmap um pixel.

## T100. Renomear ou esquecer um controle não joga o foco fora da página

**Onde:** `src/componentes/ListaDeControles.tsx`.

Enter no nome abre o campo; Enter ou Esc fecha o campo e o foco cai no `body`: o próximo Tab recomeça do início da janela. Clicar em "Cancelar" no Esquecer faz o mesmo, e depois de "Esquecer" o item some e o foco também. O `<input className="renomear">` é desmontado em `salvar()`/`aoSair()` sem nada devolver o foco ao `<button className="item-nome">` que o substitui; o `.esquecer` é desmontado ao entrar em `confirmando` e remontado ao cancelar, sem `focus()`.

`const nome = useRef<HTMLButtonElement>(null)` e `const esquecer = useRef<HTMLButtonElement>(null)`; em `salvar`/`aoSair`, `requestAnimationFrame(() => nome.current?.focus())`; ao cancelar a confirmação, `requestAnimationFrame(() => esquecer.current?.focus())`; ao confirmar Esquecer, o foco vai para o `.lista-titulo` (com `tabIndex={-1}`) antes do `invoke`. O campo `renomear` recebe `aria-label="Nome do controle"`.

**Pronto quando:** renomear com Enter, cancelar com Esc e cancelar o Esquecer deixam o foco visível no mesmo item; esquecer um controle deixa o foco em "Seus controles".

## T101. O painel abre dizendo o que é, e o passo a passo anda pelo teclado e anuncia o passo

**Onde:** `src/telas/Painel.tsx`, `src/telas/painel.css`, `src/telas/Passos.tsx` (handler de teclado, rodapé, pontos), `src/telas/passos.css`.

Clique no ícone da bandeja: o painel aparece e o Narrador fica mudo até o primeiro Tab. O percentual, que é o motivo de abrir, não é lido: `.painel` é um `div` sem `role`, sem `tabIndex`, sem foco inicial. No passo a passo, Tab começa nos botões minimizar e fechar da barra, não no Avançar; Enter não avança e Esc não pula. Quem volta ao passo 0 pelo Voltar perde o foco, porque o botão desmonta (`passo > 0 &&`), e o próximo Tab recomeça da barra. Na última tela o Pular vira um `<span />`. Os pontos são `aria-hidden` e não existe "Passo 2 de 6" em texto: o Narrador ouve só o título mudar, e às vezes nem isso, porque Avançar mantém o foco no botão.

Painel: `<div className="painel" role="dialog" aria-label={`Kontro, ${estado.textoDaCarga}, ${estado.nome}`} tabIndex={-1} ref={painel}>`, `painel.current?.focus()` a cada abertura, e em `painel.css` `.painel:focus { outline: none }`. Passo a passo: `const avancar = useRef<HTMLButtonElement>(null)` no botão Avançar/Começar e `useEffect(() => avancar.current?.focus(), [passo])`, o que no primeiro render já tira o Tab da barra de título. O handler global passa a tratar `Enter` com `evento.target === document.body`, clicando `avancar.current`, e `Escape`, que faz o mesmo que o botão Pular. A seta direita continua parando no último passo: com o foco em Começar, Enter termina. Voltar é sempre renderizado, com `disabled={passo === 0}`, e Pular no último passo recebe a classe `oculto`, com `visibility: hidden`, em vez de virar um `<span />`: ninguém que tinha foco desmonta. Ao lado dos pontos entra `<span className="contador" aria-live="polite">Passo {passo + 1} de {TOTAL}</span>`, em 12px `--text-tertiary`.

**Pronto quando:** ao abrir o painel o Narrador lê "Kontro, 77%, Xbox Wireless Controller, diálogo", e Esc fecha; abrir o passo a passo e apertar Enter seis vezes termina; Esc faz o que Pular faz; o anel de foco está sempre num botão do rodapé depois de cada troca; "Passo N de 6" aparece e é lido pelo Narrador.

## T102. A posição da pílula se ajusta pelo teclado

**Onde:** `src/componentes/Controles.tsx`, `src/telas/Configuracoes.tsx`, `src/telas/principal.css`.

Quem não usa mouse solta e prende a pílula pelo atalho, mas nunca a move: `MiniTela` é um `<span aria-hidden="true">` só ilustrativo, e a janela da pílula nasce `focused(false)` e só responde a `startDragging`. O Rust já reposiciona: `salvar_configuracoes` em lib.rs chama `janelas::posicionar_sobreposicao` sempre que a config muda com a pílula presa.

`MiniTela` aceita `aoMover?: (x: number, y: number) => void`; com ela, vira `<button type="button" aria-label={`Posição da pílula: ${Math.round(x * 100)}% da largura, ${Math.round(y * 100)}% da altura`}>` com `onKeyDown`: setas movem 0,05 (Shift: 0,01), Home/End/PageUp/PageDown vão aos cantos, tudo limitado a [0, 1]. Em Configuracoes.tsx, a linha "Posição" passa `aoMover={(x, y) => gravar({ OverlayX: x, OverlayY: y })}` só quando a pílula está presa — solta, a descrição já manda arrastar, e o Rust sobrescreve a posição ao prender. Em principal.css, `button.mini-tela { padding: 0 }` e `.mini-tela:focus-visible { border-color: var(--realce) }`.

**Pronto quando:** Tab até a mini-tela e setas movem a pílula de verdade na tela em passos de 5%, com o rótulo lido pelo Narrador atualizando a cada tecla.

---

# Movimento 16 — Sessões e controles

A área hoje é o fim da página Resumo: a lista de cinco sessões dentro do cartão do gráfico, abaixo da saúde, e a lista de controles solta embaixo. Funciona, mas quem usa não sabe qual sessão abriu no gráfico, não descobre que dá para renomear, apaga trinta dias de histórico com um X sem aviso, e lê tudo isso num cinza que não passa no contraste do próprio DESIGN.md. Vira: sessões num cartão próprio que rola até o gráfico ao ser clicado, com o cartão de 56px que a T10 desenhou, estado escolhido com o mesmo traço de realce da aba ativa, lápis e lixeira visíveis sem hover, confirmação que diz o que apaga e fecha sozinha, e um selo dizendo qual controle está na bandeja.

## T103. Clicar numa sessão mostra onde ela foi parar

**Onde:** `src/telas/Resumo.tsx`, `src/componentes/Sessoes.tsx`, `sessoes.css`.

Na janela padrão de 840×600, o cartão do gráfico com a saúde ocupa a tela inteira e as sessões ficam abaixo da dobra. A pessoa rola, clica numa sessão, e o que vê é uma linha escurecendo: o gráfico trocou 400px acima e nada leva o olhar até lá. E mesmo com o gráfico à vista, a sessão escolhida não se distingue da que está sob o mouse: `.escolhida` e `:hover` usam o mesmo `background: var(--surface-alt)`, a diferença é só `border-color: var(--stroke-strong)`, um tom acima do fundo. O botão não expõe `aria-pressed`, e o `title` diz "Ver esta sessão no gráfico" mesmo quando o clique vai fechar.

Em `Resumo.tsx`, `<Sessoes>` sai de dentro do cartão do gráfico e ganha `<section className="cartao">` próprio logo abaixo; em `sessoes.css`, a raiz `.sessoes` perde `margin-top`, `padding-top` e `border-top`, porque o cartão já separa. O cartão do gráfico recebe um `useRef`, e em `aoEscolher`, quando a sessão abre (não quando fecha), vem `ref.current.scrollIntoView({ block: "nearest", behavior: "smooth" })`. `.pagina` é quem rola, e movimento reagindo a clique não fere a regra de loop. Para o estado: `aria-pressed={escolhida === s.inicio}` no botão, `.sessoes button.sessao.escolhida { box-shadow: inset 2px 0 0 var(--realce); }` e `.escolhida .sessao-titulo { color: var(--realce); }`, o mesmo traço que `.aba.ativa` usa no trilho. O hover fica como está e o `title` sai: com traço, cor e `aria-pressed`, o tooltip nativo do WebView2 só atrapalha.

**Pronto quando:** com a janela em 600px de altura, clicar numa sessão abaixo da dobra rola até o gráfico dela ficar inteiro na tela; as sessões aparecem como cartão próprio; com o mouse fora da lista, a sessão aberta tem o traço de realce à esquerda e o nome na cor de interface; o Narrador do Windows lê "pressionado" nela; nenhum tooltip nativo aparece.

## T104. Esquecer um controle diz o que apaga, e a confirmação não fica aberta para sempre

**Onde:** `src/componentes/ListaDeControles.tsx`, `lista.css`. Referência do que se perde: `monitor.rs` (`esquecer`) e `historico.rs` (`History::esquecer`).

O botão de esquecer é um X cinza que só aparece no hover (`.esquecer { opacity: 0 }`, `1` em `.item:hover`); com caneta ou tela de toque ele não existe. Clicado, surgem "Esquecer" e "Cancelar" em 12,5px, sem dizer que `monitor.esquecer` chama `historico.esquecer(chave)` e remove a série de 30 dias, as sessões e a saúde daquele controle, sem desfazer. `confirmando` só volta a `false` pelo botão Cancelar: quem clica no X e vai embora deixa os dois botões abertos indefinidamente. O X é semântica de fechar, não de remover.

A lixeira fica visível sempre: `.esquecer { opacity: 0.55 }` no repouso, `1` no hover e no `:focus-visible`, com um glifo de lixeira de 12px no lugar do X. Enquanto `confirmando`, o `item-estado` troca para "Some da lista e leva o histórico e as sessões junto." O `.confirmar` vira o dono do foco: `Escape` cancela, e um `onBlur` que confere `e.relatedTarget` fora de `e.currentTarget` também cancela. Os dois botões usam a mesma medida dos outros `.ciclo` (`padding: 6px 12px; font-size: 13px`), e `.ciclo.perigo` já nasce preenchido, `background: var(--red); color: var(--ink)`, para ser o botão de perigo e não um contorno que só enche no hover (5,7:1 no Noite, 5,3:1 no Dia). O `title` e o `aria-label` "Esquecer este controle" ficam só como `aria-label`.

**Pronto quando:** sem mouse, a lixeira é visível em todo controle desconectado do Resumo; ao pedir para esquecer, a linha diz o que será apagado; clicar fora ou Esc cancela; e o botão de confirmar é o único vermelho preenchido da tela.

## T105. A sessão mede os 56px que a T10 desenhou

**Onde:** `src/componentes/sessoes.css`, `src/componentes/Sessoes.tsx`, `src/componentes/lista.css`.

A T10 pediu cartão de 56px com ícone de 32. O CSS entregou uma linha de ~44px: `padding: var(--space-2)`, `.sessao-icone` de 28×28 com `border-radius: 6px`, e o `<img width={24}>` reduz o PNG que `icone_do_jogo.rs` extrai com `SHGFI_LARGEICON` (32px), com `image-rendering: -webkit-optimize-contrast` para disfarçar. Ao lado, `gap: 1px`, `padding: 7px 8px`, `border-radius: 4px` e `margin-top: 2px` em `lista.css` estão fora das escalas 4/8/12/16 e 8/12/16 do §5. E a coluna da direita serrilha: `taxa()` devolve `""` abaixo de 20 min ou sem descarga, o `<span className="sessao-taxa">` vazio não gera caixa de linha, e o `align-items: center` da `.sessao` desce o "−3%" para o meio enquanto nas vizinhas o "−18%" fica no topo.

`.sessao { min-height: 56px; padding: var(--space-2) var(--space-3); }`, `.sessao-icone { width: 32px; height: 32px; border-radius: var(--radius-field); }`, `img` em 32×32 sem `image-rendering`. `.sessao-texto` e `.sessao-numeros` com `gap: var(--space-1)`. O span da taxa só é renderizado quando há taxa, e `.sessao-numeros` passa a `align-self: stretch; justify-content: flex-start` com a mesma linha de base do texto, para a porcentagem ficar sempre no topo. Em `lista.css`: `.item { padding: var(--space-2); }`, `.item-estado { margin-top: var(--space-1); }`, `.renomear { border-radius: var(--radius-field); }`. O `.item` também aparece no flyout, então a mudança de 7 para 8px vale lá.

**Pronto quando:** cada sessão mede 56px, o PNG do jogo é exibido em 32px sem reescala, cinco sessões seguidas com e sem taxa têm as porcentagens numa reta vertical, e nenhum padding, gap ou raio nesses dois CSS está fora da escala.

## T106. O ícone da sessão para de significar três coisas

**Onde:** `src/componentes/Sessoes.tsx` (`Icone`), `sessoes.css`.

Uma sessão sem jogo mostra o glifo do controle num quadrado cinza. Uma sessão de jogo cujo exe sumiu mostra exatamente o mesmo quadrado: `Icone` cai em `<Glifo>` tanto para `jogo === null` quanto para `uri === null` com jogo conhecido. Dez pixels abaixo, o mesmo glifo dentro do anel significa "um controle". E na primeira montagem `uri` nasce `null`: o glifo aparece e é trocado pelo PNG quando o `invoke("icone_do_jogo")` volta, um piscar a cada abertura do Resumo.

Com jogo e sem ícone, o quadrado mostra a inicial do nome (`jogo.trim()[0].toUpperCase()`) em `--font-display` 14px 600 e `--text-secondary`, como o Windows faz com app sem ícone. Sem jogo, `.sessao-icone.sem-jogo { background: transparent; }` e o glifo a 60% de opacidade. `uri` passa a `useState<string | null | undefined>(undefined)`, e enquanto for `undefined` o quadrado sai vazio, para nada trocar de forma depois de aparecer.

**Pronto quando:** uma sessão de jogo desinstalado mostra a letra do jogo, uma sessão sem jogo mostra o glifo sem caixa, e ao abrir o Resumo nenhum ícone troca de forma depois de aparecer.

## T107. O nome do controle avisa que dá para renomear, e o campo não muda a linha de lugar

**Onde:** `src/componentes/lista.css`, `src/componentes/ListaDeControles.tsx`.

O nome parece texto. Só no hover aparece um sublinhado tracejado em `--stroke-strong`, quase invisível. Ao clicar, o campo vem com `border: 1px solid var(--accent-teal)` em todo tema, inclusive no Ardósia, onde a interface é azul (a T16 separou `--realce` da cor de carga e este ponto ficou para trás), e a linha cresce uns 6px porque `.renomear` tem `padding: 2px 6px` e borda onde `.item-nome` tem `padding: 0`.

O sublinhado tracejado sai. `.item-nome` vira `display: inline-flex; align-items: center; gap: var(--space-1); max-width: 100%`, com o texto num `<span>` que corta com reticências e um lápis SVG de 12px ao lado, em `--text-tertiary` a `opacity: 0.55`, `1` no hover do `.item`. Um `::after` não serve: o botão já tem `overflow: hidden` e `text-overflow: ellipsis`, e o lápis seria a primeira coisa a ser cortada. `.renomear { border-color: var(--realce); height: 22px; margin: -2px -7px; padding: 0 6px; }` para ocupar a mesma caixa do nome. O `title="Renomear"` vira `aria-label={\`Renomear ${controle.nome}\`}`. Vale também no flyout, que usa a mesma lista.

**Pronto quando:** entrar e sair do modo de edição não muda a altura da linha; o campo usa a cor de realce do tema ativo; um lápis fica visível ao lado do nome sem hover; e nenhum tooltip nativo aparece no nome.

## T108. A linha do controle diz qual está na bandeja e para de mostrar "--"

**Onde:** `src/componentes/ListaDeControles.tsx` (`Linha`), `lista.css`. Referência: `modelo.rs` (`texto_da_carga`, `conectado_sem_carga`).

Com dois controles, o principal tem a borda em `--stroke-strong` e os outros em `--stroke`. É uma diferença que só se nota comparando lado a lado, e nada diz o que ela significa: a pessoa não sabe qual dos dois a bandeja e a pílula estão mostrando. Na mesma linha, um controle plugado que não informa carga aparece como "-- · no cabo", porque `texto_da_carga` devolve `--` sem percentual nem nível e `Linha` concatena `${textoDaCarga} · ${textoDaLigacao}` sem olhar `conectadoSemCarga`. Dois traços parecem dado quebrado. O cartão de estado logo acima já trata o caso com "conectado no cabo · não informa bateria", e o §7 manda mostrar "No cabo" sem número.

Quando `principal`, o nome ganha à direita `<span className="item-selo">na bandeja</span>`, com `.item-selo { flex: 0 0 auto; font-size: 11px; line-height: 16px; padding: 0 var(--space-2); border-radius: 999px; color: var(--realce); border: 1px solid color-mix(in srgb, var(--realce) 40%, transparent); }`, e `.item.principal { box-shadow: inset 2px 0 0 var(--realce); }`, a mesma linguagem da aba ativa. No `item-estado`, quando `controle.conectadoSemCarga`, sai só `controle.textoDaLigacao` ("no cabo" ou "carregando"); o ramo desligado e o ramo com número continuam iguais.

**Pronto quando:** com dois ou mais controles, o que está na bandeja tem o selo e o traço de realce, e ao trocar o principal o selo muda de linha; nenhum "--" aparece na lista em nenhuma via.

## T109. A lista de sessões diz quantas tem, mostra mais e não some vazia

**Onde:** `src/componentes/Sessoes.tsx`, `sessoes.css`.

Quem acabou de instalar vê o gráfico dizendo "sem histórico nesta janela" e a saúde dizendo que precisa de duas semanas, mas nenhuma pista de que sessões existem: `if (sessoes.length === 0) return null` esconde o título junto com a lista, e a regra de dez minutos vive em `DURACAO_MINIMA_DE_SESSAO_MS` no Rust sem nunca chegar à tela. Depois de uma semana, o oposto: `QUANTAS_MOSTRAR = 5` com `slice(0, 5)` trava a lista, e a sessão de sábado passado, a que vale comparar, não está em lugar nenhum do app.

O título fica sempre. Vazia, a lista vira `<div className="sessoes-vazio">A primeira sessão aparece depois de 10 min com o controle ligado.</div>`, com `.sessoes-vazio { padding: var(--space-3) 0; font-size: 12px; color: var(--text-tertiary); }`, alinhado com `.historico.vazio`. O número é a constante real do Rust. Com sessões, um estado `quantas` começa em 5; `.sessoes-titulo` passa a `display: flex; justify-content: space-between; align-items: center` e mostra à direita `{Math.min(quantas, sessoes.length)} de {sessoes.length}` em `--font-mono` 11px. Quando `sessoes.length > quantas`, um `<button className="mais">mais 5</button>` no rodapé da lista soma 5 por clique. `.faixa` está escopado em `.historico` e não pode ser reusado; `.sessoes .mais` repete a receita com `padding: var(--space-1) var(--space-2); border: 0; border-radius: var(--radius-field); background: transparent; font-size: 11.5px; color: var(--text-tertiary)`.

**Pronto quando:** com `history.json` vazio, o Resumo mostra "Últimas sessões" e a frase de espera; com 12 sessões, o título diz "5 de 12", clicar em "mais 5" mostra dez, e o botão some quando todas estão na tela.

---

# Movimento 17 — Painel e moldura da janela

A área hoje é um flyout que funciona e uma moldura que quase engana: o painel abre no lugar certo, com Acrylic, mas reabre quando a pessoa clica no ícone para fechar, só anima na primeira vez, pode nascer maior que a tela e leva um X que nenhum flyout do Windows tem. A janela principal tem barra de 38 px sem maximizar, sem Snap Layouts, sem estado inativo, com uma marca de 17 px que carrega o disco preto do ícone do app, e pisca branca ao abrir em máquina sem material. O que vira: o flyout se comporta como os do Windows 11 (abre e fecha pelo ícone, desliza a cada abertura, cabe na tela) e ganha o atalho da pílula; a moldura passa a ter as medidas, os botões e o esmaecimento da barra nativa; a janela acompanha o Windows no tema e na transparência no mesmo segundo; e as três folhas da área usam só os cinco tamanhos do DESIGN §4.

## T110. A moldura vira a do Windows 11: 32 px, maximizar, esmaece inativa, arrasta pela marca

**Onde:** `src/componentes/BarraDeTitulo.tsx`, `src/telas/principal.css`, `src/estilo/tokens.css`, `src-tauri/capabilities/principal.json`.

Com outra janela em foco, a barra do Kontro continua com o mesmo peso e cor de quando está ativa: no Windows 11 toda barra inativa esmaece título e botões, e com Mica a diferença é ainda mais esperada, porque o próprio Mica esmaece o chão da janela inativa. Nada escuta `onFocusChanged` na principal; o painel escuta, mas só para se esconder. A barra tem 38px com botões 46×38, e o sistema usa 32 e 46×32. "Kontro" está em 12,5px semibold com tracking, aparência de site. Não há botão de maximizar, embora a janela seja redimensionável e maximizável por duplo clique. Arrastar em cima do ícone não move a janela: o `drag.js` do Tauri 2.11 só aceita o atributo sem valor quando o elemento clicado é o próprio (`el === composedPath[0]`), e o `<svg>` da `Marca` não tem o atributo nem é `HTMLElement`. O foco nos botões da janela empurra o anel para fora da barra. E o hover do fechar usa `--red`, que é carga crítica.

Em `BarraDeTitulo.tsx`: os três `data-tauri-drag-region` viram um só, `data-tauri-drag-region="deep"` no `<header>`; entra o botão "Maximizar" entre minimizar e fechar (glifo quadrado de 10px, dois quadrados sobrepostos quando `isMaximized()`, atualizado em `onResized`), chamando `janela.toggleMaximize()`; `principal.json` libera `core:window:allow-toggle-maximize` e `core:window:allow-is-maximized`; um `useEffect` assina `getCurrentWindow().onFocusChanged` e grava `document.body.dataset.foco = "sim" | "nao"`; cada botão ganha `aria-label`. Em `principal.css`: `.barra { height: 32px }`, `.marca { font-size: var(--fs-caption); font-weight: 400; letter-spacing: 0; color: var(--text-primary) }`, `body[data-janela="principal"][data-foco="nao"] .marca` e `… .botao-janela` em `color: var(--text-tertiary)` com `transition: color var(--motion-fast) var(--curva)`, e `.botao-janela:focus-visible { outline-offset: -2px }`. O hover do fechar ganha `--fechar-janela: #c42b1c` em `tokens.css`, o vermelho de caption do Windows, com glifo `#fff`.

**Pronto quando:** clicar em outro app esmaece "Kontro" e os três botões em 120 ms, e voltar o foco restaura; a barra mede 32px e os botões 46×32 a 100%; arrastar sobre o ícone move a janela; o botão de maximizar alterna e troca de glifo; Tab pelos botões da janela mantém o anel de foco dentro da barra.

## T111. Snap Layouts sobre o botão de maximizar

**Onde:** `src-tauri/src/janelas.rs` (`criar_principal`), `src/componentes/BarraDeTitulo.tsx`, `src/telas/principal.css`.

Passar o mouse onde o Windows 11 mostra o Snap Layouts não mostra nada. O DWM só oferece o menu quando `WM_NCHITTEST` devolve `HTMAXBUTTON` sobre o botão, e nada no Rust responde a isso; com `decorations(false)` o tao devolve `HTCLIENT` para tudo que não é borda.

`criar_principal` subclassifica a janela com `SetWindowSubclass` (windows-rs, `Win32_UI_Shell`, já na lista de features) com id 1 — o tao usa o 0. No `WM_NCHITTEST`, converte o ponto para coordenadas de cliente; se cai no retângulo do botão de maximizar (46×32 lógicos vezes a escala, encostado à esquerda do fechar), devolve `HTMAXBUTTON`; senão `DefSubclassProc`. Como sobre `HTMAXBUTTON` o webview deixa de receber o mouse, a subclasse também trata `WM_NCLBUTTONUP` com esse hit alternando maximizar, e `WM_NCMOUSEMOVE`/`WM_NCMOUSELEAVE` emitindo `kontro://maximizar-hover` com `true`/`false`; `BarraDeTitulo.tsx` escuta e liga `data-hover` no botão, e `principal.css` pinta esse estado igual ao `:hover` dos outros.

**Pronto quando:** passar o mouse sobre o botão de maximizar abre o Snap Layouts do Windows 11 e o botão acende; clicar nele maximiza; duplo clique e arrastar seguem funcionando em qualquer ponto da barra.

## T112. A marca da barra é a da bandeja, não o ícone do app encolhido

**Onde:** `src/componentes/Marca.tsx`, `src/componentes/BarraDeTitulo.tsx`, `src/telas/principal.css`.

No tema Dia a barra clara ganha um ponto preto (`#0F1318`) ao lado de "Kontro". Em qualquer tema, a 17 px o anel com gradiente verde→teal vira uma mancha — exatamente o que o DESIGN §2 proíbe para 16 px. `miudo` escolhe a geometria da bandeja mas continua pintando o círculo de fundo (`CORES.fundo` cravado) e o `stroke="url(#marca-kontro)"`; o id do gradiente é fixo, e uma segunda `Marca` na mesma página duplicaria o id.

`Marca.tsx`: quando `miudo`, não desenha o círculo de fundo (§3, "a bandeja é o fundo"), usa `stroke={CORES.verde}` plano e `fill="currentColor"` no controle, para acompanhar o texto ao lado; o id do gradiente vem de `useId()`. `BarraDeTitulo.tsx` passa `tamanho={16}`, e `.marca` já fica em `--text-primary` pela tarefa da moldura, então o glifo herda a cor certa em cada tema.

**Pronto quando:** no tema Dia a barra não tem disco escuro; no Noite a marca da barra é indistinguível do ícone da bandeja ao lado.

## T113. O X avisa uma vez que o app continua na bandeja

**Onde:** `src/componentes/BarraDeTitulo.tsx`, `src-tauri/src/lib.rs` (`on_window_event`), `src-tauri/src/configuracoes.rs` (`Settings`, `CAMPOS_DA_CONFIG`), `src/config.gerada.ts`.

Na primeira vez que a pessoa clica no X, a janela some e o app continua rodando sem sinal nenhum. Quem não sabe que é app de bandeja acha que fechou, e o tooltip "Fechar" confirma o engano. `CloseAction` padrão é `MinimizeToTray` e o `on_window_event` faz `prevent_close` e `hide()` em silêncio.

`BarraDeTitulo.tsx` lê `useConfig()` e, com `CloseAction === "MinimizeToTray"`, usa `title` e `aria-label` "Fechar · o Kontro continua na bandeja". No `lib.rs`, na primeira ocultação pelo X, `app.notification().builder().title("O Kontro continua na bandeja").body("Clique no ícone para ver a bateria.").show()` e grava `tray_hint_shown: bool` em `Settings`, com a entrada `("TrayHintShown", "boolean")` em `CAMPOS_DA_CONFIG` para o teste que confere o `config.gerada.ts` passar.

**Pronto quando:** o primeiro X mostra o toast; o segundo não; passar o mouse no X diz que o app fica na bandeja.

## T114. O flyout liga e desliga a pílula sem abrir a janela

**Onde:** `src/telas/Painel.tsx`, `src/telas/painel.css`, `src/ajustes.ts`, `src/telas/Configuracoes.tsx`.

Para ligar ou desligar a pílula em jogo são três cliques: flyout, Configurações, rolar até a linha. Os flyouts do Windows 11 (rede, bateria, som) expõem os controles rápidos na própria janela. O flyout só tem "Configurações" e "Atualizar", como o §7 desenhou antes de a pílula existir; `ciclar`, `salvar` e `useConfig()` já fazem a troca do `OverlayMode` em `Configuracoes.tsx`.

`MODOS` sai de `Configuracoes.tsx` para `ajustes.ts`. `Painel.tsx` lê `useConfig()` e, acima de `.acoes`, renderiza `.rapidos` com um chip "Pílula · Só em jogo" que chama `salvar(cfg, { OverlayMode: ciclar(cfg.OverlayMode, ["Desligada", "EmJogo", "Sempre"]) })`. `painel.css`: `.painel .rapidos { display: flex; gap: var(--space-2); margin-top: var(--space-3) }` e o chip com o mesmo desenho de `.acoes button`, em Caption 12 — o `.ciclo` do `principal.css` é escopado à janela principal e não chega aqui. `Configuracoes.tsx` passa a escutar `kontro://config`, que o Rust já emite em `salvar_configuracoes`, para a linha "Pílula em jogo" refletir a troca.

**Pronto quando:** clicar no chip alterna Desligada → Só em jogo → Sempre sem abrir a janela principal, e a linha em Configurações mostra o valor novo na hora.

---

# Movimento 18 — Diário

O Diário hoje é um cartão com o total de horas, um botão de salvar e trinta quadrados em três linhas de dez, seguido de um ranking em texto com barra verde. A grade não diz que dia é cada célula, o dia só se explica pelo tooltip nativo do WebView2, o ranking esconde os jogos em medição, e o PNG exportado pinta tudo de verde e assina com texto. Vira um calendário de segunda a domingo com legenda, leitura própria e teclado; um ranking com ícone, horas e os jogos que ainda estão medindo; um cartão exportado que é a tela naquele momento; e uma aba que abre sem piscar e acompanha a sessão que está acontecendo.

## T115. O heatmap vira calendário

**Onde:** `src/telas/Diario.tsx` (`porDia`, `Mapa`), `src/telas/diario.css`.

`.mapa` é `grid-template-columns: repeat(10, 1fr)` e `porDia()` devolve 30 dias corridos a partir de hoje. Trinta quadrados em três linhas de dez não respondem "joguei no fim de semana?" nem "isso foi em agosto?": não há letra de dia, nome de mês nem marca de hoje, e a célula estica com a largura do cartão apesar de a T11 pedir tamanho fixo.

`Mapa` passa a desenhar uma grade de `auto repeat(7, 28px)` com `gap: var(--space-1)`: a coluna 0 é o mês, as sete seguintes são segunda a domingo. Antes do primeiro dia entram `(new Date(dias[0].quando).getDay() + 6) % 7` células de enchimento com `visibility: hidden`, para que cada linha seja uma semana. Uma linha de cabeçalho `S T Q Q S S D` fica acima, em `--font-mono` 12px `--text-tertiary`, e a coluna 0 mostra a abreviação do mês (`toLocaleDateString("pt-BR", { month: "short" })` sem o ponto, mesmo estilo) na primeira linha e em cada linha que contém um dia 1. `.celula` troca `aspect-ratio: 1` por `width: 28px; height: 28px; border-radius: var(--radius-field)`. A célula de hoje ganha a classe `hoje` com `box-shadow: 0 0 0 1px var(--surface), 0 0 0 2px var(--realce)`, cor de interface, não de carga.

**Pronto quando:** cada linha da grade é uma semana de segunda a domingo, as letras dos dias estão acima, o mês aparece à esquerda quando muda, a célula tem 28px em qualquer largura de janela e a de hoje tem contorno na cor de realce.

## T116. O dia se explica na própria página, e o teclado chega nele

**Onde:** `src/telas/Diario.tsx` (`Mapa`, `legenda`), `src/telas/diario.css`.

A célula é um `<div title={legenda(d)}>`: passar o mouse não muda nada visível, e um segundo depois abre a caixinha nativa do WebView2, fora do tema, dizendo "08/09 · 2 h 40 min · FC 26", sem dia da semana e sem a carga que pintou a célula de âmbar. A T11 prometeu "passar o mouse num dia mostra data, horas e jogos": só com mouse, e só depois do atraso do tooltip. `legenda()` nunca lê `menorCarga`, então quem não distingue verde de âmbar não tem número (DESIGN §8). `div` não entra na ordem de tabulação e `title` em elemento não focável não é lido: teclado e leitor de tela não chegam a célula nenhuma.

A grade vira `<div className="mapa" role="group" aria-label="Últimos 30 dias">`, e cada célula `<button type="button" aria-label={legenda(d)}>`, sem `title`, com tabindex itinerante: só o dia de hoje tem `tabIndex={0}`, setas esquerda e direita movem o foco um dia, cima e baixo uma semana, Home e End vão ao primeiro e ao último dia. `legenda()` sai como `seg, 08/09 · 2 h 40 min · terminou em 18%, carga baixa · FC 26`, com `weekday: "short"` sem o ponto e, quando há carga, `terminou em ${menorCarga}%` mais a palavra do `faixa()` que já existe no arquivo ("com folga", "carga baixa", "carga crítica"). A leitura vai para a tela como o `Historico` faz com `.historico-rodape`: `Mapa` guarda `sob: Dia | null` por `onMouseEnter` e `onFocus` e limpa em `onMouseLeave` e `onBlur`, e uma linha `.mapa-rodape` com `aria-live="polite"` e `min-height: 20px`, logo abaixo da grade, mostra a data e a duração em `--text-primary` 14px e o resto em 12px `--text-secondary`. Sem nada sob o cursor, a mesma linha mostra a legenda de cores: três pontos de 8px em `--accent-green`, `--amber` e `--red` com `terminou com ${aviso}% ou mais · abaixo de ${aviso}% · abaixo de ${critico}%`, lendo `useLimiares()`. Em `diario.css`, `.mapa .celula { border: 0; padding: 0 }` (a `.vazia` mantém a borda), `outline: 1px solid transparent; outline-offset: 1px; transition: outline-color var(--motion-fast) var(--curva)` e `.celula:hover { outline-color: var(--text-secondary) }`; o foco de teclado é o `:focus-visible` global, 2px em `--realce`, com `outline-offset: 2px`. Nada de `transform`.

**Pronto quando:** Tab chega ao dia de hoje na grade e as setas percorrem os outros 29; passar o mouse ou focar uma célula mostra, sem atraso e no tema do app, a data com dia da semana, as horas, a carga em que o dia terminou e os jogos, e o Narrador lê a mesma legenda; o contorno de hover aparece em 120 ms sem mudar o tamanho da célula; a legenda de cores muda o número quando o limiar é alterado nas Configurações.

## T117. Quatro degraus de brilho, com a escala embaixo

**Onde:** `src/telas/Diario.tsx` (`Mapa`, `desenharCartao`), `src/telas/diario.css`.

A opacidade é `0.25 + 0.75 * (d.minutos / maior)` na tela e `0.3 + 0.7 * ratio` no cartão: duas escalas contínuas e diferentes. Com um dia de 8 h no mês, um dia de 15 min fica a 27% e some ao lado da célula vazia; no tema Dia, âmbar a 27% sobre branco desaparece. E nada diz que o brilho significa duração.

Entra `nivel(minutos, maior): 0 | 1 | 2 | 3 | 4`, zero para dia sem jogo e `Math.min(4, Math.ceil((minutos / maior) * 4))` para os outros, usada na tela e no canvas. Na tela vira classe: `.celula.n1 { opacity: .35 } .n2 { opacity: .55 } .n3 { opacity: .78 } .n4 { opacity: 1 }`, e o `style={{ opacity }}` inline sai. No canvas, os mesmos quatro valores numa constante `BRILHO_POR_NIVEL` no lugar do `0.3 + 0.7 * ratio`. No lado direito da `.mapa-rodape`, uma escala de cinco quadrados de 10px, do vazio (borda `--stroke`) ao `n4`, entre "menos" e "mais" em Caption `--text-tertiary`; ela fica fixa enquanto o lado esquerdo alterna entre legenda de cores e leitura do dia.

**Pronto quando:** um dia de 15 minutos é visivelmente diferente do dia vazio nos seis temas, a escala de cinco quadrados aparece embaixo da grade, e a tela e o PNG usam os mesmos quatro degraus.

## T118. O ranking mostra o jogo inteiro, e para de ser verde

**Onde:** `src/telas/Diario.tsx` (`Ranking`, `desenharCartao`), `src/telas/diario.css`, `src/componentes/Sessoes.tsx`, `src/componentes/sessoes.css`, arquivos novos `src/componentes/IconeDoJogo.tsx` e `icone-do-jogo.css`.

Três coisas erradas na mesma lista. A barra do jogo mais faminto é `--accent-green`, a cor que no app inteiro significa bateria boa, e no PNG o `%/h` também sai em verde. Cada linha é só texto, embora o `Resumo` já mostre o ícone do jogo (T9) e `porJogo()` já calcule `minutos` que nunca aparece. E com `teto = firmes[0].porHora` um único jogo firme desenha uma barra 100% que não compara nada, enquanto os `medindo` são só contados: quem jogou FC 26 duas vezes não vê o nome do próprio jogo na página de jogos.

`Icone` sai de `Sessoes.tsx` para `src/componentes/IconeDoJogo.tsx`, levando as regras de `.sessao-icone` para `icone-do-jogo.css` com raiz `.icone-do-jogo`; `Sessoes` passa a importar de lá. `.faminto` vira `grid-template-columns: 28px minmax(0, 1fr) 120px auto auto` com o ícone na primeira coluna, e `.faminto-quantas` mostra `${duracao(f.minutos)} · ${f.sessoes} sessões`. `.faminto-barra span` passa a `background: var(--realce)` e o valor no canvas passa a `cor("--text-secondary")`. Quando `firmes.length === 1` o `span` da barra não é desenhado. Os `medindo` entram abaixo dos firmes com a mesma grade: no lugar da barra, três pontos de 6px (`.medicao span`), `f.sessoes` deles preenchidos em `--text-tertiary` e os outros só com `border: 1px solid var(--stroke-strong)`; a coluna da taxa diz "medindo" em `--text-tertiary` e a última, `${f.sessoes} de 3 sessões`. O rodapé "Mais N em medição" sai. O texto de vazio fica só para `famintos.length === 0` e passa a dizer como o nome chega, que é o que `jogo.rs` exige: "Nenhuma sessão trouxe o nome do jogo ainda. O nome é gravado quando o jogo roda em tela cheia com o controle ligado."

**Pronto quando:** cada linha tem o ícone de 24px igual ao das sessões e as horas totais ao lado da contagem; no tema Ardósia a barra é azul e em nenhum tema o jogo mais faminto aparece verde; um jogo com 2 sessões aparece pelo nome com 2 de 3 pontos preenchidos; e com um único jogo firme nenhuma barra cheia é desenhada.

## T119. Os números do mês sobem para a tela

**Onde:** `src/telas/Diario.tsx` (`Diario`, `numerosDoMes`), `src/telas/diario.css`.

O cabeçalho diz "43 h 2 min · de jogo em 30 dias" e mais nada. Dias com jogo, sessão mais longa e número de sessões já saem de `numerosDoMes()`, mas só quando o PNG não tem ranking. A tela que deveria dar vontade de mostrar tem menos informação do que a imagem que ela gera.

`numerosDoMes(dias, sessoes)` deixa de depender de `Retrato` e passa a contar só as sessões dentro da janela (`s.inicio >= dias[0].quando`), porque `sessoes_do_controle` pode trazer sessões mais velhas que os 30 dias e um número que inclui elas seria inventado. Abaixo de `.diario-cabeca` entra uma linha `.diario-numeros` com três pares: `18 de 30` "dias com jogo", `2 h 40 min` "sessão mais longa", `23` "sessões". Valor em `--font-mono` 14px `--text-primary`, rótulo em Caption 12px `--text-tertiary`, `gap: var(--space-5)`, `margin-bottom: var(--space-4)`. `desenharCartao` usa a mesma função.

**Pronto quando:** os três números aparecem entre o total de horas e o calendário e batem com o cartão gerado no mesmo instante.

## T120. A aba abre sem piscar, acompanha o jogo e nunca some inteira

**Onde:** `src/telas/Diario.tsx` (`Diario`, `Exportar`), `src/telas/diario.css`.

`useState<Sessao[]>([])` faz `totalMinutos === 0` ser verdade antes de o `invoke` responder: ao clicar em Diário aparece "Ainda não há sessão suficiente" por um frame e some. O `useEffect` tem `[]` de dependência, então quem joga com a janela aberta nunca vê a sessão de hoje entrar, enquanto o `Resumo` refaz a consulta em `[estado?.chave, estado?.percentual]`. E na primeira semana de uso a página inteira é substituída por um parágrafo cinza: a pessoa não vê que existe uma grade esperando ser preenchida.

`sessoes` vira `useState<Sessao[] | null>(null)`, e enquanto for `null` a página renderiza só o `<h1>`. O efeito recebe `const estado = useEstado()` e as mesmas dependências do `Resumo`. Com `totalMinutos === 0` o cartão do calendário continua: `.diario-numero` mostra `0 min` em `--text-tertiary`, o `Mapa` desenha os 30 dias vazios com cabeçalho de semana (célula vazia com borda, como a T11 pede), o `.diario-vazio` entra abaixo da `.mapa-rodape` em vez de substituir o cartão, e o botão de `Exportar` fica `disabled` com `title="Precisa de pelo menos uma sessão"`. O cartão do ranking segue com o texto de vazio da tarefa anterior.

**Pronto quando:** abrir a aba não pisca o texto de vazio; uma sessão que termina com a aba aberta aparece na célula de hoje sem trocar de aba; e numa instalação nova a grade de 30 células vazias aparece com o texto abaixo e o botão desabilitado.

## T121. O cartão exportado passa a ser a tela

**Onde:** `src/telas/Diario.tsx` (`Exportar`, `Retrato`, `desenharCartao`), `src/estilo/geometria.gerada.ts` (só leitura).

O PNG pinta todos os dias com `--accent-green` enquanto a tela pinta por faixa de carga. O rodapé é a palavra "Kontro" em texto, sem o anel que é a marca. "de jogo nos últimos 30 dias" não diz quais 30 dias, então a imagem envelhece sem data. Um título de jogo maior que 428px sai da imagem porque `fillText` nunca mede. E o top 3 não tem ícone, apesar de a T13 prometer e de `icone_do_jogo` já existir.

`Retrato` ganha `limiares`, passado de `Diario` por `Exportar`. Cada célula usa `faixa(d.menorCarga, limiares)` para escolher `--accent-green`, `--amber` ou `--red`, com o alfa de `nivel()` e `roundRect` de raio `Math.round(lado * 8 / 28)`, a proporção da célula de tela. O subtítulo vira `de jogo de ${diaEMes(dias[0])} a ${diaEMes(dias[29])}`. `encaixar(pincel, texto, largura)` corta o texto com "…" enquanto `measureText().width > largura`, com 400 para o título e 360 para o rodapé. Antes de desenhar, `Exportar` pede `icone_do_jogo` para os três primeiros de `famintos`; a resposta é `data:image/png;base64`, então `drawImage` não contamina o canvas e `toDataURL` continua funcionando. O ícone entra em 32px em x=700 e o nome desloca para x=744; sem ícone, `new Path2D(PAD_COM_STICKS_VAZADOS)` em `--text-tertiary` no mesmo quadrado, escalado por `32 / 412` a partir de `GLIFO_CAIXA`. A marca do rodapé segue `Marca.tsx` com `Path2D(PAD)`: 32px em x=72, círculo de fundo em `CORES.fundo`, trilha em `CORES.branco` a `APP.trilhoOpacidade`, arco de `APP.anelVarredura` graus começando em -90° com `lineWidth = APP.anelLargura` e `lineCap = "round"` em `CORES.verde` chapado (em 32px gradiente vira lama, DESIGN §2), o controle em `APP.padEscala` centrado em `APP.padCentroY` e os dois sticks em `CORES.fundo`; "Kontro" passa para x=116.

**Pronto quando:** o PNG salvo tem as mesmas cores de célula que a tela naquele momento, a marca com anel no rodapé, as datas do período no subtítulo, os ícones dos três jogos, e um jogo chamado "The Legend of Zelda: Tears of the Kingdom" termina com reticências dentro da imagem.

---

# Movimento 19 — Primeira abertura

A primeira abertura são seis telas em `src/telas/Passos.tsx` e `passos.css`, entregues na 2.12.0 e não tocadas por T1 a T22. O desenho ficou parado enquanto o resto do app andou: a folha é um cartão com cartões dentro, o traço de progresso ainda é verde de carga, o anel do passo 0 mostra um 72% que não é de ninguém, o atalho está escrito à mão, e o texto do passo Pronto descreve uma bandeja que não existe. Dois furos de fluxo pesam mais que tudo isso: fechar a janela no passo ao vivo deixa a pílula solta sobre a área de trabalho, e quem liga "Iniciar com o Windows" antes de clicar em Começar nunca mais vê o passo a passo. O que vira: os passos passam a ser feitos dos componentes reais com o estado real, a folha senta sobre o Mica como as outras páginas, o teclado leva do começo ao fim, e sair no meio não deixa nada pendurado.

## T122. Sair no meio do passo a passo não deixa a pílula solta nem esconde o tutorial no boot

**Onde:** `src-tauri/src/lib.rs` (`on_window_event`, ramo `CloseRequested`; `setup`, cálculo de `abrir_direto`).

Quem chega ao passo da pílula e fecha a janela no X fica com a pílula solta para sempre: contorno teal, cadeado, "--" em cinza se não há controle, por cima de tudo. O X cai em `CloseRequested`, que faz `prevent_close` e `hide()`; o React continua montado no passo 3, a limpeza do efeito que chama `soltar_a_pilula(false)` nunca roda, e `orquestra::sobreposicao` mostra a janela sem condição enquanto `solta` for verdadeiro. A correção é uma linha: no ramo `CloseRequested`, logo depois de conferir que a janela é a `PRINCIPAL` e antes tanto do `exit(0)` quanto do `hide()`, chamar `soltar_sobreposicao(janela.app_handle(), false)`. Ela já devolve cedo se a pílula não estava solta, grava `overlay_x/y` com o encaixe e emite `kontro://solta`, então o `Passos` volta sozinho para o texto "Está presa..." com o botão "Soltar de novo" quando a janela reabrir.

O segundo furo é no passo Pronto: a pessoa liga "Iniciar com o Windows", fecha sem clicar em Começar, e no boot seguinte o app sobe com `--minimizado`, o ícone fica atrás da setinha e a janela não aparece. Hoje `abrir_direto` é `pedido_explicito || voltou_de_atualizacao || (!subiu_com_o_sistema && (!start_minimized || !first_run_done))`: o `--minimizado` curto-circuita o `!first_run_done`. Vira `pedido_explicito || voltou_de_atualizacao || !cfg.first_run_done || (!subiu_com_o_sistema && !cfg.start_minimized)` — primeira abertura não terminada abre a janela mesmo vindo do início automático. Só Começar grava `FirstRunDone`, como o DESIGN §7 manda.

**Pronto quando:** no passo da pílula, fechar no X prende a pílula no lugar onde estava (com encaixe) e ela some da área de trabalho; reabrir pelo ícone mostra o passo 3 já com "Soltar de novo". Com `FirstRunDone: false` no `settings.json` e o app subindo com `--minimizado`, a janela principal abre no passo a passo.

## T123. A folha deixa de ser cartão dentro de cartão

**Onde:** `src/telas/passos.css` (`.folha`, `.recado`, `.previa`), `src/telas/principal.css` (`.linha`).

A folha é um cartão `--surface` com borda `--stroke`, e dentro dela o recado da bandeja, a prévia do cabo e as Linhas de "Avisar em" e "Iniciar com o Windows" são também `--surface` com borda `--stroke`: mesma cor sobre a mesma cor, só um fio de 1px separando. Nas Configurações as mesmas Linhas leem como cartões porque sentam sobre o `--fundo-janela` da `.pagina`; aqui viram mancha. No tema Dia, `--surface` é `#ffffff` — branco sobre branco. O DESIGN §7 descreve o passo a passo como "coluna centrada de 520", não como cartão.

Em `.folha` saem `border`, `border-radius` e `background`; o `padding` vira `0 var(--space-3)` (a folga lateral fica para a barra de rolagem não encostar no texto); `max-width: 520px`, `overflow-y: auto` e o resto continuam. Recado, prévia e Linhas passam a sentar sobre o Mica como em toda outra página, com o `--surface` e o `--stroke` que já têm.

**Pronto quando:** nos seis temas, o recado da bandeja e as duas Linhas do passo 4 têm fundo visivelmente distinto do que está atrás, igual às Linhas de Configurações.

## T124. A dica da setinha vai para o passo Pronto, e Pular leva até ele

**Onde:** `src/telas/Passos.tsx` (passos 0, 1 e 5, botão Pular), `src/telas/passos.css`, `src/componentes/Marca.tsx`.

O recado sobre o "^" do Windows 11 é um quadro cinza embaixo de "O ícone é o dado", no passo em que a pessoa ainda não vai procurar o ícone. Quem clica em Pular na primeira tela — o caso mais comum — nunca lê, fecha a janela, o app vai para a bandeja e o ícone não está visível. É o "instalei e não apareceu nada".

O recado sai do passo 1 e entra no passo Pronto, acima da Linha "Iniciar com o Windows". O palco do passo Pronto deixa de ser o anel a 100% e vira uma mini barra de tarefas em CSS: uma faixa de largura total e 40px de altura, `--surface-alt`, raio 8, com o `^` num `<kbd className="tecla">` e a `Marca tamanho={16}` real à direita dele, como ficam na barra de verdade. A Marca entra uma vez, deslizando 16px de trás do `^` para o lugar em `var(--motion-base) var(--curva)` — não repete. `Pular` passa a fazer `setPasso(TOTAL - 1)` em vez de `terminar`: quem pula ainda vê onde achar o ícone, e só Começar grava `FirstRunDone`. O passo 0 ganha uma frase curta no fim do parágrafo: "Ele fica na bandeja — o último passo mostra onde."

**Pronto quando:** clicar em Pular na primeira tela leva ao passo Pronto com a mini barra de tarefas e o recado; só Começar encerra.

## T125. Os anéis dos passos mostram o controle de verdade, com as marcas de limiar

**Onde:** `src/telas/Passos.tsx` (passos 0, 3 e 4), `src/componentes/Anel.tsx` (`marcas`, já existe).

Com o controle ligado, o primeiro anel que a pessoa vê marca 72% verde enquanto o ícone da bandeja mostra outro número — dois valores para a mesma bateria na primeira impressão. O passo 4 mostra dois anéis pequenos, um a 20% âmbar e outro a 10% vermelho, mas não passa `marcas` ao `Anel`, então a pessoa não vê os traços que a T3 entregou — justamente o que "os mesmos números mandam na cor do anel" descreve. E no passo ao vivo, sem controle, a pílula aparece cinza com "--" e nada explica.

`Passos` passa a chamar `useEstado()`; os limiares saem do `cfg` que já tem (`{ critico: cfg.CriticalThreshold, aviso: cfg.WarnThreshold }`). `aoVivo` é `estado && estado.via !== "Desligado" && !estado.leituraAntiga && estado.preenchimento !== null`. Passo 0: `valor={aoVivo ? estado.preenchimento : 72}`, `cor={aoVivo ? corDoAnel(estado, limiares) : "var(--accent-green)"}`, e uma `.legenda` abaixo com `{estado.nome} · {estado.textoDaCarga}` ao vivo ou "exemplo" quando não. Passo 4: os dois anéis de 54 viram um só de 120 no palco, com o mesmo `valor` e `cor` do passo 0 e `marcas={{ aviso: cfg.WarnThreshold, critico: cfg.CriticalThreshold }}`; as legendas "avisa" e "insiste" saem, os títulos das Linhas já dizem isso. Ao ciclar "Avisar em" o traço âmbar anda pelo anel, e se o valor real cruzar o limiar o arco muda de cor na hora. Passo 3 com `solta` e `estado?.via === "Desligado"`: um `<p>` a mais, "Sem controle ligado ela fica cinza com um traço; ligue um e a carga aparece nela na hora."

**Pronto quando:** com o controle ligado, o anel do passo 0 mostra o mesmo número e cor do ícone da bandeja; clicar em "Avisar em" move o traço âmbar no mesmo instante; sem controle, o passo ao vivo explica o traço cinza.

## T126. As amostras "desligado" e "no cabo" copiam o que o app mostra de verdade

**Onde:** `src/telas/Passos.tsx` (passos 1 e 2), `src/telas/passos.css` (`.amostra.apagada`, `.risco`, `.previa`, `.dizeres`), `src/formato.ts` (`quandoLeu`, já existe).

O passo 1 diz "ele vira um controle riscado" e mostra um `Anel valor={null}` — que desenha a trilha — com um risco fino por cima. O ícone off da bandeja (DESIGN §3) não tem anel: é o controle a 45% com uma barra diagonal de 46/512 de largura; o risco desenhado tem 3.4/54, dois terços disso. E `.amostra.apagada { opacity: 0.5 }` apaga a legenda "desligado" junto: `--text-tertiary` em 11.5px já está perto de 4:1, pela metade fica em torno de 2:1. O passo 2 mostra uma pílula falsa: "no cabo" em Segoe 15px 600 quando a pílula real escreve em Consolas 16px 600, padding 8/18/8/10 fora da escala, e um carimbo "lido ontem às 23:53" que nunca aconteceu — no passo que existe para dizer que o app não inventa número.

Passo 1: o `Anel` da amostra apagada sai; fica `<span className="riscado">` com só o `Glifo tamanho={22} cor="var(--gray)"` e o `svg.risco` com `strokeWidth="4.9"` (46/512 × 54). Em `passos.css`, `.amostra.apagada .riscado { opacity: 0.45 }` e nada na legenda. Passo 2: o carimbo vira `estado ? quandoLeu(estado) : "sem leitura ainda"` — é a última leitura desta máquina ou a declaração de que não há. O `.valor` passa a "No cabo" em 16px `--text-primary` (é a composição do flyout, DESIGN §7: "No cabo" em Subtitle e a leitura em Caption), `.carimbo` em 12px mono, `.previa` com `padding: var(--space-2) var(--space-4) var(--space-2) var(--space-2)`. O anel `girando` fica: é o que a pílula, o painel e o Resumo mostram hoje no cabo (`girando = via == Cabo && preenchimento.is_none()` em `modelo.rs`), e mudar isso é decisão da pílula, não do passo.

**Pronto quando:** a amostra "desligado" não tem trilha e o risco tem a proporção do ícone da bandeja, com a legenda na mesma cor das outras três; o carimbo do passo 2 é a hora real da última leitura ou "sem leitura ainda".

## T127. passos.css entra na escala do sistema: palco fixo, margens zeradas, texto em 12/14/16 e realce em --realce

**Onde:** `src/telas/passos.css` (`.palco`, `h1`, `p`, `.folha`, `.legenda`, `.recado`, `.ponto.agora`), `src/telas/principal.css` (`.mini-tela.solta .mini-pilula`), `DESIGN.md` §7.

O título pula 24px ao ir do passo 0 para o 1 e desce de volta no 5: os anéis de 120 esticam o `.palco { min-height: 96px }`, os outros passos cabem nos 96. Entre palco e título há 27px, entre título e corpo 41px, entre corpo e Linha 26px — o `0.67em` do `h1` e o `1em` do `p` somam ao `gap` de 12px, porque margem não colapsa em flex. Legendas em 11.5px, carimbo em 11px, recado em 13px: nenhum degrau do DESIGN §4. E o traço de progresso e a mini-pílula solta são `--accent-green`: no Ardósia, aba ativa, foco, chave e Começar são azuis e só o progresso é verde — a única coisa verde que não fala de bateria. O DESIGN §7 ainda escreve AccentGreen, mas a T16 separou `--realce` como cor de interface.

`.palco { height: 136px; min-height: 0 }` — o valor do DESIGN §7, e a MiniTela do passo 3 (84px de altura) e o anel de 120 cabem. `.passos h1, .passos p { margin: 0 }`; o `gap` da folha segue `--space-3`. `.legenda` para 12px, `.recado` para 14px com `line-height: 1.5`. `.ponto.agora` e `.mini-tela.solta .mini-pilula` para `var(--realce)`, e a frase do DESIGN §7 passa a "um traço de 20px em --realce". Em 720x520, o passo 4 (palco 136 + título + três linhas de corpo + duas Linhas) fica no limite da folha; se ainda rolar, o padding vertical de `.passos` desce de `--space-4` para `--space-3`, que são os 8px que faltam.

**Pronto quando:** o topo do título fica na mesma coordenada y nos seis passos; nenhum `font-size` em `passos.css` fora de 12/14/16/22; em 720x520 nenhum passo mostra barra de rolagem; no Ardósia o traço de progresso e a mini-pílula solta são azuis como a aba ativa, e no Noite continuam teal.

## T128. A troca de passo desliza e os anéis preenchem na entrada

**Onde:** `src/telas/Passos.tsx` (`.folha`), `src/telas/passos.css`, `src/componentes/Anel.tsx`.

Clicar em Avançar troca o conteúdo seco, e o anel de 72% já nasce cheio: `.folha` não tem `key`, então o React só troca filhos, e `Anel` inicia `suave` em `valor ?? 0`, ou seja, no alvo — só anima em mudanças posteriores. O DESIGN §7 diz "o anel de verdade, animando", o §6 fixa 180ms ease-out, e a pílula já entra com `kontro-pilula-entra` em 240ms; o passo a passo não tem entrada nenhuma.

`<div className="folha" key={passo}>` e, em `passos.css`, `.folha { animation: kontro-passo-entra var(--motion-base) var(--curva) }` com `@keyframes kontro-passo-entra { from { opacity: 0; transform: translateX(12px) } }` — uma vez por troca, sem loop. Em `Anel.tsx`, prop `desde?: number` usada em `useState(desde ?? valor ?? 0)`; os anéis dos passos recebem `desde={0}` e preenchem de 0 ao valor nos 180ms que o componente já tem. Como o `Anel` anima por `requestAnimationFrame` e o `prefers-reduced-motion` do `base.css` só alcança CSS, com `matchMedia("(prefers-reduced-motion: reduce)").matches` o `useState` parte do próprio `valor`.

**Pronto quando:** cada passo entra deslizando 12px em 180ms; os anéis dos passos 0 e 4 preenchem de 0 ao valor na entrada; com "reduzir movimento" ligado no Windows, tudo aparece direto.

## T129. A mini tela acompanha o arraste e mostra o encaixe antes de prender

**Onde:** `src-tauri/src/lib.rs` (`on_window_event`), `src-tauri/src/janelas.rs` (`Pouso`, `onde_a_sobreposicao_parou`), `src/telas/Passos.tsx`, `src/componentes/Controles.tsx` (`MiniTela`).

O texto do passo ao vivo diz "Perto de um canto ou do meio ela encaixa sozinha" e mostra a mini tela com a pílula. A pessoa arrasta a pílula de verdade e o desenho não muda; solta perto do canto e nada encaixa. Só ao clicar no cadeado a pílula salta para o canto e o desenho atualiza, de uma vez, porque `cfg.OverlayX/Y` só são recalculados em `soltar_sobreposicao(false)` e não existe evento de posição enquanto a janela se move. A promessa e o feedback não batem no único momento em que a pessoa está aprendendo.

`on_window_event` vira um `match`: além do `CloseRequested` da principal, trata `WindowEvent::Moved(_)` da janela `SOBREPOSICAO`; se `sobreposicao_solta` for verdadeiro, chama `janelas::onde_a_sobreposicao_parou` — que já passa x e y por `encaixar` com a folga de 28px — e emite `kontro://pouso` com o `Pouso` (`#[derive(Serialize)]` nele). Em `Passos.tsx`, `listen<{ x: number; y: number }>("kontro://pouso")` guarda `pouso` em estado, zerado quando `solta` volta a falso, e a `MiniTela` recebe `x={pouso?.x ?? cfg.OverlayX}` e `y={pouso?.y ?? cfg.OverlayY}`. A transição de 180ms que `.mini-pilula` já tem em `principal.css` faz a mini-pílula deslizar junto e saltar para 0, 0.5 ou 1 no instante em que a pílula entra na zona de encaixe.

**Pronto quando:** arrastando a pílula, a mini-pílula se move em tempo real e salta para o canto ou o meio quando a pílula entra na zona de encaixe, antes de clicar no cadeado.

---

# Ordem

    T1 -> T2 -> T3 -> T4 -> T5        material e luz, na ordem de retorno
    T6 -> T7 -> T8 -> T9 -> T10       a cadeia do jogo, cada uma depende da anterior
    T11 -> T12 -> T13                 só faz sentido com dado de jogo gravado
    T14                               depende só da T1
    T15 -> ... -> T19 -> T20          independentes de todas
    T21                               conserta a T7 e a T8
    T22                               independente de todas
    T23 -> T27                        a pílula por cima da tela cheia, na ordem em que estão
    T28 -> T41                        desempenho e fluidez
    T42 -> T51                        janelas, bandeja e atalhos
    T52 -> T59                        sistema de design e temas
    T60 -> T64                        textos e estados
    T65 -> T73                        resumo
    T74 -> T78                        pílula e aviso
    T79 -> T88                        instalação, atualização e avisos do sistema
    T89 -> T95                        configurações
    T96 -> T102                       acessibilidade e teclado
    T103 -> T109                      sessões e controles
    T110 -> T114                      painel e moldura da janela
    T115 -> T121                      diário
    T122 -> T129                      primeira abertura

T1 e T6 são independentes: dá para tocar o visual e a detecção em paralelo. Tudo de T11 para
frente precisa de duas semanas de dados gravados com jogo para ser visto de verdade — vale
começar a gravar cedo, com T6 a T8, mesmo que a tela venha depois.

Na Parte 2 a ordem dentro de cada movimento é a do texto, do maior retorno para o menor, e
os movimentos estão na ordem em que valem mais. Três dependências atravessam movimentos: as
tarefas que mexem no ciclo de 2 s e no `palco` do aviso vêm depois de «Medir» e «Posição em
pixel físico» (T23 e T27); a prévia da pílula em Configurações depende de as
páginas ficarem montadas ao trocar de aba; e o aviso de "continuo na bandeja" usa a
notificação nativa, que o toast com o anel substitui — quem vier depois usa o novo.

# O que este plano recusa

Da Parte 1:

- **Widget na Xbox Game Bar.** Exige empacotamento UWP e um projeto separado. O ganho não
  paga a segunda cadeia de build.
- **Detectar jogo por lista de títulos conhecidos.** Vira manutenção eterna e erra em jogo
  indie. Tela cheia mais nome do exe é honesto e não envelhece.
- **Estimar quanto falta para carregar.** Continua recusado pelo mesmo motivo da tarefa 24
  do `TAREFAS.md`: no cabo não existe percentual, e a conta seria chute com cara de conta.

Da Parte 2, sobre a tela cheia (o detalhe está em `TELA-CHEIA.md`):

- **UIAccess e bandas de janela, por qualquer via.** Pela via oficial: assinatura com cadeia confiável que hoje não existe e que, pelo Artifact Signing, não está disponível para pessoa física no Brasil; instalação em Program Files com admin; UAC em cada atualização; migração de quem já instalou; um modo de falha em que o app não abre; e uma função que a Microsoft diz para não usar "by applications that just want to appear above other applications". Pelos atalhos (token do `winlogon`, DLL no `explorer.exe`): admin, função privada que muda sem aviso, e cara de malware. O app roda sem admin de propósito.
- **Injeção com hook de `Present`.** É o que resolve a tela cheia exclusiva legada, e é o que anti-cheat chama de cheat. Sem contrato com Epic, BattlEye e Riot, quem leva o ban é o usuário. Nem como opção "por sua conta".
- **`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`.** Overlay topmost, transparente ao clique e invisível à captura é a forma do ESP "streamproof" que anti-cheat existe para pegar (§2.6). Nem como chave desligada por padrão.
- **Esconder a pílula em tela cheia exclusiva.** Não compra nada onde o DWM não compõe, e em outra GPU ou driver pode esconder uma pílula que estaria visível, com base num sinal sem documentação (§1.2 B). Registrar, sim; agir, não.
- **Mexer nas propriedades de compatibilidade dos jogos.** Dava para escrever `DISABLEDXMAXIMIZEDWINDOWEDMODE` no registro do usuário e "consertar" o jogo. O app mede bateria; não reconfigura o jogo dos outros. Dizer é o limite.
- **Lista de jogos que usam tela cheia exclusiva.** Manutenção eterna, o mesmo motivo pelo qual o plano já recusou detectar jogo por título.
- **Inferir tela cheia exclusiva por heurística** (`GetWindowRect` igual ao monitor mais "parece jogo"). O Windows tem um sinal explícito; usar outro seria chute com cara de conta.
- **Brigar por Z sem limite, ou contra quem não cobre.** Relógio sem teste de oclusão, reafirmação sem desistência, ou teste que conta janela em camadas como cobertura, é o "walls and ladders" que Raymond Chen manda parar — e, no último caso, uma frase inventada em Configurações.
- **Título e caminho de janela de terceiro no diagnóstico.** O arquivo é para ser enviado; o que vai nele é o nome batizado do programa, e só.
