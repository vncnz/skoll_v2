use signal_hook::{consts::{SIGHUP, SIGINT, SIGPIPE, SIGTERM}, iterator::Signals, low_level::signal_name};

use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState, shell::wlr_layer::LayerShell, shm::Shm
};
use wayland_client::{Connection, EventQueue, globals::{GlobalList, registry_queue_init}, protocol::wl_compositor};

use std::{sync::mpsc::{self, Receiver, Sender}, time::Duration};

use std::panic;

use crate::{
    commands::start_command_listener, data::RatatoskrSocket, time_of_scope::TimeOfScope, utils::log_to_file
};

mod data;
mod config;
mod time_of_scope;
mod skoll_layer;
mod commands;
mod niri;
mod utils;
mod launcher_ui;
mod mode_calculator;
mod mode_search;
mod launcher_data;
mod module_ratatoskr;

use config::Config;
// use chrono;

use crate::skoll_layer::SkollLayer;
// use crate::battery::start_battery_listener;

use clap::{crate_name, crate_version, Parser};

#[derive(Debug, Parser)]
#[command(disable_version_flag = true, about = "Another launcher for Wayland", long_about = None)]
struct Args {
    /* /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>, */

    //#[arg(short, long, default_value_t = 1)]
    //count: u8,

    #[arg(short = 'V', long, help = "Print version")]
    version: bool,
}
/*
fn choose_output (app: &SkollLayer) -> std::option::Option<WlOutput>{
    let mut chosen_output = None;
    for output in app.output_state.outputs() {
        if let Some(info) = app.output_state.info(&output) {
            // eprintln!("Display info {:?}", info);
            if let Some(name) = info.name {
                log_to_file(format!("Found display {name}"));
                if name.starts_with("eDP") {
                    chosen_output = Some(output.clone());
                    // dbg_println!("Found internal display");
                    log_to_file(format!("{name} is an embedded display!"));
                }
            }
        }
        if chosen_output.is_none() {
            chosen_output = Some(output.clone());
        }
    }
    chosen_output
}
*/

fn main() {

    let args = Args::parse();

    if args.version {
        println!("{} {}", crate_name!(), crate_version!());
        std::process::exit(0);
    }

    panic::set_hook(Box::new(|info| {
        eprintln!("PANIC");
        eprintln!("{info}");
        let bt = std::backtrace::Backtrace::capture();
        eprintln!("{bt}");

        log_to_file("PANIC".to_string());
        log_to_file(format!("{info}"));
        log_to_file(format!("{bt}"));
    }));

    let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP, SIGPIPE]).unwrap();
    std::thread::spawn(move || {
        for sig in signals.forever() {
            let name = signal_name(sig).unwrap_or("UNKNOWN");
            eprintln!("Received signal {sig} ({name}) from the system");
            log_to_file(format!("Received signal {sig} ({name}) from the system"));

            match sig {
                SIGPIPE | SIGHUP => {
                    // log only
                }

                SIGINT | SIGTERM => {
                    eprintln!("Graceful shutdown requested");
                    std::process::exit(0);
                }

                /* SIGABRT | SIGSEGV => {
                    eprintln!("Fatal signal {}, aborting", sig);
                    std::process::abort(); // preserve core dump
                } */

                _ => {} // Dummy case
            }
        }
    });

    env_logger::init();

    log_to_file(format!("{} {} started", crate_name!(), crate_version!()));

    let config = Config::load_from_file("~/.config/skoll/config.json");
    log_to_file(format!("Loaded configuration: {:?}", config));

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue): (GlobalList, EventQueue<SkollLayer>) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).unwrap();
    let layer_shell = LayerShell::bind(&globals, &qh).unwrap();

    let mut app = SkollLayer::new(
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        SeatState::new(&globals, &qh),
        Shm::bind(&globals, &qh).unwrap(),
        config.clone()
    );

    event_queue.roundtrip(&mut app).unwrap();
    // event_queue.dispatch_pending(&mut app).unwrap();
    // let chosen_output = choose_output(&globals, &qh);
    // let output_state = OutputState::new(&globals, &qh);
    // let mut outputs = output_state.outputs();
    // let chosen_output = outputs.next();
    let raw_compositor: wl_compositor::WlCompositor =
    globals.bind::<wl_compositor::WlCompositor, _, _>(&qh, 1..=4, ())
    .expect("failed to bind wl_compositor for region creation");

    // The launcher surface is created on-demand when the user requests it.
    // This keeps memory usage low while idle and lets us select the output each time.

    let (tx, rx_cmds): (Sender<String>, Receiver<String>) = mpsc::channel();
    let _ = start_command_listener(tx, "/tmp/skoll_cmds");

    let mut sock = RatatoskrSocket::new("/tmp/ratatoskr.sock");

    /* let (_tx_battery, rx_battery): (Sender<BatteryStats>, Receiver<BatteryStats>) = mpsc::channel();
    thread::spawn(|| {
        // start_battery_listener(tx_battery); // TODO: enable or remove?
        dbg_println!("{}", "Battery listener OK".green().to_string());
    }); */

    // let (demo_tx, demo_rx) = mpsc::channel::<(String, String)>();


    /* #[cfg(debug_assertions)]
    if true {
        thread::spawn(move || {
            let actions: Vec<(&str, &str, Duration)> = vec![
                ("open", "", Duration::from_secs(1))
                // ("warning-ram", "0.9", Duration::from_secs(1)),
                //("warning-load", "0.6", Duration::from_secs(1)),
                //("warning-disk", "0.31", Duration::from_secs(1))
                /*
                ("battery", "false", Duration::from_secs(5)),
                ("battery", "true", Duration::from_secs(2)),
                ("wob", "0.35", Duration::from_secs(1)),
                ("wob", "0.45", Duration::from_millis(200)),
                ("wob", "0.55", Duration::from_millis(300)),
                ("wob", "0.65", Duration::from_millis(200)),
                ("timer", "3s", Duration::from_secs(3)),
                ("timer", "off", Duration::from_secs(6)) */
            ];

            for (kind, value, delay) in actions {
                std::thread::sleep(delay);
                let _ = demo_tx.send((kind.to_string(), value.to_string()));
            }
        });
    } */


    loop {
        let _ = event_queue.dispatch_pending(&mut app);
        sock.poll_messages();

        // Prova a leggere nuovi eventi — non blocca
        match event_queue.prepare_read() {
            Some(guard) => {
                if let Err(_) = guard.read() {
                    // Silenzia WouldBlock (nessun evento da leggere)
                    /* if let Some(raw_err) = e.raw_os_error() {
                        if raw_err != 11 {
                            eprintln!("Wayland read() error: {:?}", e);
                        }
                    } */
                }
            }
            _ => {
                // Se non pronto a leggere, prova solo a flushare
                let _ = conn.flush();
            }
        }

        // Dispatch wayland events
        let _ = event_queue.dispatch_pending(&mut app);

        // Dispatch demo events
        /* if let Ok((kind, value)) = demo_rx.try_recv() {
            match (kind.as_str(), value.as_str()) {
                ("toggle", _) => {
                    if app.layer_visible {
                        app.close_layer();
                    } else {
                        app.open_layer(&qh, &compositor, &layer_shell, &raw_compositor);
                    }
                },
                ("open", _) => {
                    app.open_layer(&qh, &compositor, &layer_shell, &raw_compositor);
                },
                ("close", _) => {
                    app.close_layer();
                },
                _ => {
                    eprintln!("Unknown demo command: {} {}", kind, value);
                }
            }
        } */

        if let Ok(cmd) = rx_cmds.try_recv() {
            if let Some(layer_cmd) = crate::skoll_layer::LayerCommand::from_input(&cmd) {
                match layer_cmd {
                    crate::skoll_layer::LayerCommand::Toggle => {
                        if app.layer_visible {
                            app.close_layer();
                        } else {
                            let _time = TimeOfScope::new("open_layer");
                            app.open_layer(&qh, &compositor, &layer_shell, &raw_compositor);
                        }
                    },
                    crate::skoll_layer::LayerCommand::Open => {
                        let _time = TimeOfScope::new("open_layer");
                        app.open_layer(&qh, &compositor, &layer_shell, &raw_compositor);
                    },
                    crate::skoll_layer::LayerCommand::Close => {
                        app.close_layer();
                    }
                }
            } else {
                match &*cmd {
                    _ => {
                        println!("cmd to be parsed: {}", cmd);
                        let parts: Vec<&str> = cmd.split(" ").collect();
                        match parts.as_slice() {
                            ["timer", _value_str] => {}
                            
                            [kind, value_str] => {
                                match value_str.parse::<f64>() {
                                    Ok(value) => { app.show_value(value, Some(*kind)); },
                                    Err(_) => { eprintln!("Invalid number: {}", value_str); }
                                }
                            }

                            [value_str] => {
                                match value_str.parse::<f64>() {
                                    Ok(value) => { app.show_value(value, None); },
                                    Err(_) => { eprintln!("Invalid number: {}", value_str); }
                                }
                            }

                            _ => {
                                eprintln!("Unknown command");
                            }
                        };
                    }
                };
            }
            // app.request_redraw();
        }
        
        if let Ok(data) = sock.rx.try_recv() {
            // log_to_file(format!("Received: {} {:?}", data.resource, data));
            // recv.push(data.resource.chars().nth(0).unwrap());
            app.process_partial_msg(data);
        }

        app.check_redraw_timeout();
        app.maybe_redraw(&qh);
        conn.flush().unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
}
