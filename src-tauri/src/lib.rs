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

use crate::history::{Amostra, Sessao};
use crate::model::BatteryState;
use crate::settings::{CloseAction, Limiares, Settings};

pub struct Compartilhado {
    estado: Mutex<BatteryState>,
    todos: Mutex<Vec<BatteryState>>,
    serie: Mutex<Vec<Amostra>>,
    sessoes: Mutex<Vec<Sessao>>,
    saude: Mutex<Option<history::Saude>>,
    sobreposicao_a_mao: Mutex<Option<bool>>,
    config: Mutex<Settings>,
    novidade: Mutex<Option<atualizacao::Novidade>>,
}

enum Pedido {
    LerAgora,
    Renomear { chave: String, nome: String },
    Esquecer { chave: String },
    Encerrar(mpsc::Sender<()>),
}

const INTERVALO_DO_CICLO: Duration = Duration::from_secs(2);

pub fn executar() {
    let argumentos: Vec<String> = std::env::args().collect();
    if let Some(i) = argumentos.iter().position(|a| a == "--diagnose") {
        let destino = argumentos
            .get(i + 1)
            .cloned()
            .unwrap_or_else(|| "kontro-diagnostico.txt".to_string());

        device::iniciar_apartamento();
        match diagnostico::escrever(&destino, None) {
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
        sessoes: Mutex::new(Vec::new()),
        saude: Mutex::new(None),
        sobreposicao_a_mao: Mutex::new(None),
        todos: Mutex::new(Vec::new()),
        config: Mutex::new(config),
        novidade: Mutex::new(None),
    });

    let (envio, recebimento) = mpsc::channel::<Pedido>();
    let ao_encerrar = envio.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
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
            sessoes_do_controle,
            saude_da_bateria,
            configuracoes,
            salvar_configuracoes,
            ler_agora,
            versao_disponivel,
            procurar_atualizacao,
            mostrar_janela,
            esconder_janela,
            quantidade_de_telas,
            salvar_diagnostico,
            ajustar_altura_do_painel
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            janelas::criar_todas(&handle)?;
            montar_bandeja(&handle)?;

            atalho::aplicar(&handle, compartilhado.config.lock().unwrap().overlay_shortcut_enabled);

            let pedido_explicito = std::env::args().any(|a| a == "--show");

            if std::env::args().any(|a| a == "--painel") {
                if let Some(painel) = handle.get_webview_window(janelas::PAINEL) {
                    janelas::posicionar_painel(&handle);
                    let _ = painel.show();
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

fn iniciar_ciclo(
    app: AppHandle,
    compartilhado: Arc<Compartilhado>,
    pedidos: mpsc::Receiver<Pedido>,
) {
    std::thread::spawn(move || {
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
                {
                    let mut sessoes = compartilhado.sessoes.lock().unwrap();
                    *sessoes = monitor.historico().sessoes(&principal.key);
                }

                let _ = app.emit("kontro://estado", &principal);
                let _ = app.emit("kontro://controles", &panorama.todos);
                let limiares = compartilhado.config.lock().unwrap().limiares();
                atualizar_bandeja(&app, &principal, limiares, &mut ultimo_icone);
            }

            if tempo::agora() >= proxima_checagem {
                proxima_checagem = tempo::agora() + atualizacao::JANELA_MS;
                let liberado = compartilhado.config.lock().unwrap().auto_check_updates;
                if liberado {
                    avisar_versao_nova(&app);
                }
            }

            {
                let estado = compartilhado.estado.lock().unwrap().clone();
                let cfg = compartilhado.config.lock().unwrap().clone();
                let mao = *compartilhado.sobreposicao_a_mao.lock().unwrap();
                orquestrador.reavaliar(&app, &estado, &cfg, mao, ligados(&compartilhado));
            }

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

fn ligados(compartilhado: &Compartilhado) -> usize {
    compartilhado
        .todos
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e.mode != model::LinkMode::Offline)
        .count()
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

fn atualizar_bandeja(
    app: &AppHandle,
    estado: &BatteryState,
    limiares: Limiares,
    ultimo: &mut String,
) {
    let Some(bandeja) = app.tray_by_id("kontro") else { return };

    let dica = format!(
        "{} - {} - {}",
        estado.device_name, estado.texto_da_carga, estado.texto_da_ligacao
    );
    let _ = bandeja.set_tooltip(Some(dica));

    let tamanho = tray::tamanho_do_icone();
    let assinatura = format!(
        "{:?}|{:?}|{tamanho}|{}|{}",
        estado.mode, estado.preenchimento, limiares.critico, limiares.aviso
    );
    if assinatura == *ultimo {
        return;
    }
    *ultimo = assinatura;

    if let Some(icone) = tray::desenhar(estado, tamanho, limiares) {
        let _ = bandeja.set_icon(Some(icone));
    }
}

fn avisar_versao_nova(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let atualizacao::Consulta::Nova(novidade) = atualizacao::procurar(&app) else {
            return;
        };

        let _ = app
            .notification()
            .builder()
            .title(format!("Kontro {} disponivel", novidade.versao))
            .body("Abra as configuracoes do Kontro para instalar.")
            .show();

        let compartilhado = app.state::<Arc<Compartilhado>>();
        *compartilhado.novidade.lock().unwrap() = Some(novidade);
    });
}

#[tauri::command]
fn estado_atual(compartilhado: tauri::State<Arc<Compartilhado>>) -> BatteryState {
    compartilhado.estado.lock().unwrap().clone()
}

#[tauri::command]
fn controles(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<BatteryState> {
    compartilhado.todos.lock().unwrap().clone()
}

#[tauri::command]
fn renomear_controle(chave: String, nome: String, envio: tauri::State<mpsc::Sender<Pedido>>) {
    let _ = envio.send(Pedido::Renomear { chave, nome });
}

#[tauri::command]
fn esquecer_controle(chave: String, envio: tauri::State<mpsc::Sender<Pedido>>) {
    let _ = envio.send(Pedido::Esquecer { chave });
}

#[tauri::command]
fn serie_do_historico(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<Amostra> {
    compartilhado.serie.lock().unwrap().clone()
}

#[tauri::command]
fn sessoes_do_controle(compartilhado: tauri::State<Arc<Compartilhado>>) -> Vec<Sessao> {
    compartilhado.sessoes.lock().unwrap().clone()
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

    {
        let anterior = compartilhado.config.lock().unwrap().start_with_windows;
        if novas.start_with_windows != anterior
            && !autostart::definir(novas.start_with_windows)
        {
            novas.start_with_windows = autostart::ligado();
        }
    }

    {
        let anterior = compartilhado.config.lock().unwrap().overlay_mode;
        if novas.overlay_mode != anterior {
            *compartilhado.sobreposicao_a_mao.lock().unwrap() = None;
        }
    }

    atalho::aplicar(&app, novas.overlay_shortcut_enabled);

    novas.ajustar();
    novas.salvar();
    janelas::posicionar_sobreposicao(&app, &novas, ligados(&compartilhado));

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
    notas: Option<String>,
    atual: String,
}

#[tauri::command]
fn versao_disponivel(compartilhado: tauri::State<Arc<Compartilhado>>) -> Option<VersaoNova> {
    compartilhado.novidade.lock().unwrap().clone().map(|n| VersaoNova {
        versao: n.versao,
        notas: n.notas,
        atual: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(serde::Serialize)]
struct Busca {
    estado: &'static str,
    versao: Option<String>,
    notas: Option<String>,
    atual: String,
    motivo: Option<String>,
}

#[tauri::command]
fn procurar_atualizacao(
    app: AppHandle,
    compartilhado: tauri::State<Arc<Compartilhado>>,
) -> Busca {
    let atual = env!("CARGO_PKG_VERSION").to_string();

    match atualizacao::procurar(&app) {
        atualizacao::Consulta::Nova(n) => {
            *compartilhado.novidade.lock().unwrap() = Some(n.clone());
            Busca {
                estado: "nova",
                versao: Some(n.versao),
                notas: n.notas,
                atual,
                motivo: None,
            }
        }
        atualizacao::Consulta::EmDia => {
            *compartilhado.novidade.lock().unwrap() = None;
            Busca {
                estado: "em-dia",
                versao: None,
                notas: None,
                atual,
                motivo: None,
            }
        }
        atualizacao::Consulta::Falhou(motivo) => Busca {
            estado: "falhou",
            versao: None,
            notas: None,
            atual,
            motivo: Some(motivo),
        },
    }
}

#[tauri::command]
fn salvar_diagnostico(compartilhado: tauri::State<Arc<Compartilhado>>) -> Result<String, String> {
    let destino = paths::arquivo("diagnostico.txt");
    let estados = compartilhado.todos.lock().unwrap().clone();
    let principal = compartilhado.estado.lock().unwrap().key.clone();
    let sessoes = compartilhado.sessoes.lock().unwrap().clone();

    paths::garantir_dir();
    diagnostico::escrever(
        &destino.to_string_lossy(),
        Some(diagnostico::AoVivo {
            principal: &principal,
            estados: &estados,
            sessoes: &sessoes,
        }),
    )
    .map_err(|e| e.to_string())?;

    let _ = std::process::Command::new("explorer")
        .arg("/select,")
        .arg(&destino)
        .spawn();

    Ok(destino.to_string_lossy().to_string())
}

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

#[tauri::command]
fn ajustar_altura_do_painel(app: AppHandle, altura: f64) {
    let Some(janela) = app.get_webview_window(janelas::PAINEL) else { return };

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
