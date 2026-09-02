use crate::app::App;
use winit::event_loop::EventLoop;

pub mod emulator;
pub mod app;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(App::new()).unwrap();
}
