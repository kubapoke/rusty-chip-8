use std::num::NonZeroU32;
use softbuffer::{Buffer, Surface};
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
    source_width: u16,
    source_height: u16,
    dest_width: u16,
    dest_height: u16,
    source_x: u16,
    source_y: u16,
) -> (u16, u16) {
    todo!()
}

fn determine_pixel_color(
    value: bool
) -> u32 {
    match value {
        true => ARRAY_TRUE_COLOR,
        false => ARRAY_FALSE_COLOR,
    }
}

pub fn fill_from_bool_vec(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle + AsRef<dyn Window>>,
    vec: &Vec<Vec<bool>>
) {
    let surface_size = surface.window().as_ref().surface_size();
    resize(surface, surface_size);

    let mut buffer = surface.buffer_mut().expect("Failed to get the softbuffer buffer");

    for pixel in buffer.iter_mut() {
        todo!()
    }

    buffer.present().expect("Failed to present the softbuffer buffer");
}

const ARRAY_FALSE_COLOR: u32 = 0x0000_0000;
const ARRAY_TRUE_COLOR: u32 = 0xffff_ffff;
