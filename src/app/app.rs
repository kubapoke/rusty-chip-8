use crate::emulator::emulator::*;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::window::{WindowId};

#[derive(Debug)]
pub struct App {
    emulator: Emulator
}

impl App {
    pub fn new() -> Self {
        Self {
            emulator: Emulator::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        todo!()
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        todo!()
    }
}
