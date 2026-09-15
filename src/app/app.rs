use crate::app::utils::{fill_from_display, get_key_event_message, resize, update_display_from_interface};
use crate::emulator::{Emulator, EmulatorInterface};
use log::info;
use softbuffer::{Context, Surface};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Debug)]
pub struct App {
    surface: Option<Surface<OwnedDisplayHandle, Box<dyn Window>>>,
    display: [u64; 32],
    emulator_interface: EmulatorInterface,
}

impl App {
    pub fn new() -> Self {
        let mut emulator = Emulator::new();
        let emulator_interface = emulator.get_interface();

        let handle = thread::spawn(move || {
            emulator.load_program(Path::new("./programs/test_opcode.ch8")); // TODO: Add proper program loading
            emulator.begin_execution();
        });

        Self {
            surface: None,
            display: [0; 32],
            emulator_interface,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_surface_size(LogicalSize::new(DISPLAY_WIDTH * SIZE_MULTIPLIER, DISPLAY_HEIGHT * SIZE_MULTIPLIER))
            .with_resizable(false)
            .with_title("CHIP-8");
        let window = event_loop.create_window(window_attributes).expect("Failed creating window");

        let context =
            Context::new(event_loop.owned_display_handle()).expect("Failed creating context");
        let surface = Surface::new(&context, window).expect("Failed creating surface");
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        info!("{event:?}");
        match event {
            WindowEvent::CloseRequested => {
                info!("Close was requested; stopping");
                self.emulator_interface.stop_execution();
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(surface_size) => {
                let surface = self.surface.as_mut().expect("Resize event without a surface");
                resize(surface, surface_size);
                surface.window().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().expect("Failed to get the softbuffer buffer");
                update_display_from_interface(&mut self.display, &self.emulator_interface);
                fill_from_display(surface, &self.display);

                surface.window().request_redraw();
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if let Some((key, pressed)) = get_key_event_message(event) {
                    self.emulator_interface.send_input_update(key, pressed);
                }
            }
            _ => ()
        }
    }
}

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
const SIZE_MULTIPLIER: u32 = 16;
