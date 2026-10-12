use std::io::{Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    delegate_noop,
    globals::registry_queue_init,
    protocol::{wl_buffer, wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_shm_pool, wl_surface},
    Connection, QueueHandle,
};
use xkbcommon::xkb::keysyms;

const LAYER_WIDTH: u32 = 560;
const LAYER_HEIGHT: u32 = 220;

pub fn open_layer(stop_rx: Receiver<()>, closed_tx: Sender<()>) {
    thread::spawn(move || {
        println!("[wayland] opening layer shell");

        if let Err(err) = run_layer(stop_rx, closed_tx) {
            eprintln!("[wayland] layer failed: {err}");
        }
    });
}

fn run_layer(stop_rx: Receiver<()>, closed_tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Top,
        Some("skoll_v2"),
        None,
    );

    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_exclusive_zone(0);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(LAYER_WIDTH, LAYER_HEIGHT);
    layer.commit();

    let mut state = LayerWindow {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        width: LAYER_WIDTH,
        height: LAYER_HEIGHT,
        layer,
        pool: None,
        buffer: None,
        keyboard: None,
        keyboard_focus: false,
        pointer: None,
    };

    event_queue.roundtrip(&mut state)?;

    loop {
        match stop_rx.try_recv() {
            Ok(()) => {
                println!("[wayland] stop request received");
                state.exit = true;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.exit = true;
            }
        }

        if state.exit {
            break;
        }

        event_queue.blocking_dispatch(&mut state)?;
    }

    println!("[wayland] closing layer surface");
    state.layer.wl_surface().destroy();
    let _ = closed_tx.send(());
    Ok(())
}

struct LayerWindow {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    exit: bool,
    first_configure: bool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    pool: Option<wl_shm_pool::WlShmPool>,
    buffer: Option<wl_buffer::WlBuffer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,
}

impl CompositorHandler for LayerWindow {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }
}

impl OutputHandler for LayerWindow {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for LayerWindow {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0.max(1);
        self.height = configure.new_size.1.max(1);

        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for LayerWindow {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for LayerWindow {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[u32],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if self.keyboard_focus && event.keysym == keysyms::KEY_Escape {
            println!("[wayland] ESC pressed — closing launcher layer");
            self.exit = true;
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
    ) {
    }
}

impl PointerHandler for LayerWindow {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }

            match event.kind {
                PointerEventKind::Enter { .. } => {}
                PointerEventKind::Leave { .. } => {}
                PointerEventKind::Motion { .. } => {}
                PointerEventKind::Press { .. } => {}
                PointerEventKind::Release { .. } => {}
                PointerEventKind::Axis { .. } => {}
            }
        }
    }
}

impl ShmHandler for LayerWindow {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerWindow {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width as usize;
        let height = self.height as usize;
        let stride = width as i32 * 4;
        let total = width * height * 4;

        let mut pixels = vec![0u8; total];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = 0x0b;
            chunk[1] = 0x13;
            chunk[2] = 0x22;
            chunk[3] = 0xff;
        }

        draw_border(&mut pixels, width, height);
        draw_text(&mut pixels, width, height, "skoll_v2");

        if self.pool.is_none() || self.buffer.is_none() {
            let mut file = tempfile::tempfile().expect("failed to create shm file");
            file.set_len(total as u64).expect("failed to resize shm file");
            file.seek(SeekFrom::Start(0)).expect("failed to rewind shm file");
            file.write_all(&pixels).expect("failed to write shm pixels");

            let raw_fd = file.as_raw_fd();
            let pool = self.shm.wl_shm().create_pool(raw_fd, total as i32, qh, ());
            std::mem::forget(file);

            let buffer = pool.create_buffer(
                0,
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Argb8888,
                qh,
                (),
            );
            self.pool = Some(pool);
            self.buffer = Some(buffer);
        }

        let buffer = self.buffer.as_ref().expect("layer buffer not created");
        self.layer.wl_surface().attach(Some(buffer), 0, 0);
        self.layer.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.layer.wl_surface().frame(qh, self.layer.wl_surface().clone());
        self.layer.commit();
    }
}

fn draw_border(canvas: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let on_border = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;
            if on_border {
                canvas[idx] = 0x4a;
                canvas[idx + 1] = 0x9f;
                canvas[idx + 2] = 0xff;
                canvas[idx + 3] = 0xff;
            }
        }
    }
}

fn draw_text(canvas: &mut [u8], width: usize, height: usize, text: &str) {
    let mut x = 34usize;
    let y = 76usize;

    for ch in text.chars() {
        draw_char(canvas, width, height, x, y, ch);
        x += 12;
        if x > width - 24 {
            break;
        }
    }
}

fn draw_char(canvas: &mut [u8], width: usize, height: usize, x: usize, y: usize, ch: char) {
    let pixels = match ch {
        's' => [0x3c, 0x42, 0x40, 0x3c, 0x02, 0x42, 0x3c],
        'k' => [0x41, 0x42, 0x44, 0x78, 0x44, 0x42, 0x41],
        'o' => [0x3c, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3c],
        'l' => [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x3c],
        'v' => [0x41, 0x42, 0x42, 0x24, 0x18, 0x08, 0x00],
        '2' => [0x3c, 0x42, 0x02, 0x04, 0x08, 0x10, 0x3e],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f],
        'e' => [0x3e, 0x48, 0x48, 0x48, 0x3e, 0x40, 0x7f],
        'c' => [0x3e, 0x41, 0x41, 0x41, 0x41, 0x41, 0x3e],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    };

    for row in 0..7 {
        let bits = pixels[row];
        for col in 0..5 {
            let bit = 1u8 << (4 - col);
            if bits & bit != 0 {
                let px = x + col;
                let py = y + row;
                if px < width && py < height {
                    let idx = ((py * width) + px) * 4;
                    canvas[idx] = 0xff;
                    canvas[idx + 1] = 0xff;
                    canvas[idx + 2] = 0xf0;
                    canvas[idx + 3] = 0xff;
                }
            }
        }
    }
}

delegate_noop!(LayerWindow: ignore wl_shm_pool::WlShmPool);
delegate_noop!(LayerWindow: ignore wl_buffer::WlBuffer);

delegate_compositor!(LayerWindow);
delegate_output!(LayerWindow);
delegate_shm!(LayerWindow);
delegate_seat!(LayerWindow);
delegate_keyboard!(LayerWindow);
delegate_pointer!(LayerWindow);
delegate_layer!(LayerWindow);
delegate_registry!(LayerWindow);

impl ProvidesRegistryState for LayerWindow {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_keysym_is_detected() {
        assert_eq!(is_escape_key(keysyms::KEY_Escape), true);
        assert_eq!(is_escape_key(0xFF1B), true);
        assert_eq!(is_escape_key(keysyms::KEY_Return), false);
    }

    fn is_escape_key(keysym: u32) -> bool {
        keysym == keysyms::KEY_Escape || keysym == 0xFF1B
    }
}
