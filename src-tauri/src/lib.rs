//! Montagem do app: as janelas, a bandeja, o ciclo de leitura e a ponte com a interface.
//!
//! O monitor vive numa thread propria e nunca atravessa a fronteira: o que a interface
//! recebe e o estado ja pronto. Isso mantem os objetos do WinRT presos a um unico
//! apartamento e evita que a leitura trave o desenho.

mod atalho;
mod atualizacao;
mod autostart;
mod device;
mod diagnostico;
mod geometria;
mod history;
mod icones;
mod janelas;
mod known;
mod model;
mod monitor;
mod orquestra;
mod paths;
mod settings;
mod tela;
mod tempo;
mod tray;

use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri_plugin_notification::NotificationExt;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::history::Amostra;
use crate::model::BatteryState;
use crate::settings::{CloseAction, Settings};

/// O que a interface le. Nunca contem objeto do sistema, so dado ja apurado.
pub struct Compartilhado {
    /// O controle que manda no icone, na sobreposicao e no aviso.
    estado: Mutex<BatteryState>,
    /// Todos os conhecidos, para o painel listar.
    todos: Mutex<Vec<BatteryState>>,
    serie: Mutex<Vec<Amostra>>,
    saude: Mutex<Option<history::Saude>>,
    /// Escolha do atalho: nula enquanto a regra manda, `Some` depois que o usuario mexeu.
    sobreposicao_a_mao: Mutex<Option<bool>>,
    config: Mutex<Settings>,
    /// Versao nova encontrada, para a janela de configuracoes mostrar.
    novidade: Mutex<Option<atualizacao::Novidade>>,
}

/// Pedidos que a interface faz ao ciclo de leitura.
enum Pedido {
    LerAgora,
    Renomear { chave: String, nome: String },
    Esquecer { chave: String },
    /// Grave o que ainda esta so na memoria e pare. O canal de volta e o aperto de mao:
    /// sem ele quem pediu segue em frente e o processo morre antes da gravacao acabar.
    Encerrar(mpsc::Sender<()>),
}

const INTERVALO_DO_CICLO: Duration = Duration::from_secs(2);

pub fn executar() {
    // O diagnostico roda antes de qualquer janela existir: ele nao precisa de interface,
    // e quem o pede esta atras de um arquivo para mandar, nao de um app aberto.
    let argumentos: Vec<String> = std::env::args().collect();
    if let Some(i) = argumentos.iter().position(|a| a == "--diagnose") {
        let destino = argumentos
            .get(i + 1)
            .cloned()
            .unwrap_or_else(|| "kontro-diagnostico.txt".to_string());

        device::iniciar_apartamento();
        match diagnostico::escrever(&destino) {
            Ok(()) => println!("diagnostico salvo em {destino}"),
            Err(e) => eprintln!("nao consegui salvar o diagnostico: {e}"),
        }
        return;
    }

    if let Some(i) = argumentos.iter().position(|a| a == "--icon-preview") {
        let destino = argumentos.get(i + 1).cloned().unwrap_or_else(|| "icone.png".into());
        let tamanho = argumentos
            .get(i + 2)
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(tray::tamanho_do_icone);
        // `--claro` agora escolhe so o fundo da previa: e sobre barra clara que o
        // contraste do icone e mais dificil de sustentar.
        let fundo_claro = argumentos.iter().any(|a| a == "--claro");
        match tray::salvar_previa(&destino, tamanho, fundo_claro) {
            Some(()) => println!("previa de {tamanho}px salva em {destino}"),
            None => eprintln!("nao consegui desenhar a previa"),
        }
        return;
    }

    if let Some(i) = argumentos.iter().position(|a| a == "--gerar-icones") {
        let raiz = argumentos.get(i + 1).cloned().unwrap_or_else(|| ".".into());
        match icones::gerar(&raiz) {
            Ok(()) => println!("icones redesenhados a partir de {raiz}"),
            Err(e) => eprintln!("nao consegui gravar os icones: {e}"),
        }
        return;
    }

    let mut config = Settings::carregar();

    // O usuario pode ter tirado o app da inicializacao pelo Gerenciador de Tarefas. O
    // registro e a verdade; o arquivo so guarda a preferencia.
    config.start_with_windows = autostart::ligado();

    let compartilhado = Arc::new(Compartilhado {
        estado: Mutex::new(model::BatteryState::montar(
            model::LinkMode::Offline,
            None,
            model::Precisao::Nenhuma,
            None,
            None,
            false,
            true,
            "Procurando controle".to_string(),
            None,
            "wired".to_string(),
            0,
            None,
        )),
        serie: Mutex::new(Vec::new()),
        saude: Mutex::new(None),
        sobreposicao_a_mao: Mutex::new(None),
        todos: Mutex::new(Vec::new()),
        config: Mutex::new(config),
        novidade: Mutex::new(None),
    });

    let (envio, recebimento) = mpsc::channel::<Pedido>();
    let ao_encerrar = envio.clone();

    let app = tauri::Builder::default()
        // Precisa vir antes dos outros: quando ja ha uma instancia rodando, este plugin
        // encerra a nova imediatamente, e nada mais chega a subir.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Abrir o app de novo com ele ja aberto significa "quero ver o Kontro" --
            // entao a segunda tentativa traz a janela de configuracoes para a frente em
            // vez de morrer em silencio.
            if let Some(janela) = app.get_webview_window(janelas::PRINCIPAL) {
                let _ = janela.unminimize();
                let _ = janela.show();
                let _ = janela.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(atalho::plugin())
        .manage(compartilhado.clone())
        .manage(envio.clone())
        .invoke_handler(tauri::generate_handler![
            estado_atual,
            controles,
            renomear_controle,
            esquecer_controle,
            serie_do_historico,
            saude_da_bateria,
            configuracoes,
            salvar_configuracoes,
            ler_agora,
            versao_disponivel,
            procurar_atualizacao,
            mostrar_janela,
            esconder_janela,
            quantidade_de_telas,
            ajustar_altura_do_painel
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            janelas::criar_todas(&handle)?;
            montar_bandeja(&handle)?;

            atalho::aplicar(&handle, compartilhado.config.lock().unwrap().overlay_shortcut_enabled);

            // `--show` abre a janela mesmo com "iniciar minimizado" ligado. Serve para
            // o atalho do menu e para conferir a interface sem ter de mexer na
            // configuracao do usuario so para ver uma tela.
            let pedido_explicito = std::env::args().any(|a| a == "--show");

            // Abre o painel da bandeja direto. Ele so aparece por clique no icone, o que
            // torna impossivel conferir o desenho sem a mao no mouse.
            if std::env::args().any(|a| a == "--painel") {
                if let Some(painel) = handle.get_webview_window(janelas::PAINEL) {
                    janelas::posicionar_painel(&handle);
                    let _ = painel.show();
                    // sem foco ele se esconde sozinho no mesmo instante, que e o
                    // comportamento correto e o que torna a conferencia impossivel
                    let _ = painel.set_focus();
                }
            }
            let subiu_com_o_sistema = std::env::args().any(|a| a == "--minimizado");
            let abrir_direto = pedido_explicito
                || (!subiu_com_o_sistema && {
                    let cfg = compartilhado.config.lock().unwrap();
                    !cfg.start_minimized || !cfg.first_run_done
                });
            if abrir_direto {
                if let Some(j) = handle.get_webview_window(janelas::PRINCIPAL) {
                    let _ = j.show();
                    let _ = j.set_focus();
                }
            }

            // A primeira execucao acabou de acontecer, entao ela precisa ficar
            // registrada. Sem isto a marca nasce desligada e nunca muda: o app se
            // apresenta a cada inicializacao, para sempre, e parece quebrado. So nao
            // aparecia em quem migrou do app antigo, que ja tinha a marca gravada.
            {
                let mut cfg = compartilhado.config.lock().unwrap();
                if !cfg.first_run_done {
                    cfg.first_run_done = true;
                    cfg.salvar();
                }
            }

            iniciar_ciclo(handle, compartilhado.clone(), recebimento);
            Ok(())
        })
        .on_window_event(|janela, evento| {
            let tauri::WindowEvent::CloseRequested { api, .. } = evento else { return };
            if janela.label() != janelas::PRINCIPAL {
                return;
            }

            // Por padrao o X so esconde: o app vive na bandeja, e quem fecha a janela
            // quase sempre quer tirar ela da frente, nao parar de monitorar. Mas a
            // escolha esta na tela, e ate agora ela nao fazia nada -- oferecer "Encerrar"
            // e continuar minimizando e pior do que nao oferecer.
            let encerrar = janela
                .app_handle()
                .try_state::<Arc<Compartilhado>>()
                .map(|c| c.config.lock().unwrap().close_action == CloseAction::Exit)
                .unwrap_or(false);

            if encerrar {
                janela.app_handle().exit(0);
                return;
            }

            api.prevent_close();
            let _ = janela.hide();
        })
        .build(tauri::generate_context!())
        .expect("nao foi possivel iniciar o Kontro");

    // `Builder::run` nunca devolve o controle: por baixo dele o laco de eventos tem
    // assinatura `-> !` e o processo sai por `process::exit`. Todo codigo escrito depois
    // dele e codigo morto -- e era ali que morava a unica gravacao do historico. Na
    // pratica o arquivo nunca era escrito, e depois de reiniciar o computador o app
    // abria mostrando como "ultima leitura" o que sobrara de sessoes muito anteriores.
    //
    // Montando em duas etapas, o evento de saida chega antes do processo acabar, e da
    // para esperar a thread do monitor terminar de gravar.
    app.run(move |_app, evento| {
        if !matches!(evento, tauri::RunEvent::Exit) {
            return;
        }
        let (feito, esperar) = mpsc::channel();
        if ao_encerrar.send(Pedido::Encerrar(feito)).is_ok() {
            let _ = esperar.recv_timeout(Duration::from_secs(3));
        }
    });
}

/// O ciclo de leitura, na sua propria thread.
fn iniciar_ciclo(
    app: AppHandle,
    compartilhado: Arc<Compartilhado>,
    pedidos: mpsc::Receiver<Pedido>,
) {
    std::thread::spawn(move || {
        // toda chamada ao WinRT exige apartamento; esta thread bloqueia, entao MTA
        device::iniciar_apartamento();

        let mut monitor = monitor::Monitor::novo();
        let mut ultimo_icone = String::new();
        let mut orquestrador = orquestra::Orquestrador::novo();
        let mut proxima_checagem = atualizacao::ultima_checagem() + atualizacao::JANELA_MS;

        loop {
            if let Some(panorama) = monitor.ciclo() {
                let principal = panorama.principal.clone();
                {
                    let mut atual = compartilhado.estado.lock().unwrap();
                    *atual = principal.clone();
                }
                {
                    let mut todos = compartilhado.todos.lock().unwrap();
                    *todos = panorama.todos.clone();
                }
                {
                    let mut serie = compartilhado.serie.lock().unwrap();
                    *serie = monitor.historico().serie(&principal.key).to_vec();
                }
                {
                    let mut saude = compartilhado.saude.lock().unwrap();
                    *saude = Some(monitor.historico().saude(&principal.key));
                }

                let _ = app.emit("kontro://estado", &principal);
                let _ = app.emit("kontro://controles", &panorama.todos);
                atualizar_bandeja(&app, &principal, &mut ultimo_icone);
            }

            if tempo::agora() >= proxima_checagem {
                proxima_checagem = tempo::agora() + atualizacao::JANELA_MS;
                let liberado = compartilhado.config.lock().unwrap().auto_check_updates;
                if liberado {
                    avisar_versao_nova(&app);
                }
            }

            // Fora do `if`: a visibilidade da sobreposicao depende do que ocupa a tela,
            // nao do estado da bateria. Reavaliar so na mudanca de carga faria entrar e
            // sair de um jogo nao ter efeito ate a proxima variacao de percentual.
            {
                let estado = compartilhado.estado.lock().unwrap().clone();
                let cfg = compartilhado.config.lock().unwrap().clone();
                let mao = *compartilhado.sobreposicao_a_mao.lock().unwrap();
                orquestrador.reavaliar(&app, &estado, &cfg, mao);
            }

            // A espera e o proprio ponto de escuta: dormir e so conferir depois fazia o
            // botao "Atualizar" levar ate dois segundos para ser notado, e o pedido de
            // encerramento chegar tarde demais para valer alguma coisa.
            match pedidos.recv_timeout(INTERVALO_DO_CICLO) {
                Ok(Pedido::Encerrar(feito)) => {
                    monitor.salvar();
                    let _ = feito.send(());
                    break;
                }
                Ok(Pedido::Renomear { chave, nome }) => monitor.renomear(&chave, &nome),
                Ok(Pedido::Esquecer { chave }) => monitor.esquecer(&chave),
                Ok(Pedido::LerAgora) => monitor.ler_agora(),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

fn montar_bandeja(app: &AppHandle) -> tauri::Result<()> {
    let abrir = MenuItem::with_id(app, "abrir", "Configurações", true, None::<&str>)?;
    let atualizar = MenuItem::with_id(app, "atualizar", "Ler a bateria agora", true, None::<&str>)?;
    let sair = MenuItem::with_id(app, "sair", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&abrir, &atualizar, &sair])?;

    TrayIconBuilder::with_id("kontro")
        .tooltip("Kontro")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, evento| match evento.id.as_ref() {
            "abrir" => {
                if let Some(j) = app.get_webview_window(janelas::PRINCIPAL) {
                    let _ = j.show();
                    let _ = j.set_focus();
                }
            }
            "atualizar" => {
                if let Some(envio) = app.try_state::<mpsc::Sender<Pedido>>() {
                    let _ = envio.send(Pedido::LerAgora);
                }
            }
            "sair" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|icone, evento| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = evento {
                let app = icone.app_handle();
                if let Some(painel) = app.get_webview_window(janelas::PAINEL) {
                    if painel.is_visible().unwrap_or(false) {
                        let _ = painel.hide();
                    } else {
                        janelas::posicionar_painel(app);
                        let _ = painel.show();
                        let _ = painel.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Redesenha o icone so quando o desenho realmente muda.
///
/// O ciclo roda a cada dois segundos; rasterizar de novo um icone identico seria
/// trabalho jogado fora num app que passa o dia parado.
fn atualizar_bandeja(app: &AppHandle, estado: &BatteryState, ultimo: &mut String) {
    let Some(bandeja) = app.tray_by_id("kontro") else { return };

    // A dica e so texto, e refaze-la nao custa nada. Presa a assinatura do desenho, ela
    // continuava dizendo "no cabo" depois de o controle comecar a carregar: `charging`
    // muda o texto da ligacao e nao muda desenho nenhum.
    let dica = format!(
        "{} - {} - {}",
        estado.device_name, estado.texto_da_carga, estado.texto_da_ligacao
    );
    let _ = bandeja.set_tooltip(Some(dica));

    let tamanho = tray::tamanho_do_icone();
    // o tamanho entra na assinatura porque trocar a escala da tela muda o desenho
    let assinatura = format!("{:?}|{:?}|{tamanho}", estado.mode, estado.preenchimento);
    if assinatura == *ultimo {
        return;
    }
    *ultimo = assinatura;

    if let Some(icone) = tray::desenhar(estado, tamanho) {
        let _ = bandeja.set_icon(Some(icone));
    }
}

/// Avisa que saiu versao nova, pela notificacao do sistema.
///
/// A consulta vai numa thread propria: a rede pode demorar, e o ciclo de leitura nao
/// pode ficar parado esperando o GitHub responder.
fn avisar_versao_nova(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let atualizacao::Consulta::Nova(novidade) = atualizacao::procurar() else {
            return;
        };

        // A notificacao do sistema fica na Central de Acoes para ser lida depois. O
        // balao antigo do Windows piscava e sumia, o que para um aviso que pede uma
        // acao do usuario e o mesmo que nao avisar.
        let _ = app
            .notification()
            .builder()
            .title(format!("Kontro {} disponivel", novidade.versao))
            .body("Abra a pagina da release para baixar o instalador.")
            .show();

        // o estado precisa de nome proprio: encadear a chamada descartaria o
        // emprestimo antes do cadeado ser usado
        let compartilhado = app.state::<Arc<Compartilhado>>();
        *compartilhado.novidade.lock().unwrap() = Some(novidade);
    });
}

// ------------------------------------------------------------------ comandos

#[tauri::command]
fn estado_atual(compartilhado: tauri::State<Arc<Compartilhado>>) -> BatteryState {
    compartilhado.estado.lock().unwrap().clone()
}

#[tauri::command]
fn controles(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<BatteryState> {
    compartilhado.todos.lock().unwrap().clone()
}

/// Troca o nome com que um controle aparece.
///
/// O nome do sistema continua guardado: o apelido so cobre a exibicao. Sem isso, um
/// controle sem Bluetooth fica para sempre chamado "controlador de jogo compativel com
/// HID" -- que e o que o Windows responde, e serve para todos igualmente.
#[tauri::command]
fn renomear_controle(chave: String, nome: String, envio: tauri::State<mpsc::Sender<Pedido>>) {
    let _ = envio.send(Pedido::Renomear { chave, nome });
}

/// Tira um controle da lista.
#[tauri::command]
fn esquecer_controle(chave: String, envio: tauri::State<mpsc::Sender<Pedido>>) {
    let _ = envio.send(Pedido::Esquecer { chave });
}

#[tauri::command]
fn serie_do_historico(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<Amostra> {
    compartilhado.serie.lock().unwrap().clone()
}

#[tauri::command]
fn saude_da_bateria(compartilhado: tauri::State<Arc<Compartilhado>>) -> Option<history::Saude> {
    compartilhado.saude.lock().unwrap().clone()
}

#[tauri::command]
fn configuracoes(compartilhado: tauri::State<Arc<Compartilhado>>) -> Settings {
    compartilhado.config.lock().unwrap().clone()
}

#[tauri::command]
fn salvar_configuracoes(
    app: AppHandle,
    compartilhado: tauri::State<Arc<Compartilhado>>,
    novas: Settings,
) {
    let mut novas = novas;

    // Guardar a preferencia nao basta: quem faz o app subir com o sistema e a chave do
    // registro. Se a escrita falhar, a configuracao volta a dizer a verdade em vez de
    // prometer o que nao vai acontecer.
    {
        let anterior = compartilhado.config.lock().unwrap().start_with_windows;
        if novas.start_with_windows != anterior
            && !autostart::definir(novas.start_with_windows)
        {
            novas.start_with_windows = autostart::ligado();
        }
    }

    // Mexer no modo da sobreposicao devolve a palavra a regra: se o usuario acabou de
    // dizer como ele quer a pilula, uma escolha antiga do atalho contradiria o que ele
    // acabou de pedir.
    {
        let anterior = compartilhado.config.lock().unwrap().overlay_mode;
        if novas.overlay_mode != anterior {
            *compartilhado.sobreposicao_a_mao.lock().unwrap() = None;
        }
    }

    atalho::aplicar(&app, novas.overlay_shortcut_enabled);

    novas.ajustar();
    novas.salvar();
    janelas::posicionar_sobreposicao(&app, &novas);

    // A pilula desenha com o tamanho e a opacidade escolhidos, e ela e uma janela a
    // parte: sem este aviso ela so mudaria de aparencia na proxima vez que fosse aberta.
    let _ = app.emit("kontro://config", &novas);

    *compartilhado.config.lock().unwrap() = novas;
}

#[tauri::command]
fn ler_agora(envio: tauri::State<mpsc::Sender<Pedido>>) {
    let _ = envio.send(Pedido::LerAgora);
}

#[derive(serde::Serialize)]
struct VersaoNova {
    versao: String,
    pagina: String,
    atual: String,
}

#[tauri::command]
fn versao_disponivel(compartilhado: tauri::State<Arc<Compartilhado>>) -> Option<VersaoNova> {
    compartilhado.novidade.lock().unwrap().clone().map(|n| VersaoNova {
        versao: n.versao,
        pagina: n.pagina,
        atual: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(serde::Serialize)]
struct Busca {
    estado: &'static str,
    versao: Option<String>,
    pagina: Option<String>,
    atual: String,
    motivo: Option<String>,
}

#[tauri::command]
fn procurar_atualizacao(compartilhado: tauri::State<Arc<Compartilhado>>) -> Busca {
    let atual = env!("CARGO_PKG_VERSION").to_string();

    match atualizacao::procurar() {
        atualizacao::Consulta::Nova(n) => {
            *compartilhado.novidade.lock().unwrap() = Some(n.clone());
            Busca {
                estado: "nova",
                versao: Some(n.versao),
                pagina: Some(n.pagina),
                atual,
                motivo: None,
            }
        }
        atualizacao::Consulta::EmDia => {
            *compartilhado.novidade.lock().unwrap() = None;
            Busca {
                estado: "em-dia",
                versao: None,
                pagina: None,
                atual,
                motivo: None,
            }
        }
        atualizacao::Consulta::Falhou(motivo) => Busca {
            estado: "falhou",
            versao: None,
            pagina: None,
            atual,
            motivo: Some(motivo),
        },
    }
}

/// Quantas telas existem, para a escolha do monitor da sobreposicao.
///
/// A tela oferecia "Monitor 1" e "Monitor 2" e nada mais: quem tem tres nunca alcancava
/// a terceira, porque o ciclo estava escrito com o numero dois dentro dele.
#[tauri::command]
fn quantidade_de_telas(app: AppHandle) -> usize {
    app.available_monitors().map(|m| m.len()).unwrap_or(1)
}

#[tauri::command]
fn mostrar_janela(app: AppHandle, rotulo: String) {
    if let Some(j) = app.get_webview_window(&rotulo) {
        let _ = j.show();
    }
}

/// O painel diz de quanto espaco precisa, e a janela obedece.
///
/// Fixar a altura na criacao significa acertar na regua toda vez que o conteudo muda --
/// e quando erra, o usuario ve os botoes cortados na borda. Quem sabe a altura e o
/// proprio painel, depois de desenhado.
#[tauri::command]
fn ajustar_altura_do_painel(app: AppHandle, altura: f64) {
    let Some(janela) = app.get_webview_window(janelas::PAINEL) else { return };

    // limites frouxos, so para uma medida absurda nao virar uma janela absurda
    let altura = altura.clamp(200.0, 900.0);
    let _ = janela.set_size(tauri::LogicalSize::new(janelas::LARGURA_DO_PAINEL, altura));
    janelas::posicionar_painel(&app);
}

#[tauri::command]
fn esconder_janela(app: AppHandle, rotulo: String) {
    if let Some(j) = app.get_webview_window(&rotulo) {
        let _ = j.hide();
    }
}
