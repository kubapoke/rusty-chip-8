use super::app::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use winit::dpi;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub fn resize(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle>,
    surface_size: dpi::PhysicalSize<u32>,
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
    color: u32,
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
    dest_coords: (u32, u32),
) -> (u32, u32) {
    let source_x = (dest_coords.0 * source_dimensions.0) / dest_dimensions.0;
    let source_y = (dest_coords.1 * source_dimensions.1) / dest_dimensions.1;
    (source_x, source_y)
}

fn determine_pixel_color(
    value: u64
) -> u32 {
    match value {
        0 => ARRAY_FALSE_COLOR,
        _ => ARRAY_TRUE_COLOR,
    }
}

pub fn update_display_from_feed(display: &mut [u64; 32], feed: &Receiver<(u8, u64)>) {
    while let Ok((row, value)) = feed.try_recv() {
        display[row as usize] = value;
    }
}

pub fn fill_from_display(
    surface: &mut Surface<impl HasDisplayHandle, impl HasWindowHandle + AsRef<dyn Window>>,
    display: &[u64; 32],
) {
    let surface_size = surface.window().as_ref().surface_size();
    resize(surface, surface_size);

    let mut buffer = surface.buffer_mut().expect("Failed to get the softbuffer buffer");
    let buffer_dims = (buffer.width().get(), buffer.height().get());
    let display_dims = DISPLAY_DIMS;

    for (i, pixel) in buffer.iter_mut().enumerate() {
        let i = i as u32;
        let buffer_coords = (i % buffer_dims.0, i / buffer_dims.0);
        let coords = map_coordinates(display_dims, buffer_dims, buffer_coords);
        *pixel = determine_pixel_color(*display.get(coords.1 as usize).expect("Failed to get display value") & (1 << ((DISPLAY_WIDTH as u64 - 1) - coords.0 as u64)));
    }

    buffer.present().expect("Failed to present the softbuffer buffer");
}

pub fn get_key_event_message(event: KeyEvent) -> Option<(u8, bool)> {
    if event.repeat {
        return None
    }

    let is_pressed = match event.state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    };

    let key: u8 = match event.physical_key {
        PhysicalKey::Code(KeyCode::Digit1) => 0x1,
        PhysicalKey::Code(KeyCode::Digit2) => 0x2,
        PhysicalKey::Code(KeyCode::Digit3) => 0x3,
        PhysicalKey::Code(KeyCode::Digit4) => 0xc,
        PhysicalKey::Code(KeyCode::KeyQ) => 0x4,
        PhysicalKey::Code(KeyCode::KeyW) => 0x5,
        PhysicalKey::Code(KeyCode::KeyE) => 0x6,
        PhysicalKey::Code(KeyCode::KeyR) => 0xd,
        PhysicalKey::Code(KeyCode::KeyA) => 0x7,
        PhysicalKey::Code(KeyCode::KeyS) => 0x8,
        PhysicalKey::Code(KeyCode::KeyD) => 0x9,
        PhysicalKey::Code(KeyCode::KeyF) => 0xe,
        PhysicalKey::Code(KeyCode::KeyZ) => 0xa,
        PhysicalKey::Code(KeyCode::KeyX) => 0x0,
        PhysicalKey::Code(KeyCode::KeyC) => 0xb,
        PhysicalKey::Code(KeyCode::KeyV) => 0xf,
        _ => { return None },
    };

    Some((key, is_pressed))
}

const ARRAY_FALSE_COLOR: u32 = 0x0000_0000;
const ARRAY_TRUE_COLOR: u32 = 0xffff_ffff;
const DISPLAY_DIMS: (u32, u32) = (DISPLAY_WIDTH, DISPLAY_HEIGHT);
