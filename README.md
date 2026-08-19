# Kontro

Bateria do seu controle, em tempo real, na bandeja do sistema.

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

## O que acontece no cabo

**Não existe percentual quando o controle está no cabo.** Isso não é limitação do app.

Ao ser plugado, o controle troca de protocolo e o GATT Battery Service desaparece. O que
sobra é a escala grosseira de quatro níveis do protocolo GIP, que erra feio: medindo um
controle com 77% de carga real, ela reportava 100%.

Por isso, no cabo, o Kontro mostra a **última leitura conhecida com o horário** em vez de
inventar um número. Mostrar nada é melhor que mostrar errado.

## Instalação

Baixe o `Setup.exe` da [última release](../../releases/latest). A instalação é por
usuário, não pede administrador, e o app se atualiza sozinho a partir das releases
publicadas aqui.

## Configuração

Pelo ícone na bandeja você ajusta:

- Iniciar com o Windows e iniciar minimizado
- O que o botão X faz: minimizar para a bandeja ou encerrar o app
- Limiares de aviso de bateria baixa e crítica
- Verificação automática de atualizações

> O Windows 11 esconde ícones novos da bandeja atrás da setinha `^`. Arraste o ícone do
> Kontro para fora uma vez e ele fica fixo.

## Compilando

Requer o SDK do .NET 8.

```bash
dotnet build -c Release
dotnet run
```

Sinalizadores úteis durante o desenvolvimento:

| Sinalizador | O que faz |
|---|---|
| `--make-icon <ico> [preview]` | Gera o `app.ico` a partir da geometria do controle |
| `--icon-preview <png>` | Renderiza todos os estados do ícone da bandeja |
| `--diagnose <txt>` | Despeja o que a descoberta enxerga: HID por usage, Bluetooth pareado, resultado |
| `--show` | Abre o painel já na inicialização |
| `--check-update [arquivo] [--apply]` | Consulta as releases e, com `--apply`, baixa e instala |

## Publicando uma versão

A build de release é automática. Basta empurrar uma tag:

```bash
git tag v1.1.0
git push origin v1.1.0
```

O GitHub Actions compila, gera o instalador com o Velopack e anexa tudo na release. A
versão do assembly sai da própria tag, então não existe número duplicado em dois lugares.

## Licença

MIT
