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

Pelo ícone na bandeja, em Configurações:

- Iniciar com o Windows e iniciar minimizado
- O que o botão X faz: minimizar para a bandeja ou encerrar o app
- Limiares de aviso de carga baixa e de carga crítica
- Sobreposição: quando aparece, em que canto, em qual tela, tamanho e transparência
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
| `--gerar-icones [raiz]` | Redesenha todos os ícones do repositório a partir da geometria |
| `--icon-preview <png> [tamanho] [--claro]` | Renderiza os estados do ícone da bandeja, ampliados |
| `--diagnose <txt>` | Despeja o que a descoberta enxerga: HID por usage, Bluetooth pareado, o que o monitor conclui |
| `--show` | Abre o painel já na inicialização |
| `--painel` | Abre direto o painel da bandeja |
| `--minimizado` | Sobe só para a bandeja, sem janela |

### Ícones

Todos os ícones saem de `src-tauri/src/geometria.rs`. Nenhum é editado à mão:

```bash
cargo run --release -- --gerar-icones .
```

O comando redesenha `src-tauri/icons/`, os ícones de `docs/`, o `setup.ico` do instalador,
o favicon da interface e os vetores de referência em `assets/svg/`. Se algo aparecer no
`git status` depois de rodar, é porque a geometria mudou e os arquivos estavam atrasados.

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
