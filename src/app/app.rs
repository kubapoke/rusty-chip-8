use crate::app::utils::{fill_from_display, resize};
use crate::emulator::Emulator;
use log::info;
use softbuffer::{Context, Surface};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Debug)]
pub struct App {
    display: Arc<Mutex<[u64; 32]>>,
    handle: JoinHandle<()>,
    surface: Option<Surface<OwnedDisplayHandle, Box<dyn Window>>>,
}

impl App {
    pub fn new() -> Self {
        let mut emulator = Emulator::new();
        let display = emulator.get_display();

        let handle = thread::spawn(move || {
            emulator.load_program(Path::new("./programs/IBM Logo.ch8")); // TODO: Add proper program loading
            emulator.begin_execution();
        });

        Self {
            display,
            handle,
            surface: None,
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
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(surface_size) => {
                let surface = self.surface.as_mut().expect("Resize event without a surface");
                resize(surface, surface_size);
                surface.window().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().expect("Failed to get the softbuffer buffer");
                fill_from_display(surface, &self.display);

                surface.window().request_redraw();
            }
            _ => ()
        }
    }
}

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
const SIZE_MULTIPLIER: u32 = 16;
