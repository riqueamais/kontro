# Plano de remodelação do Kontro

## Objetivo

Responder em segundos a três perguntas. **Quanto ainda dá para jogar? O que aconteceu nas minhas sessões? Onde ajusto o comportamento do app?** A bandeja serve para olhar e agir rápido. A janela principal serve para entender o histórico e configurar o Kontro.

O sistema visual parte do preto, do branco e do verde do Kontro, do controle dentro do anel e da leitura de carga. O verde #5FE083 é a referência da marca e da carga saudável. Verde para ações, verde para dados e cor de foco têm tokens distintos, mesmo quando usam tons próximos. Amarelo, vermelho e cinza comunicam estados da leitura junto com texto.

O projeto inclui a experiência completa no Windows. A moldura e o tamanho da janela, o ícone na barra de tarefas, o menu e o painel da bandeja, a pílula, os avisos, a primeira abertura, o instalador e a atualização fazem parte do mesmo produto.

## 0. Fixar a referência antes do código

- Usar a v2.15.0 e as capturas da landing como referência de identidade e temas. Usar a bandeja da v2.16.0 como referência de interação aprovada. Registrar capturas com os mesmos dados de exemplo para poder comparar.
- Confirmar no código as rotas Resumo, Diário e Configurações, o menu da bandeja e os atalhos. Levantar estados reais de leitura, pílula, avisos, atualização, sessões e múltiplos controles.
- Revisar a UI com a UI UX Pro Max. Aplicar contraste, hierarquia, foco e densidade ao contexto de um utilitário Windows. A busca não retornou orientação específica para o corte do tooltip. O diagnóstico desse ponto vem do código.
- Evidências de partida: [landing no repositório](./docs/index.html), [captura de temas](./docs/tela-temas.png), [identidade atual](./DESIGN.md), [tokens do app](./src/estilo/tokens.css). A página pública não carregou na consulta, então a análise usou o HTML e as capturas versionadas.

**Aceite.** Inventário dos fluxos e temas completo e referências comparáveis registradas. Reproduzir o corte do gráfico no Windows ou documentar a condição que falta para reproduzi-lo.

## 1. Aprovar arquitetura e sistema visual em protótipo

**Navegação.** Prototipar uma barra lateral curta com ícone e texto para Resumo, Diário e Configurações. Ponto de partida de 152–168 px, área central com 20–24 px de margem. Comparar com a navegação superior atual usando as mesmas tarefas. Na janela mínima, testar uma lateral mais estreita com rótulos visíveis. Só mudar para ícones sem texto se houver evidência de que a alternativa é mais fácil de usar. A posição da navegação fica decidida nessa etapa.

**Componentes.** Desenhar quatro peças reutilizáveis: cabeçalho de página, cartão de dado, lista de itens e grupo de ajustes. Cada cartão tem título, conteúdo e até uma ação principal. Métricas relacionadas dividem uma superfície. Usar bordas discretas, raio inicial de 8–10 px e escala de espaçamento 4/8/12/16/24/32 px. Segoe UI, títulos entre 24 e 28 px, texto entre 13 e 14 px, números tabulares nas leituras. Medidas iniciais para protótipo, sujeitas à captura na janela mínima.

**Temas.** Restaurar a referência de cada tema anterior ao cobre. Sistema acompanha o Windows. Noite é carvão e verde; Preto mantém fundo preto; Dia usa superfícies claras e verde escuro legível; Ardósia mantém seu azul e Brasa seu laranja nas interações. A cor da bateria continua semântica em todos. A amostra de tema deve usar os mesmos tokens que pinta na interface. Manter os valores persistidos de tema e testar mudança do Windows ao vivo para quem escolheu Sistema. Comparar as seis opções com a v2.15.0 e documentar diferenças intencionais.

**Aceite.** Capturas em 720 × 520 e 840 × 600, em Noite e Dia, com os mesmos dados. Escolher a navegação após testar abrir Resumo pela bandeja, encontrar uma sessão e chegar a um ajuste sem procurar pela tela inteira. Aprovar as três páginas, o padrão dos cartões e as amostras dos temas antes de implementar o conjunto. Contraste de texto normal ≥ 4,5:1 e foco visível em todos os temas.

### 1.1. Desenhar a janela inteira

- **Moldura e título.** Prototipar a janela com a barra de título completa, incluindo ícone, nome, área de arraste e botões de minimizar, maximizar/restaurar e fechar. Comparar a barra personalizada atual com a moldura do sistema. Escolher pela integração com Windows 10/11, pelos estados ativo e inativo e pelo espaço que sobra para o conteúdo. Testar a barra também maximizada, com a moldura encostada nas bordas da tela.
- **Espaçamento de fora para dentro.** Separar borda e sombra do sistema, margem da página, espaço entre seções e padding interno dos cartões. Começar com margem de 20 px na janela mínima e 24 px na padrão, intervalo de 12–16 px entre blocos e padding de 16 px dentro dos cartões. Alinhar o início do título, dos cartões e das listas ao mesmo eixo. Em janela larga, limitar a largura útil do conteúdo e usar duas colunas só quando a leitura melhorar. Não esticar textos e gráficos até a borda da tela.
- **Densidade e rolagem.** A navegação e o título da janela continuam acessíveis enquanto o conteúdo rola. Validar o primeiro quadro de cada tela com a barra de título visível, inclusive o Resumo em 720 × 520. Conferir padding inferior, foco perto das bordas e o espaço entre scrollbar e conteúdo.
- **Material e contorno.** Especificar borda, cantos e sombra em janela normal e maximizada, com foco ou sem foco, em claro e escuro. Validar o acabamento com transparência disponível e com fundo opaco. Evitar o efeito de duas molduras sobrepostas e o recorte de cantos ou controles.

**Aceite.** As capturas para aprovação mostram a janela inteira, contra fundos claros e escuros do desktop. Incluir a janela mínima, padrão, maximizada e encaixada. A decisão de moldura e a escolha de navegação precedem a implementação.

## 2. Implementar a estrutura e a fundação dos temas

- Atualizar [Principal.tsx](./src/telas/Principal.tsx), [principal.css](./src/telas/principal.css) e [tokens.css](./src/estilo/tokens.css). Definir uma área de conteúdo com rolagem por página e componentes compartilhados a partir dos padrões aprovados.
- Preservar atalhos, foco, posição de rolagem por página, notificações de versão nova e abertura direta pelo menu da bandeja. Adequar as setas à orientação da navegação escolhida.
- Alinhar [useTema](./src/estado.ts), amostras em Configurações e tokens para que a prévia corresponda ao resultado.
- Tratar a moldura como uma entrega própria entre [BarraDeTitulo.tsx](./src/componentes/BarraDeTitulo.tsx), [janelas.rs](./src-tauri/src/janelas.rs) e os eventos em [lib.rs](./src-tauri/src/lib.rs). Conferir arraste pela área livre, duplo clique no título, redimensionamento por lados e cantos, encaixe do Windows, restauração, Alt+Tab e Alt+F4. Fechar deve seguir a preferência de recolher para a bandeja ou sair. Reabrir pela bandeja deve restaurar uma posição utilizável, inclusive depois de remover um monitor.

**Aceite.** Configurações existentes abrem com o mesmo tema selecionado e mantêm seu comportamento após reiniciar. Fluxos de mouse e teclado funcionam em todas as páginas. Cada página tem apenas uma rolagem principal e não cria rolagem horizontal na janela mínima. Verificar janela normal, encaixada e maximizada em 100%, 125% e 150% e ao mover entre monitores com escalas diferentes.

## 3. Remodelar as três páginas por tarefa

### Resumo · quanto ainda dá para jogar

Em [Resumo.tsx](./src/telas/Resumo.tsx), a primeira superfície reúne controle ativo, carga, autonomia, conexão e horário da leitura. “Ler agora” ou “Procurar” fica no mesmo lugar nos dois estados. Autonomia e taxa só aparecem quando existem. Se há mais de um controle, mostrar a lista junto do estado atual, antes do histórico.

A segunda área agrupa histórico de carga, período de 7/30 dias e saúde da bateria. Abaixo, sessões recentes mostram atividade com um caminho claro para o Diário. Se selecionar uma sessão altera o gráfico, o período escolhido e a saída dessa seleção precisam ficar visíveis.

**Aceite.** Na janela mínima, a primeira dobra mostra o controle ativo, a leitura ou seu estado, e a ação principal. Testar carga exata, aproximada, crítica, leitura antiga, cabo sem porcentagem, sem controle e múltiplos controles. A mesma informação precisa ser entendida sem depender da cor.

### Diário · o que aconteceu nas sessões

Em [Diario.tsx](./src/telas/Diario.tsx), agrupar total de tempo, dias ativos e calendário em “Últimos 30 dias”. Selecionar um dia atualiza um detalhe fixo próximo ao calendário com tempo, jogos e menor carga quando conhecida. Mouse e teclado chegam ao mesmo detalhe. “Consumo por jogo” fica abaixo, com unidade e contexto para jogos ainda em medição. “Salvar cartão” pertence ao bloco que exporta.

**Aceite.** Testar diário vazio, um dia, sessões cruzando a meia-noite, nomes longos, ranking com dados insuficientes e exportação. Comparar totais e períodos com a versão anterior usando a mesma base. A seleção de dia tem indicador que funciona sem cor e não muda a posição do restante da página.

### Configurações · onde ajustar

Em [Configuracoes.tsx](./src/telas/Configuracoes.tsx), agrupar as opções em cinco destinos: **Geral**, **Aparência**, **Bateria e avisos**, **Pílula e atalhos**, **Versão e ajuda**. Exibir um grupo por vez, com título e descrição curta, e linhas dentro de uma superfície comum. A amostra de Aparência mostra um trecho real do padrão de interface. Pílula e atalhos reúne a prévia e as ações que a afetam.

Na verificação de atualizações, mostrar progresso, erro ou versão disponível junto ao botão que iniciou a busca. O caminho de entrada pela indicação de nova versão abre diretamente **Versão e ajuda**.

**Aceite.** Localizar tema, limiar, atalho e atualização sem percorrer grupos alheios. Testar os estados procurando, em dia, versão disponível e erro. O botão para instalar aparece ao lado do resultado sem exigir voltar ao topo. Conferir que salvamento e validação de atalhos e limiares mantêm o comportamento atual.

## 4. Preservar o painel da bandeja e corrigir o gráfico

Manter a ordem aprovada em [Painel.tsx](./src/telas/Painel.tsx): leitura, histórico, modos da pílula e ações rápidas. Manter o ícone nativo e seu comportamento. Aplicar ao painel apenas os tokens restaurados e os ajustes necessários à legibilidade e ao gráfico.

Em [Historico.tsx](./src/componentes/Historico.tsx) e [historico.css](./src/componentes/historico.css), a dica está centrada horizontalmente no ponto e muda de lado na vertical com um limite fixo. O painel de 328 px tem contêiner rolável. Essa combinação explica um possível corte nas bordas. Reproduzir primeiro com pontos à esquerda e à direita. Medir o tamanho real da dica e limitar sua posição à área visível com margem de 8 px. Escolher acima ou abaixo conforme o espaço e manter o marcador ancorado no ponto. Se a altura compacta inviabilizar uma dica legível, exibir a mesma leitura em uma faixa fixa abaixo do gráfico compacto. A dica não deve mexer na altura da janela a cada movimento do cursor.

**Aceite.** Dica inteira no primeiro e no último ponto, no alto e no baixo, com mouse e setas, nos temas Noite e Dia. Testar a bandeja nos cantos da tela e com escala de 100%, 125% e 150%. Verificar fechar com Escape, perda de foco, abertura do Resumo/Ajustes e modos da pílula.

## 5. Projetar as superfícies fora da janela

| Superfície | O que definir e validar |
| --- | --- |
| Ícone do app e barra de tarefas | Conferir o mesmo símbolo no título, Alt+Tab, barra de tarefas, menu Iniciar e atalho. Testar legibilidade em 16, 24 e 32 px, com barra clara e escura. Se houver alteração na marca, gerar os formatos a partir da fonte canônica do repositório. |
| Bandeja e menu de contexto | Manter a composição aprovada do painel. Conferir ícone, dica nativa, clique esquerdo e direito, nomes e agrupamento das ações, barra automática e área de ícones ocultos. |
| Pílula no jogo | Revisar margem da borda, escala, opacidade, contraste sobre cenas claras e escuras e estados de encaixe e movimentação. Testar jogo em janela, sem borda e tela cheia, alternância de monitor e controle desconectado. |
| Avisos | Conferir posição, recuo, leitura e desaparecimento dos avisos de conexão e carga. Verificar se o aviso aparece no monitor esperado e sem roubar foco do jogo. |
| Primeira abertura | Revisar o percurso em [Passos.tsx](./src/telas/Passos.tsx) com a moldura definitiva. Conferir pular, voltar, finalizar e rever pelo app. |
| Instalador, atualização e landing | Conferir o ícone e as artes do instalador, a atualização da versão anterior, a reabertura após instalar e a apresentação da nova versão. Atualizar as capturas da landing com a janela inteira e a bandeja. |

**Aceite.** O protótipo cobre a relação de cada superfície com o desktop, e a revisão final usa a versão empacotada. Registrar uma captura ou gravação por superfície. Para a pílula e os avisos, registrar também comportamento e foco durante o jogo.

## 6. Validar e liberar

- Fazer revisão visual lado a lado com a referência da etapa 0. Matriz mínima: 720 × 520, 840 × 600 e janela maximizada; todos os temas; teclado, alto contraste, movimento reduzido e texto ampliado; bandeja com escalas de 100%, 125% e 150%. Conferir ícone, aviso, pílula e primeira abertura.
- Executar build do frontend e a CI Windows do repo. Testar a versão empacotada com controle físico conectado, por cabo e Bluetooth quando disponíveis. Registrar separadamente o que foi simulado e o que foi observado com hardware.
- Atualizar as capturas da landing a partir da versão validada. Seguir o fluxo do repositório de publicar beta por develop, verificar instalação/atualização da versão anterior e promover a mesma versão para main após os aceites. Versão e data são decisões da etapa de release.
- Em cada entrega, registrar evidência, resultado e desvios do aceite. Uma etapa só fecha quando seus critérios passam ou quando um desvio é explicitamente aceito. A aprovação visual da etapa 1 antecede alterações nas telas.

## Recorte da primeira entrega

Entregar a matriz de temas e protótipos de Resumo, Diário e Configurações dentro da janela inteira. Comparar as navegações e a moldura em tamanho mínimo, padrão e maximizado. Incluir uma prancha com bandeja, pílula e avisos sobre o desktop e isolar o caso do tooltip. Navegação, moldura e aparência dos temas são as decisões de aprovação que destravam a implementação.

### Evidências da primeira rodada

- Protótipo navegável com dados simulados. Compara lateral compacta e abas superiores, moldura Kontro e aproximação da moldura Windows, quatro tamanhos de janela e seis opções de tema. Inclui Resumo, Diário, grupos de Configurações e uma vista de bandeja, pílula e aviso.
- No Edge, as três páginas passaram sem rolagem horizontal interna em 720 × 520, nas duas navegações e nos seis modos de tema. Testados também seleção de dia, resultado de atualização junto ao botão, troca do modo da pílula e abertura do Resumo pela bandeja.
- O corte da dica foi reproduzido no frontend atual com dados simulados. Em um painel de 328 px, a borda direita da dica do último ponto chegou a 391 px, 63 px além do painel. O teste selecionou o ponto com End. A confirmação pelo mouse no Windows, as escalas de DPI e a correção ficam na etapa 4.
- A moldura Windows no protótipo é uma aproximação visual. Foi escolhida a moldura personalizada já integrada ao app e a lateral compacta, que mantém os três rótulos visíveis na janela mínima. A barra de título tem 38 px, e o conteúdo tem margens de 20 a 24 px antes da limitação de largura nas janelas grandes.

### Implementação e validação

- O Resumo coloca os controles conhecidos antes do histórico e dá acesso ao Diário junto das sessões. O Diário mantém o detalhe do dia selecionado. Configurações mantém um grupo por vez e o cartão de atualização junto ao botão de busca. A prévia da pílula fica no grupo que altera seu comportamento.
- No gráfico compacto, foi adotada a faixa fixa de leitura prevista na etapa 4. Os testes no Edge usaram mouse e teclado no primeiro e no último ponto, com o conteúdo dentro dos 328 px e sem alteração na altura do painel.
- O frontend compilou. Foram testados cinco temas explícitos, janela mínima e larga, teclado, contraste forçado, busca de atualização simulada e primeira abertura. Os testes usaram dados simulados. Moldura, encaixe, transparência, escalas de DPI, avisos, jogo em tela cheia, atualização empacotada e controle físico ainda precisam de validação nativa antes da publicação.
