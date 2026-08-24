//! Montagem do app: as janelas, a bandeja, o ciclo de leitura e a ponte com a interface.
//!
//! O monitor vive numa thread propria e nunca atravessa a fronteira: o que a interface
//! recebe e o estado ja pronto. Isso mantem os objetos do WinRT presos a um unico
//! apartamento e evita que a leitura trave o desenho.

mod atualizacao;
mod autostart;
mod device;
mod diagnostico;
mod geometria;
mod history;
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
use crate::settings::Settings;

/// O que a interface le. Nunca contem objeto do sistema, so dado ja apurado.
pub struct Compartilhado {
    estado: Mutex<BatteryState>,
    serie: Mutex<Vec<Amostra>>,
    config: Mutex<Settings>,
    /// Versao nova encontrada, para a janela de configuracoes mostrar.
    novidade: Mutex<Option<atualizacao::Novidade>>,
}

/// Pedidos que a interface faz ao ciclo de leitura.
enum Pedido {
    LerAgora,
    Encerrar,
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
        config: Mutex::new(config),
        novidade: Mutex::new(None),
    });

    let (envio, recebimento) = mpsc::channel::<Pedido>();

    tauri::Builder::default()
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
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(compartilhado.clone())
        .manage(envio.clone())
        .invoke_handler(tauri::generate_handler![
            estado_atual,
            serie_do_historico,
            configuracoes,
            salvar_configuracoes,
            ler_agora,
            versao_disponivel,
            procurar_atualizacao,
            mostrar_janela,
            esconder_janela,
            ajustar_altura_do_painel
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            janelas::criar_todas(&handle)?;
            montar_bandeja(&handle)?;

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
            // fechar a janela de configuracoes nao encerra o app: ele vive na bandeja
            if let tauri::WindowEvent::CloseRequested { api, .. } = evento {
                if janela.label() == janelas::PRINCIPAL {
                    api.prevent_close();
                    let _ = janela.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("nao foi possivel iniciar o Kontro");

    let _ = envio.send(Pedido::Encerrar);
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
        let mut proxima_checagem = tempo::agora();

        loop {
            match pedidos.try_recv() {
                Ok(Pedido::Encerrar) | Err(mpsc::TryRecvError::Disconnected) => break,
                Ok(Pedido::LerAgora) => {}
                Err(mpsc::TryRecvError::Empty) => {}
            }

            if let Some(estado) = monitor.ciclo() {
                {
                    let mut atual = compartilhado.estado.lock().unwrap();
                    *atual = estado.clone();
                }
                {
                    let mut serie = compartilhado.serie.lock().unwrap();
                    *serie = monitor.historico().serie(&estado.key).to_vec();
                }

                let _ = app.emit("kontro://estado", &estado);
                atualizar_bandeja(&app, &estado, &mut ultimo_icone);
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
                orquestrador.reavaliar(&app, &estado, &cfg);
            }

            std::thread::sleep(INTERVALO_DO_CICLO);
        }

        monitor.salvar();
    });
}

fn montar_bandeja(app: &AppHandle) -> tauri::Result<()> {
    let abrir = MenuItem::with_id(app, "abrir", "Configurações", true, None::<&str>)?;
    let atualizar = MenuItem::with_id(app, "atualizar", "Atualizar agora", true, None::<&str>)?;
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
    let tamanho = tray::tamanho_do_icone();

    // o tamanho entra na assinatura porque trocar a escala da tela muda o desenho
    let assinatura = format!("{:?}|{:?}|{tamanho}", estado.mode, estado.preenchimento);
    if assinatura == *ultimo {
        return;
    }
    *ultimo = assinatura;

    let Some(icone) = tray::desenhar(estado, tamanho) else { return };
    if let Some(bandeja) = app.tray_by_id("kontro") {
        let _ = bandeja.set_icon(Some(icone));
        let dica = format!(
            "{} - {} - {}",
            estado.device_name, estado.texto_da_carga, estado.texto_da_ligacao
        );
        let _ = bandeja.set_tooltip(Some(dica));
    }
}

/// Avisa que saiu versao nova, pela notificacao do sistema.
///
/// A consulta vai numa thread propria: a rede pode demorar, e o ciclo de leitura nao
/// pode ficar parado esperando o GitHub responder.
fn avisar_versao_nova(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(novidade) = atualizacao::procurar() else { return };

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
fn serie_do_historico(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<Amostra> {
    compartilhado.serie.lock().unwrap().clone()
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

    novas.salvar();
    janelas::posicionar_sobreposicao(&app, &novas);
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

/// Procura agora, sem esperar a proxima janela. Devolve o que achou.
#[tauri::command]
fn procurar_atualizacao(compartilhado: tauri::State<Arc<Compartilhado>>) -> Option<VersaoNova> {
    let achado = atualizacao::procurar();
    *compartilhado.novidade.lock().unwrap() = achado.clone();
    achado.map(|n| VersaoNova {
        versao: n.versao,
        pagina: n.pagina,
        atual: env!("CARGO_PKG_VERSION").to_string(),
    })
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
