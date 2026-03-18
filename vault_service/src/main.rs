use std::default::Default;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, OnceLock};

use vault::config::Config;

#[cfg(target_os = "windows")]
use windows::Win32::System::Services::{
    SERVICE_ACCEPT_STOP, SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS,
    SERVICE_STOP_PENDING, SERVICE_WIN32_OWN_PROCESS, SetServiceStatus,
};
static TX: std::sync::OnceLock<Arc<Sender<AppMessage>>> = OnceLock::new();

#[cfg(target_os = "windows")]
pub unsafe extern "system" fn service_handler(
    a: u32,
    b: u32,
    c: *mut core::ffi::c_void,
    d: *mut core::ffi::c_void,
) -> u32 {
    use windows::Win32::System::Services::SERVICE_CONTROL_STOP;
    if a == SERVICE_CONTROL_STOP {
        if let Some(mut tx) = TX.get() {
            let _ = tx.send(AppMessage::RequestShutdown);
        }
    }
    0
}

fn run_vault_ui() {}

pub enum ServiceStatusChangeMessage {
    Starting,
    Running,
    Stopping,
    Stopped,
}
#[cfg(windows)]
// a function that handles requests to tell the windows scm api about the current state of the application
fn run_svc_status_handler(reciever: std::sync::mpsc::Receiver<ServiceStatusChangeMessage>) {
    use windows::Win32::System::Services::RegisterServiceCtrlHandlerExW;
    use windows::core::w;
    let status_handle = unsafe {
        RegisterServiceCtrlHandlerExW(w!("VaultService"), Some(service_handler), None).unwrap()
    };
    while let Ok(message) = reciever.recv() {
        match message {
            ServiceStatusChangeMessage::Starting => {
                let status = SERVICE_STATUS {
                    dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                    dwCurrentState: SERVICE_START_PENDING,
                    dwControlsAccepted: SERVICE_ACCEPT_STOP,
                    ..Default::default()
                };
                unsafe { SetServiceStatus(status_handle, &status).unwrap() };
            }
            ServiceStatusChangeMessage::Running => {
                let status = SERVICE_STATUS {
                    dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                    dwCurrentState: SERVICE_RUNNING,
                    dwControlsAccepted: SERVICE_ACCEPT_STOP,
                    ..Default::default()
                };
                unsafe { SetServiceStatus(status_handle, &status).unwrap() };
            }
            ServiceStatusChangeMessage::Stopping => {
                let status = SERVICE_STATUS {
                    dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                    dwCurrentState: SERVICE_STOP_PENDING,
                    dwControlsAccepted: SERVICE_ACCEPT_STOP,
                    ..Default::default()
                };
                unsafe { SetServiceStatus(status_handle, &status).unwrap() };
            }
            ServiceStatusChangeMessage::Stopped => {
                let status = SERVICE_STATUS {
                    dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                    dwCurrentState: SERVICE_RUNNING,
                    dwControlsAccepted: SERVICE_ACCEPT_STOP,
                    ..Default::default()
                };
                unsafe { SetServiceStatus(status_handle, &status).unwrap() };
                break;
            }
        }
    }
}

pub enum AppMessage {
    RequestShutdown,
}

/// the true 'entry point' runs the app logic
fn run_vault(
    config: Config,
    service_status_sender: Arc<Sender<ServiceStatusChangeMessage>>,
    app_message_reciever: std::sync::mpsc::Receiver<AppMessage>,
    ui_message_sender: std::sync::mpsc::Sender<UiMessage>,
) {
    println!("running startup tasks");
    service_status_sender
        .send(ServiceStatusChangeMessage::Running)
        .ok();

    loop {
        print!("app loop");
        if let Ok(message) = app_message_reciever.recv() {
            // shut down
            match message {
                AppMessage::RequestShutdown => {
                    println!("shutdown requested");
                    // attempt shutdown, tell OS that we are shuttting down
                    service_status_sender
                        .send(ServiceStatusChangeMessage::Stopping)
                        .ok();
                    ui_message_sender.send(UiMessage::Shutdown).ok();
                    // tell any depending services to shut down as well
                    break;
                }
            }
        }
    }

    // perform cleanup, shudown, then tell windows that we are done

    service_status_sender
        .send(ServiceStatusChangeMessage::Stopped)
        .ok();
}
pub enum UiMessage {
    Shutdown,
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
extern "system" fn service_main(dw_arg_c: u32, _lpsz_arg_v: *mut windows::core::PWSTR) {
    // send the sender to the handler function instead of global state
    let config =
        vault::config::Config::try_load_default_merged().unwrap_or(Config::try_default().unwrap());
    let (service_status_sender, service_status_reciever) =
        std::sync::mpsc::channel::<ServiceStatusChangeMessage>();
    // wrap the sender in an arc so we can pass it to multiple places
    let service_status_sender = std::sync::Arc::new(service_status_sender);

    // tell scm that we are starting
    service_status_sender
        .send(ServiceStatusChangeMessage::Starting)
        .ok();
    // service_sender to app
    let (app_msg_sender, app_msg_reciever) = std::sync::mpsc::channel::<AppMessage>();
    let app_msg_sender = std::sync::Arc::new(app_msg_sender);
    TX.set(app_msg_sender.clone()).ok();

    // app thread -> service status thread tells OS whats happening
    // ui thread -> app thread ui sends state change messages to app
    // ui thread requests state changes in the app

    // run the service handler thread. accepts messages on the sender

    std::thread::spawn(move || {
        run_svc_status_handler(service_status_reciever);
    });
    let c1 = config.clone();

    // the app thread
    let (app_to_ui_thread_sender, ui_app_thread_reciever) = channel::<UiMessage>();
    std::thread::spawn(move || {
        run_vault(
            c1,
            service_status_sender,
            app_msg_reciever,
            app_to_ui_thread_sender,
        );
    });

    let app_msg_sender1 = app_msg_sender.clone();
    // run the web_ui thread
    std::thread::spawn(move || {
        let app_message_sender = app_msg_sender1;
        let r = vault_service::server::rocket::tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(val)
            .thread_name("Vault-thread-fn")
            .enable_all()
            .build();
        if r.is_err() {
            // failed to build runtime
            app_message_sender.send(AppMessage::RequestShutdown).ok();
        };
        let rt = r.unwrap();
        rt.block_on(async {
            // TODO tell the 'app' thread that the server is starting
            let rocket = vault_service::server::build_rocket(config).ignite().await;
            if rocket.is_err() {
                app_message_sender.send(AppMessage::RequestShutdown).ok();
            }
            let rocket = rocket.unwrap();
            let shutdown = rocket.shutdown();
            vault_service::server::rocket::tokio::spawn(rocket.launch());
            // listen for shutdown signals
            let s1 = shutdown.clone();
            let s2 = shutdown.clone();
            loop {
                let message = ui_app_thread_reciever.try_recv();
                match message {
                    Ok(UiMessage::Shutdown) => {
                        s1.notify();
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        s2.notify();
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        vault_service::server::rocket::tokio::time::sleep(
                            vault_service::server::rocket::tokio::time::Duration::from_millis(200),
                        )
                        .await;
                        continue;
                    }
                }
            }
        })
    });
    // this only runs once for now. needed to be able to see 'shutdown' from main loop

    // when these threads are dropped and they exit, the service should also exit
    // std::process::exit(0);
}

pub fn app_main() {
    println!("setting up????");
    let config =
        vault::config::Config::try_load_default_merged().unwrap_or(Config::try_default().unwrap());
    let (service_status_sender, service_status_reciever) =
        std::sync::mpsc::channel::<ServiceStatusChangeMessage>();
    // wrap the sender in an arc so we can pass it to multiple places
    let service_status_sender = std::sync::Arc::new(service_status_sender);

    // service_sender to app
    let (app_msg_sender, app_msg_reciever) = std::sync::mpsc::channel::<AppMessage>();
    let app_msg_sender = std::sync::Arc::new(app_msg_sender);
    // TX.set(app_msg_sender.clone()).ok();

    // app thread -> service status thread tells OS whats happening
    // ui thread -> app thread ui sends state change messages to app
    // ui thread requests state changes in the app

    // run the service handler thread. accepts messages on the sender

    let c1 = config.clone();

    // the app thread
    let (app_to_ui_thread_sender, ui_app_thread_reciever) = channel::<UiMessage>();
    let app = std::thread::spawn(move || {
        run_vault(
            c1,
            service_status_sender,
            app_msg_reciever,
            app_to_ui_thread_sender,
        );
    });

    let app_msg_sender1 = app_msg_sender.clone();
    println!("attempting to spawn tokio thread");
    // run the web_ui thread
    let ui = std::thread::spawn(move || {
        println!("Attempting to start rocket server");
        let app_message_sender = app_msg_sender1;
        let r = vault_service::server::rocket::tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(val)
            .thread_name("Vault-thread-fn")
            .enable_all()
            .build();
        if r.is_err() {
            println!("failed to build runtime");
            // failed to build runtime
            app_message_sender.send(AppMessage::RequestShutdown).ok();
        };
        let rt = r.unwrap();
        rt.block_on(async {
            println!("async runtime started. attempting to start rocket");
            // TODO tell the 'app' thread that the server is starting
            let rocket = vault_service::server::build_rocket(config).ignite().await;

            let rocket = match rocket {
                Ok(r) => r,
                Err(e) => {
                    println!("Failed to ignite rocket: {e}... requesting shutdown");
                    app_message_sender.send(AppMessage::RequestShutdown).ok();
                    return;
                }
            };

            let shutdown = rocket.shutdown();
            println!("launching");
            let rocket_launch_result = vault_service::server::rocket::tokio::spawn(rocket.launch());
            // listen for shutdown signals
            println!("waiting for shutdown");
            let s1 = shutdown.clone();
            let s2 = shutdown.clone();
            loop {
                let message = ui_app_thread_reciever.try_recv();
                match message {
                    Ok(UiMessage::Shutdown) => {
                        println!("app thread asked for shutdown");
                        s1.notify();
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        println!("app thread disconnected");
                        s2.notify();
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        vault_service::server::rocket::tokio::time::sleep(
                            vault_service::server::rocket::tokio::time::Duration::from_millis(200),
                        )
                        .await;
                        continue;
                    }
                }
            }
            let r = rocket_launch_result.await;
            let _ = dbg!(r);
        });
    });
    app.join().ok();
    ui.join().ok();
}

/// the application entry point stub. the real service start function later
fn main() {
    // on windows we need to set up this app as a service so it can run in the background
    let mut args = std::env::args();
    let _ = args.next();
    let opt = args.next();
    if opt.is_none() {
        // run as service
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Services::{
                SERVICE_TABLE_ENTRYW, StartServiceCtrlDispatcherW,
            };
            use windows::core::PWSTR;
            let service_table = [SERVICE_TABLE_ENTRYW {
                lpServiceName: PWSTR(windows::core::w!("VaultService").as_ptr() as *mut _),
                lpServiceProc: Some(service_main),
            }];
            unsafe { StartServiceCtrlDispatcherW(service_table.as_ptr()).unwrap() };
        }
        #[cfg(all(not(target_os = "windows"), target_os = "linux"))]
        {
            // listen for signals and forward them to the seperate thread
            app_main();
        }
    } else {
        // dont run as service
        app_main();
    }
}
