use std::sync::mpsc::{Receiver, Sender};
use std::thread;

/// Lightweight placeholder for the Wayland layer.
/// This spawns a thread that waits until `stop_rx` receives a message, then
/// sends a notification on `closed_tx` and exits. The full Wayland drawing
/// implementation will be added later.
pub fn open_layer(stop_rx: Receiver<()>, closed_tx: Sender<()>) {
    thread::spawn(move || {
        println!("[wayland] placeholder thread started");
        // Block until a stop request arrives
        let _ = stop_rx.recv();
        println!("[wayland] placeholder thread stopping");
        let _ = closed_tx.send(());
    });
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
