use std::num::NonZeroU32;
use softbuffer::Surface;
use winit::dpi;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub fn resize(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle>,
    surface_size: dpi::PhysicalSize<u32>
) {
    let (Some(width), Some(height)) =
        (NonZeroU32::new(surface_size.width), NonZeroU32::new(surface_size.height))
    else {
        return;
    };

    surface.resize(width, height).expect("Failed to resize the softbuffer surface")
}

pub fn fill(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle + AsRef<dyn Window>>,
    color: u32
) {
    let surface_size = surface.window().as_ref().surface_size();
    resize(surface, surface_size);

    let mut buffer = surface.buffer_mut().expect("Failed to get the softbuffer buffer");
    buffer.fill(color);
    buffer.present().expect("Failed to present the softbuffer buffer");
}
