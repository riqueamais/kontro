# Tarefas

Levantamento feito sobre o commit `05e8ed3`. O que está resolvido ficou registrado com a
causa, porque a causa é o que evita o problema voltar. O que sobrou está no fim.

---

## Resolvido

### 1. O histórico nunca era gravado em disco

**Sintoma:** reiniciava o PC, abria o app, e ele mostrava "a última leitura" com um valor
que não era o último visto.

**Causa:** `monitor.salvar()`, no fim do laço de leitura, era código inalcançável.
`tauri::Builder::run()` nunca devolve o controle — por baixo, `tao::EventLoop::run` tem
assinatura `-> !` e o processo sai por `process::exit`. O `Pedido::Encerrar` nunca era
enviado, o laço nunca saía do `loop`, e a única gravação do histórico nunca acontecia.
Na prática `History::salvar()` só rodava de dentro de `esquecer()`.

Medido nesta máquina: `history.json` tinha sido escrito pela última vez sete dias antes,
com o app rodando desde as 9h daquele dia.

**Correção:** a montagem passou a ser em duas etapas (`build` + `run`), o evento de saída
dispara a gravação com um aperto de mão para a thread do monitor, e existe uma gravação
periódica a cada 30 s enquanto houver amostra nova — queda de energia não manda evento.
Os dois caminhos foram verificados com o app rodando.

### 2. Valor sem data do Windows derrubava leitura com hora

Segunda causa do número errado. `aceitar_guardada` só recusava um valor sem carimbo
quando a nossa leitura tinha menos de cinco minutos — e depois de um boot a leitura
semeada do arquivo tem sempre mais que isso. O valor velho entrava, era carimbado com a
hora de agora e ainda ia para o histórico como se fosse medida nova.

Agora: com data, entra (o piso da consulta já garantiu que é mais nova). Sem data, só
preenche vazio ou substitui outro valor igualmente sem data — e fica marcado como
`incerto`, que impede tanto o "ao vivo" na tela quanto a entrada no histórico.

### 3. A interface escondia a data

`Resumo` e `Painel` formatavam só `HH:MM`, então uma leitura de três dias atrás aparecia
como "lido às 21:34" e passava por recente — justamente a dúvida que o horário existe
para levantar. Agora sai "ontem às 21:34" ou "em 24/08, às 21:34", e a formatação vive
num lugar só.

### 4. Desconectar o controle dizia "no cabo"

**Causa:** a lista de presentes era recalculada a cada 30 s. Ao desligar o controle o
GATT respondia desconectado no mesmo ciclo, mas ele seguia constando como presente por
até meio minuto — e `no_cabo()` caía no ramo `ha_controle_presente`, que responde cabo.
Resultado: até 30 s de anel girando e "no cabo", mais um aviso falso de "agora no cabo"
antes do de desconectado.

**Correção:** a varredura passou a ser disparada por evento. `device/vigia.rs` observa as
interfaces HID de gamepad/joystick/multi-eixo e a do XUSB, e avisa no instante em que o
Windows publica ou retira uma. O relógio de 30 s virou rede de segurança de 15 s. Medido
no diagnóstico: 4 observadores de pé, eventos chegando.

### 5. `no_cabo` errava em controle que o XInput não enxerga

O diagnóstico desta máquina mostra o porquê: o Xbox por Bluetooth ocupa o slot 0 do
XInput mas se declara como `tipo=0`. Então `alguem_no_cabo` e `alguem_na_bateria` são
ambos falsos, e a resposta antiga para "não sei" era cabo. Um DualSense nem aparece.

A prova já existia e era jogada fora: a descoberta extrai o endereço Bluetooth do id da
interface **atual**, e endereço não-zero significa que o controle chegou por Bluetooth
agora. A lista de presentes passou a guardar essa evidência em vez de só a chave, e a
decisão da via virou função pura, coberta por teste.

### 6. Presença comparada com chave de outra origem

Um controle pareado por Bluetooth e depois plugado no cabo é descoberto sem endereço, com
chave `hid:...`, enquanto o registro salvo guarda a do endereço de propósito. Comparando
só as chaves, ele aparecia desconectado com o cabo na mão — ou duplicava na lista. O
casamento agora repete o critério de `Conhecidos::fundir`: chave **ou** container.

### 7. O botão "Atualizar" não fazia nada

`Ok(Pedido::LerAgora) => {}` — o pedido chegava ao ciclo e era descartado. Os dois botões
e o item de bandeja não faziam absolutamente nada. Agora refaz a varredura, zera os
relógios de releitura e relê o GATT na hora. O item da bandeja virou "Ler a bateria
agora", que era o que ele sempre quis dizer.

### 8. Limiares sem controle na tela

Eram usados pelo código e prometidos pelo README, mas não existia nenhum controle para
mexer neles — e a própria linha dizia "ao cruzar os limiares abaixo", com nada abaixo.
Agora são dois botões, e o Rust confere que o crítico cabe abaixo do de aviso.

### 9. Aviso disparado por leitura velha

Nem `limiares` nem o `critico` da sobreposição olhavam `estado.stale`. Uma última carga
de 8% da semana passada anunciava "carga crítica" e puxava a pílula para a tela no
instante em que o controle reaparecesse — inclusive no cabo, carregando.

### 10. "Segue o jogo" não seguia nada

A busca chamava `current_monitor()`, descartava o resultado com um
`and_then(|_| monitores.first())` e devolvia sempre o primeiro monitor. Agora a pílula
acompanha a janela em foco, e não o cursor — em jogo de tela cheia o cursor some.

### 11. Seletor de monitor preso em dois

O ciclo tinha o número dois escrito dentro dele. Novo comando `quantidade_de_telas`, e a
tela cicla sobre as telas que existem.

### 12. O painel abria no monitor errado

Ancorava no monitor primário. Agora ancora no monitor do cursor — o painel nasce de um
clique no ícone, então o cursor está na tela certa por definição. De quebra, a posição
passou a usar a área de trabalho do sistema, que já desconta a barra de tarefas; antes
havia um `-56` fixo que só acertava na configuração mais comum.

### 13. Estado de aviso era global

`avisados` e `ultimo_percentual` eram valores únicos: o "já avisei 20%" de um controle
calava o aviso do outro assim que o principal trocasse. Agora são por controle. A
detecção de troca de via também passou a carregar a chave junto, senão um segundo
controle entrando parecia o primeiro mudando de via.

### 14. Dica da bandeja envelhecia

A assinatura que decide redesenhar é `modo|preenchimento|tamanho`, e `charging` não entra
nela — mas muda o texto. A dica continuava dizendo "no cabo" depois de o controle começar
a carregar. Agora o texto é refeito sempre; só o desenho depende da assinatura.

### 15. A saúde da bateria não se atualizava

Só refazia a conta quando o controle trocava. Agora acompanha a hora da última leitura.

### 16. Thread vazada na leitura HID por entrada

Desistir da espera não desistia da operação: a thread ficava presa no `join` para sempre,
segurando o dispositivo. Num controle que declara o relatório de entrada e nunca responde,
era uma thread por releitura. Agora a operação é cancelada.

### 17. Um `settings.json` ilegível apagava tudo em silêncio

Descoberto durante o teste: um arquivo salvo com BOM — o que qualquer editor ou script do
PowerShell faz — não passa pelo `serde_json`. O app caía nos padrões, via `first_run_done`
falso e regravava por cima, apagando toda a configuração sem nada na tela. Agora a leitura
tolera o BOM, e um arquivo que mesmo assim não for entendido é guardado como
`settings.json.invalido` em vez de sobrescrito.

### 18. Limpeza

- `tauri-plugin-shell` saiu: estava no Cargo, no package.json, registrado e com
  `shell:allow-open` liberado, e nada no app abria URL nem processo.
- `overlay_tip_shown` saiu — declarada, serializada e lida por ninguém.
- `overlay_scale` e `overlay_opacity` deixaram de ser mortas: viraram tamanho e
  transparência da pílula, com controle na tela e aviso ao vivo para a janela dela.
- `Monitor.ativo` saiu — escrito em três lugares, lido só para soltar um vínculo que o
  ciclo seguinte já soltaria.
- `escolher_principal` rodava duas vezes por ciclo sobre a mesma lista.
- `let _ = FOLGA_LATERAL_DO_AVISO;` existia só para calar o compilador.
- `montar_um` pedia a hora de novo em vez de receber a do ciclo.
- `History::carregar` podava trinta dias e nunca gravava a poda; e série vazia não vai
  mais para o arquivo.
- A varredura deixou de perguntar ao Gaming.Input quantos controles existem, e só
  pergunta ao XInput quando há controle presente sem endereço Bluetooth.
- A lista de pareados — que abre cada dispositivo Bluetooth só para ler o nome — passou a
  ser guardada por um minuto. É o que torna barato varrer com frequência.
- README: dizia que o instalador sai pelo Velopack (é NSIS + updater do Tauri) e que os
  limiares eram ajustáveis pela bandeja.

### 19. Testes

`monitor.rs` concentra todas as regras e não tinha um teste sequer. São 12 agora, um por
caso real: desligar não é ligar no cabo, quem chegou por Bluetooth nunca está no cabo, o
controle plugado continua sendo o mesmo, número sem data não derruba leitura com hora, a
fonte sem data continua acompanhada, o ícone mostra quem está pior.

O diagnóstico ganhou duas seções: se o vigia está de pé, e o que o monitor conclui depois
de rodar o ciclo real pelo tempo de confirmação. Quando alguém disser "mostra a carga
errada", é a diferença entre uma fonte que mente e uma regra que escolheu mal.

---

## Resolvido na 2.5.0

### 20. A autonomia contava noite desligada como consumo

`consumo_por_hora` voltava por todas as amostras não crescentes sem olhar o tempo entre
elas, enquanto `descargas` — que alimenta a saúde — já quebrava em buracos de meia hora.
A regra estava escrita e aplicada em só um dos dois lugares. Ficou dormente enquanto o
histórico não chegava ao disco; com a série atravessando sessões, um controle que caiu de
80% para 60% ao longo de um dia desligado renderia "~72 h de jogo". A conta passa a sair
de `descargas`, e nos primeiros minutos de sessão vale a média da semana em vez do
silêncio de antes.

### 21. A pílula mostra os outros controles ligados

Ela só mostrava o de menor carga — critério certo para a bandeja, onde cabe um — e em jogo
local escondia o que se quer saber com dois na mesa.

### 22. Os comentários saíram do resto do código

A limpeza tinha parado no sistema de ícones. Saíram de Rust, TypeScript e CSS, com um
analisador que acompanha o estado do texto para não confundir URL em string, barra de
divisão e tempo de vida do Rust com comentário.

---

## Resolvido nas 2.6.0 a 2.9.0

- **A troca de bateria** passou a ser percebida. A série de um controle não é a série de
  uma bateria: com 87% de carga nova o app projetava tempo de jogo na taxa da bateria que
  tinha morrido, e a saúde compararia duas baterias diferentes. Uma subida de quinze pontos
  em menos de cinco minutos não é carga nenhuma — é troca.
- **O rádio parou de ser acordado à toa.** A cada dois segundos, por controle guardado,
  o ciclo abria uma conexão Bluetooth só para conferir se estava ligado — inclusive para
  controle desligado. A prova já vinha na varredura: interface presente com endereço é
  conexão de agora. Junto apareceu um ponto cego, com dois controles de Bluetooth o
  segundo ficava sem fonte de carga alguma.
- **A enumeração do PnP** deixou de se repetir dentro da mesma leitura.
- **As sessões de uso** entraram no resumo e no diagnóstico.
- **A autonomia** deixou de projetar horas a partir de um ponto de queda.
- **A verificação de atualização** virou uma só, a do instalador, que é a que confere
  assinatura. E a nota da release passou a viajar dentro do manifesto, então o app mostra
  o que a versão nova traz em vez de mandar procurar a página.
- **O diagnóstico** ganhou botão em Configurações, sem levantar um segundo monitor.
- **As permissões** deixaram de ser as mesmas para as quatro janelas.
- **`Saude` e `Sessao`** passaram a sair em camelCase como o resto.
- **A pílula com vários controles** foi conferida na tela: divisória, tamanhos e
  espaçamento corretos, sem corte.

---

## Resolvido na 2.10.0

### O gráfico mentia por três motivos

Ele ligava todos os pontos numa linha só, atravessando os dias em que o controle ficou
desligado como se fossem medição contínua. Auto-escalava o eixo vertical para o intervalo
dos dados, então uma variação de dois pontos preenchia a altura toda e parecia uma queda
enorme. E trocava de cor conforme a carga de agora, pintando a história inteira de vermelho
porque o controle está fraco hoje -- cor seguindo o valor atual, não a série.

Agora: eixo de tempo real com a janela escolhida (7 ou 30 dias), eixo vertical fixo de 0 a
100, linha quebrada onde a série quebra, cor estável, e a troca de bateria marcada.

### Dois estilos vazavam entre janelas

Os CSS das quatro janelas viram um arquivo só, aplicado a todas. `aviso.css` declarava
`.cartao { display: flex }` sem escopo, e isso mandava no cartão da janela principal --
era por isso que o gráfico, a saúde e as sessões apareciam lado a lado espremidos. E
`.trilho` é ao mesmo tempo a barra lateral do app e a barrinha de progresso da saúde: a
barrinha herdava `padding` e `border-right` da barra lateral e virava uma caixa vazia.

Ambos escopados. Vale conferir isso ao criar qualquer classe nova com nome genérico.

### Clicar numa sessão abre ela no gráfico

O uso é em rajadas, então numa janela de sete dias a maior parte fica vazia e a descarga
vira uma linha quase vertical. A lista responde "quando"; o gráfico agora responde "como
drenou", com eixo em minutos e a taxa daquela sessão embaixo.

### Quanto dura uma carga cheia

O número que responde "essa pilha presta?" a partir de uma sessão medida, em vez das duas
semanas que a saúde exige.

---

## Resolvido na 2.11.0

Levantamento feito sobre o commit `26949f8`, cobrindo funcionalidade, código repetido e
padrão de nomes de uma vez.

### 27. O instalador saía com lixo, e o lixo dobrava a cada release

**Sintoma:** o `copyright` e a `longDescription` do `tauri.conf.json` eram sequências
ilegíveis. Os dois campos vão para o recurso de versão do `.exe` e para o instalador
NSIS — é o que aparece nas propriedades do arquivo.

**Causa:** o passo que carimba a versão fazia round-trip por
`ConvertFrom-Json | ConvertTo-Json`, reescrevendo o arquivo inteiro. Leitura sem encoding
declarado decodifica UTF-8 como ANSI, então as duas únicas linhas acentuadas do arquivo
voltavam com uma camada a mais de codificação. Uma camada por release, medida commit a
commit: 39, 41, 46, 57, 81, 256, 525, 1123, 2454 caracteres.

**Correção:** textos restaurados; o carimbo troca só a linha da versão, com
`System.Text.UTF8Encoding` explícito nas duas pontas; e a conferência recusa a release se
o `Ã` reaparecer no arquivo.

### 28. A cor do anel ignorava os limiares configurados

O `DESIGN.md` já mandava, em letras maiúsculas: "a cor deve seguir o valor configurado,
não os números acima cravados". Mas 30 e 60 estavam cravados em dois lugares — no
`bandeja.rs` e no `estado.ts`. Quem punha o aviso em 40% recebia a notificação de carga
baixa com o anel ainda verde.

Agora saem de `Settings::limiares()` e de `useLimiares()`. Os limiares entraram também na
assinatura que decide redesenhar o ícone: sem isso, mudar o limiar não repintava a
bandeja até a carga mudar.

### 29. O principal.css continuava global

A 2.10.0 registrou o vazamento de `.cartao` e escopou o `aviso.css` — mas só do lado de
quem sofria. `.cartao`, `.linha`, `.ciclo`, `.aba` e `.corpo` seguiam soltos no
`principal.css`, e o `lista.css` ainda soltava `.item`, `.confirmar` e `.ciclo.perigo`.

A regra agora vale para as sete folhas e é mecânica: **todo seletor começa pela raiz do
próprio arquivo**. A janela principal se escopa por `body[data-janela="principal"]`, que
o `App.tsx` já carimbava.

### 30. Carregar era lido como trocar a bateria — e era isso que travava a saúde

Resposta para a tarefa 26. A série guardava só hora e porcentagem; a via e o estado de
carga, que o app mede a cada ciclo e mostra na tela, eram descartados antes do disco.

Nos dados desta máquina: 12% às 23:10, 51% às 00:19 e 87% às 00:20 — o controle no cabo.
`ultima_troca` respondia "pilha nova", `desde_a_troca` cortava 8 dos 10 dias de série, e a
Saúde dizia "1 dia de histórico, faltam 13" com o app rodando havia dez dias.

Agora a amostra carrega a via. Subida com o cabo na mão não é troca. E subida encadeada
também não: uma pilha nova é uma descontinuidade única, enquanto uma carga sobe ponto a
ponto — critério que vale também para a série antiga, que não tem via gravada.

**Medido depois:** série útil de 1,9 para 9,9 dias, descargas aceitas de 1 para 4,
`Saude.dias` de 1 para 9. O veredito ainda não sai porque faltam as duas semanas, e não
mais porque a conta joga os dados fora.

### 31. O corte de 30 minutos não correspondia ao ritmo das leituras

`SALTO_QUE_QUEBRA_O_TRECHO_MS` significava "o controle estava desligado". Medindo os
intervalos reais entre amostras: mediana 11 min, p75 33 min, p90 94 min. Com o GATT
avisando só na mudança, meia hora calado durante o jogo é uso, não ausência — e a maior
descarga da série inteira era picotada em quatro pedaços curtos demais para contar.

O desligamento agora é gravado como ponto na série, e a quebra sai do fato. O relógio
virou rede de segurança de 6 h, para o caso de o app morrer sem gravar o marcador, e a
regra dos 30 min continua valendo só onde não há via gravada — ou seja, na série antiga.

### 32. A marca tinha três cópias

O README dizia que todos os ícones saem de `geometria.rs` e que nenhum é editado à mão.
Valia para os PNG, os ICO e os SVG; não valia para o app. O path do controle existia em
três cópias — `geometria.rs`, `Glifo.tsx` e, inline, `Marca.tsx` — e os raios, escalas,
opacidades e cores em duas.

`--gerar` agora escreve também `src/estilo/geometria.gerada.ts`. Conferido: depois da
troca os PNG e os ICO saem byte a byte iguais; só os SVG mudaram, e só porque `0.10`
passou a sair como `0.1`.

### 33. O `interface Config` do TypeScript

Tarefa 25. Agora sai de `CAMPOS_DA_CONFIG`, no mesmo arquivo do `Settings`, e vira
`src/config.gerada.ts`. Um teste compara a lista com as chaves que o serde realmente
grava, então campo novo em `Settings` sem entrada na lista quebra o teste em vez de
passar despercebido. Adicionar uma *variante* de enum ainda não é pego — só renomear uma.

### 34. Padrão de nomes

Os arquivos eram metade em inglês enquanto o miolo deles era português. `autostart`,
`history`, `known`, `model`, `paths`, `settings`, `tray`, `device` e `discovery` viraram
`inicio_automatico`, `historico`, `conhecidos`, `modelo`, `caminhos`, `configuracoes`,
`bandeja`, `dispositivo` e `descoberta`.

`BatteryState` era o pior ponto: `mode`, `percent`, `read_at`, `charging`, `stale`,
`device_name` e `known_count` em inglês ao lado de `precisao`, `nivel` e
`texto_da_carga` — e isso vazava inteiro para o front, porque a serialização é
automática. Virou `EstadoDoControle`, e `LinkMode` virou
`Via { Desligado, Bluetooth, Cabo, SemFio }`.

`montar` recebia doze argumentos posicionais, dois deles `bool` vizinhos: trocar
`charging` e `stale` de lugar compilava e passava nos testes. Agora recebe um `Bruto` com
`Default`, e cada chamada nomeia o que preenche.

`Settings` e `ControleSalvo` ficam em inglês de propósito: eles não são tipos do app, são
o formato de `settings.json` e `controllers.json`. Renomear os campos renomearia as chaves
dos arquivos que já estão na máquina de quem usa.

### 35. Limpeza

- `detalhe(estado)` existia idêntica em `Painel` e `Resumo`; quatro arquivos formatavam
  data em pt-BR por conta própria. Foram para `src/formato.ts`. Junto saiu uma
  inconsistência: a lista de sessões escrevia "2 h 15" e a saúde, "2 h 15 min".
- O leitor de propriedade do WinRT tinha três cópias — duas dentro do `descoberta.rs` e a
  original no `pnp.rs`, que já fazia exatamente aquilo.
- `igual_a` não comparava `autonomia`: a estimativa podia mudar sem a tela receber o
  evento.
- `impl Orquestrador` estava aberto duas vezes no mesmo arquivo.
- `posicionar_painel` somava constante nomeada com número mágico (`FOLGA - 12.0`). Agora
  as duas metades têm nome: sangria da janela menos margem do cartão.
- "Avisar sobre versões novas" estava na seção **Problemas**, embaixo do diagnóstico.
- `Anel` era o único componente que se estilizava inline; foi para `anel.css`, levando o
  keyframe do giro, que morava no `base.css` global sem precisar.
- Saíram os 18 comentários que restavam no front, e os do `Cargo.toml` e do
  `vite.config.ts`.
- `Assets` virou `assets`, para combinar com `docs`, `public` e `src`.

### 36. O que passou a ser verificado sozinho

Não havia CI de verificação: o workflow só rodava no push de tag, então `cargo test`,
`tsc` e a formatação nunca rodavam antes da release.

Agora o `verificar.yml` roda em todo push e PR: `cargo fmt --check`, `cargo test`,
`npm run build`, e regera os artefatos para conferir que ninguém editou um `*.gerada.ts`
à mão. O `rustfmt.toml` fixa o estilo denso que o código já usava — sem ele, `cargo fmt`
reformatava 113 trechos e ninguém podia rodá-lo. E o `.gitattributes` acabou com a mistura
de CRLF e LF que vinha sujando os diffs.

---

## Em aberto

### 23. Limiar e aviso por controle

Hoje é um par de números para todos. Quem tem um controle que segura muito e outro que não
segura nada quer limiares diferentes.

**Não foi feito de propósito.** A ideia é minha, não veio de uso real; quem mantém o app
tem um controle só, então a tela para dois nunca seria exercitada; e o custo é superfície
de configuração nova numa parte que hoje é simples. Vale esperar alguém pedir.

### 24. Estimativa de tempo até carregar

No cabo o GATT some e não existe percentual para acompanhar a subida. Daria para inventar
a partir do último nível conhecido, e seria chute com cara de conta — o oposto do que o
resto do app faz. Fica registrado como recusado, não como pendente.

### 26. O primeiro veredito da saúde

A causa de ele nunca aparecer está resolvida na tarefa 30. O que falta é ver o primeiro
veredito de verdade e conferir contra os dados brutos, em vez de confiar de primeira. Com
a série desta máquina em 9 dias e o corte em 14, é questão de esperar.

### 37. A descarga final não é representativa, e entra na média

Nos dados desta máquina, o controle cai em degraus de 5 pontos que aceleram no fim: 57%
às 22:21 e 12% às 23:10, quarenta e cinco pontos em quarenta e nove minutos. É o colapso
de fim de carga, e é real — mas 34,6 %/h não descreve o consumo do controle, que nas
sessões anteriores fica entre 5 e 9 %/h.

O teto de 60 %/h de `descargas` deixa passar. A média da semana dilui, e `de_agora` só
vale para o trecho que fecha a série, então o estrago é limitado. **Não foi mexido:**
qualquer guarda aqui seria calibrada em uma semana de dados de um controle só, que é
exatamente o tipo de heurística que este app evita. Vale rever quando houver série de mais
de um aparelho.

### 38. Cores duplicadas entre o Rust e o CSS

`VERDE`, `AMBAR`, `VERMELHO`, `CINZA` e `FUNDO` vivem no `geometria.rs` e, com os mesmos
valores, no `tokens.css`. O módulo gerado já expõe os do Rust, e a `Marca` usa de lá — mas
o `tokens.css` continua com a cópia dele, porque é a camada CSS do sistema de design, e
gerar token de tema a partir do gerador de ícones inverte a hierarquia. São cinco valores;
se divergirem, aparece na tela.
