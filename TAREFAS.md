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

## Em aberto

### 23. A autonomia confia demais num trecho curto

Apareceu nos dados reais: com 83% e uma queda de um ponto em dezesseis minutos, o app
anunciou "~22 h 14 min de jogo". A conta fecha -- 3,75 %/h -- mas dezesseis minutos de
controle parado na mesa não sustentam uma projeção de vinte e duas horas. `descargas`
aceita qualquer trecho com quinze minutos e um ponto de queda, e o trecho de agora ganha
do histórico sem precisar provar nada.

Provável correção: o trecho de agora só manda sozinho quando já caiu o bastante para
significar alguma coisa -- cinco pontos, por exemplo. Abaixo disso, a média da semana é
mais honesta.

### 24. Conferir a pílula com dois controles na tela

O desenho foi verificado por aritmética — dois controles ocupam 219 px numa janela de 284,
quatro ocupam 359 numa de 452, não há corte em nenhuma contagem — mas ninguém olhou se
fica bonito. Falta uma tela livre e um segundo controle.

### 25. Dois mecanismos de atualização

`atualizacao.rs` consulta o mesmo `latest.json` que o `tauri-plugin-updater` consulta, e
reimplementa a comparação de versão que o plugin já faz. A tela usa o caminho Rust para
saber se há novidade e depois chama `check()` do plugin para instalar — duas requisições.
Não foi mexido de propósito: é o caminho de atualização, está funcionando, e a
consolidação merece um passo isolado.

### 26. `Novidade.pagina` nunca é mostrada

Calculada, atravessa a fronteira em `VersaoNova`, e nenhuma tela renderiza o link.

### 27. Contrato de serialização inconsistente

`BatteryState` sai em camelCase e `Saude` em snake_case, e por isso `Saude.tsx` carrega um
`SaudeCrua` só para traduzir nomes. O `interface Config` do TypeScript ainda reescreve
`Settings` à mão em PascalCase — agora num lugar só, mas sem nada garantindo que os dois
não divirjam.

### 28. Capabilities largas

`updater:*` e `process:allow-restart` continuam liberados para as quatro janelas,
inclusive a sobreposição e o aviso. Vale uma capability restrita à `principal`.

### 29. `gatt::conectado` a cada dois segundos

É uma ida ao WinRT por controle conhecido, por ciclo. O certo é ouvir
`ConnectionStatusChanged` no `BluetoothLEDevice` que o `VinculoGatt` já segura.

### 30. Enumeração completa do PnP repetida

`pnp::nos_com_bateria()` enumera todos os nós de dispositivo da máquina, e
`ler_sem_bluetooth` pode chamá-la duas vezes por controle por leitura. Enumerar uma vez
por ciclo e passar a lista adiante.

### 31. Limiar e aviso por controle

Hoje é um par de números para todos. Quem tem um controle que segura muito e outro que
não segura nada quer limiares diferentes.

### 32. Registro de sessão

"Desligou com 68% às 22:14, depois de 3 h 20" — o histórico já tem os dados, falta a tela.

### 33. Diagnóstico pela interface

`--diagnose` só existe na linha de comando, e o executável é do subsistema gráfico. Um
botão em Configurações que grava o arquivo e abre a pasta resolve para quem for reportar
um problema.

### 34. Estimativa de tempo até carregar

No cabo o app não tem o que dizer. Com o histórico de subida dá para estimar quanto falta
para encher.
