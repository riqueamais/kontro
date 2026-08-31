# Kontro

Bateria do seu controle, em tempo real, na bandeja do sistema.

**[kontro.riqueamais.github.io](https://riqueamais.github.io/kontro/)** · [baixar a última versão](https://github.com/riqueamais/kontro/releases/latest)

O ícone mostra a porcentagem exata e muda de cor conforme a carga cai. Quando não há
controle ligado, ele vira um controle riscado. O app avisa quando a bateria fica baixa
e estima quanto tempo ainda dá para jogar, a partir do consumo medido.

## Como ele lê a bateria

Controles sem fio expõem a carga pelo **GATT Battery Service** do Bluetooth LE. É a
mesma fonte que o Windows usa, dá o percentual exato de 0 a 100 e ainda suporta
notificação: o controle avisa sozinho quando o nível muda, sem ninguém ficar perguntando.

O Kontro não depende de marca nem de modelo. A descoberta funciona assim:

1. Enumera dispositivos **HID** cuja usage é gamepad, joystick ou multi-axis
2. Extrai o endereço Bluetooth do id da interface
3. Cruza com os dispositivos Bluetooth pareados

Quem não declara ser um controle nunca entra na lista. Teclado, mouse e fone continuam
invisíveis para o app mesmo tendo bateria e estando pareados.

## Como ele sabe se o controle está no cabo

A ordem é da prova mais direta para a menos. Se o controle foi encontrado por uma
interface Bluetooth, ele está sem fio e não há o que perguntar — um DualSense nem aparece
para o XInput, e um Xbox por Bluetooth se declara a ele como tipo desconhecido. Só quando
não há endereço nenhum a resposta vem do XInput, que distingue cabo de bateria no próprio
relatório.

Presença vale pelo instante em que foi medida. O Windows avisa quando a interface aparece
e some, e é esse aviso que dispara a varredura — não um relógio.

## O que acontece no cabo

**Não existe percentual quando o controle está no cabo.** Isso não é limitação do app.

Ao ser plugado, o controle troca de protocolo e o GATT Battery Service desaparece. O que
sobra é a escala grosseira de quatro níveis do protocolo GIP, que erra feio: medindo um
controle com 77% de carga real, ela reportava 100%.

Por isso, no cabo, o Kontro mostra a **última leitura conhecida, com a hora e a data** em
vez de inventar um número. Mostrar nada é melhor que mostrar errado — e uma leitura de
outro dia diz na tela que é de outro dia.

## Instalação

Baixe o `Setup.exe` da [última release](../../releases/latest). A instalação é por
usuário, não pede administrador, e o app se atualiza sozinho a partir das releases
publicadas aqui.

## Configuração

Na primeira abertura o app se explica em seis telas — o que o ícone diz, por que não há
percentual no cabo, e a pílula solta na tela para você arrastar até onde quiser. Dá para
rever quando quiser, em Configurações.

Pelo ícone na bandeja, em Configurações:

- Iniciar com o Windows e iniciar minimizado
- O que o botão X faz: minimizar para a bandeja ou encerrar o app
- Limiares de aviso de carga baixa e de carga crítica
- Sobreposição: quando aparece, em qual tela, tamanho e transparência
- Onde a pílula mora: solte ela e arraste até qualquer ponto da tela. Perto de um
  canto ou do meio da borda ela encaixa sozinha
- Atalhos, que valem por cima do jogo: `Ctrl + Shift + K` mostra e esconde a pílula e
  `Ctrl + Shift + M` solta ela para arrastar. As duas combinações são trocáveis — clique
  no atalho e tecle a que você quiser
- Verificação automática de atualizações

> O Windows 11 esconde ícones novos da bandeja atrás da setinha `^`. Arraste o ícone do
> Kontro para fora uma vez e ele fica fixo.

## Compilando

Requer Node 20 e a toolchain estável do Rust.

```bash
npm install
npm run tauri dev
npm run tauri build
```

Sinalizadores úteis durante o desenvolvimento:

| Sinalizador | O que faz |
|---|---|
| `--gerar [raiz]` | Redesenha tudo que é derivado do Rust: ícones, geometria e a interface `Config` do front |
| `--icon-preview <png> [tamanho] [--claro]` | Renderiza os estados do ícone da bandeja, ampliados |
| `--diagnose <txt>` | Despeja o que a descoberta enxerga: HID por usage, Bluetooth pareado, o que o monitor conclui |
| `--show` | Abre o painel já na inicialização |
| `--painel` | Abre direto o painel da bandeja |
| `--minimizado` | Sobe só para a bandeja, sem janela |

### O que é gerado, e o que é escrito à mão

Nada que exista em dois lugares é digitado duas vezes. Um comando redesenha tudo que
deriva do Rust:

```bash
cargo run --release -- --gerar .
```

| sai de | vira |
|---|---|
| `geometria.rs` | `src-tauri/icons/`, ícones de `docs/`, `setup.ico`, `public/favicon.svg`, `assets/svg/` |
| `geometria.rs` | `src/estilo/geometria.gerada.ts` — o path da marca e os raios que `Glifo` e `Marca` desenham |
| `configuracoes.rs` | `src/config.gerada.ts` — a interface `Config` que o front usa |

Se algo aparecer no `git status` depois de rodar, é porque a fonte mudou e os arquivos
estavam atrasados. A CI roda o mesmo comando e recusa o push quando isso acontece.

Arquivos `*.gerada.ts` não se editam: a próxima execução sobrescreve. Um campo novo em
`Settings` sem entrada em `CAMPOS_DA_CONFIG` quebra o teste, e não a interface em silêncio.

### Convenções

- **Nomes**: tudo em português, arquivos inclusive. Ficam em inglês só as siglas de
  protocolo (`gatt`, `hid`, `pnp`, `xinput`) e os dois tipos que espelham arquivo em
  disco — `Settings` e `ControleSalvo` — porque o nome dos campos deles é o nome das
  chaves em `settings.json` e `controllers.json`.
- **Sem comentários.** O nome diz o que faz; o porquê vai na mensagem de commit e no
  `TAREFAS.md`.
- **CSS**: cada folha leva o nome da classe raiz que ela escopa, e *todo* seletor dela
  começa por essa raiz. As quatro janelas compartilham um bundle só, então classe de nome
  genérico sem escopo vaza de uma janela para as outras — já vazou.
- **Formatação**: `cargo fmt` com o `rustfmt.toml` do repositório. A CI confere.

## Publicando uma versão

A build de release é automática. Basta empurrar uma tag:

```bash
git tag v1.1.0
git push origin v1.1.0
```

O GitHub Actions compila, gera o instalador NSIS, assina o pacote e publica o `latest.json`
que o próprio app consulta para se atualizar. A versão sai da própria tag e é carimbada no
`Cargo.toml` e no `tauri.conf.json`, então não existe número duplicado em dois lugares.

## Licença

MIT
