use crate::emulator::emulator::*;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};
use log::info;
use crate::app::utils::{fill_from_display, resize};

#[derive(Debug)]
pub struct App {
    emulator: Emulator,
    surface: Option<Surface<OwnedDisplayHandle, Box<dyn Window>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            emulator: Emulator::new(),
            surface: None,
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = event_loop.create_window(window_attributes).expect("Failed creating window");

        let context =
            Context::new(event_loop.owned_display_handle()).expect("failed creating context");
        let surface = Surface::new(&context, window).expect("Failed creating surface");
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        info!("{event:?}");
        match event {
            WindowEvent::CloseRequested => {
                info!("Close was requested; stopping");
                event_loop.exit();
            },
            WindowEvent::SurfaceResized(surface_size) => {
                let surface = self.surface.as_mut().expect("Resize event without a surface");
                resize(surface, surface_size);
                surface.window().request_redraw();
            },
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().expect("Failed to get the softbuffer buffer");

                let arr = [0b1111_1111_1111_1111_1111_1111_1111_1111,0b0000_0000_0000_0000_0000_0000_0000_0000,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,
                    0b1010_1010_1010_1010_1010_1010_1010_1010,0b0101_0101_0101_0101_0101_0101_0101_0101,];

                fill_from_display(surface, &arr);
            }
            _ => ()
        }
    }
}
