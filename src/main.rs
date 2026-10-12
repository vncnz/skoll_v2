mod wayland_layer;

use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const FIFO_PATH: &str = "/tmp/skoll_cmd";

fn ensure_fifo(path: &str) -> std::io::Result<()> {
    if Path::new(path).exists() {
        // If it's not a fifo, remove and recreate
        let meta = fs::metadata(path)?;
        if !meta.file_type().is_fifo() {
            fs::remove_file(path)?;
        }
    }

    if !Path::new(path).exists() {
        // create fifo with 0o600
        nix::unistd::mkfifo(path, nix::sys::stat::Mode::from_bits_truncate(0o600))?;
    }
    Ok(())
}

fn fifo_reader_thread(tx: Sender<String>) -> std::io::Result<()> {
    loop {
        // Open FIFO for reading (blocking until writer opens)
        let file = std::fs::OpenOptions::new().read(true).open(FIFO_PATH)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(cmd) => {
                    let cmd = cmd.trim().to_string();
                    if !cmd.is_empty() {
                        if tx.send(cmd).is_err() {
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading fifo: {}", e);
                    break;
                }
            }
        }
    }
}

use std::sync::{Arc, Mutex};

fn handle_commands(rx: Receiver<String>) {
    // Shared optional sender to request the active Wayland layer to stop
    let active_stop: Arc<Mutex<Option<Sender<()>>>> = Arc::new(Mutex::new(None));

    for cmd in rx.iter() {
        match cmd.as_str() {
            "open" => {
                println!("Command: open -> opening fullscreen layer");
                // Create per-open control channels
                let (stop_tx, stop_rx) = channel::<()>();
                let (closed_tx, closed_rx) = channel::<()>();

                // Record the sender so other commands can request stop
                {
                    let mut guard = active_stop.lock().unwrap();
                    *guard = Some(stop_tx.clone());
                }

                // Spawn the wayland layer (it runs in its own thread)
                wayland_layer::open_layer(stop_rx, closed_tx);

                // Spawn a watcher that waits for the closed signal and clears
                // the active sender when done.
                let active_stop_clone = active_stop.clone();
                thread::spawn(move || {
                    let _ = closed_rx.recv();
                    println!("Wayland layer closed");
                    let mut guard = active_stop_clone.lock().unwrap();
                    *guard = None;
                });
            }
            "close" => {
                println!("Command: close -> sending stop signal");
                let mut guard = active_stop.lock().unwrap();
                if let Some(tx) = guard.take() {
                    let _ = tx.send(());
                } else {
                    println!("No active layer to close");
                }
            }
            other => {
                println!("Unknown command: {}", other);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("skoll_v2 daemon starting");

    ensure_fifo(FIFO_PATH)?;

    let (tx, rx) = channel();

    // Spawn thread to read fifo commands
    let reader_tx = tx.clone();
    thread::spawn(move || {
        if let Err(e) = fifo_reader_thread(reader_tx) {
            eprintln!("fifo reader thread exited: {}", e);
        }
    });

    // Main thread handles commands
    handle_commands(rx);

    Ok(())
}
