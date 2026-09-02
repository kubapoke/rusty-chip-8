use super::app::{DISPLAY_WIDTH, DISPLAY_HEIGHT};
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

fn map_coordinates(
    source_dimensions: (u32, u32),
    dest_dimensions: (u32, u32),
    dest_coords: (u32, u32)
) -> (u32, u32) {
    let source_x = (dest_coords.0 * source_dimensions.0) / dest_dimensions.0;
    let source_y = (dest_coords.1 * source_dimensions.1) / dest_dimensions.1;
    (source_x, source_y)
}

fn determine_pixel_color(
    value: u32
) -> u32 {
    match value {
        0 => ARRAY_FALSE_COLOR,
        _ => ARRAY_TRUE_COLOR,
    }
}

pub fn fill_from_display(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle + AsRef<dyn Window>>,
    display: &[u32; 16]
) {
    let surface_size = surface.window().as_ref().surface_size();
    resize(surface, surface_size);

    let mut buffer = surface.buffer_mut().expect("Failed to get the softbuffer buffer");

    let mut counter: u32 = 0;
    let buffer_dims = (buffer.width().get(), buffer.height().get());
    let display_dims = DISPLAY_DIMS;

    for pixel in buffer.iter_mut() {
        let buffer_coords = (counter % buffer_dims.0, counter / buffer_dims.0);
        let coords = map_coordinates(display_dims, buffer_dims, buffer_coords);
        *pixel = determine_pixel_color(*display.get(coords.1 as usize).expect("Failed to get display value") & (1 << (coords.0)));

        counter += 1;
    }

    buffer.present().expect("Failed to present the softbuffer buffer");
}

const ARRAY_FALSE_COLOR: u32 = 0x0000_0000;
const ARRAY_TRUE_COLOR: u32 = 0xffff_ffff;
const DISPLAY_DIMS: (u32, u32) = (DISPLAY_WIDTH, DISPLAY_HEIGHT);
