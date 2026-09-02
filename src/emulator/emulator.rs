use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; MEM_SIZE],
    display: [u32; 16],
    program_counter: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    variables: [u8; VARIABLE_AMOUNT],
}

impl Emulator {
    fn place_font_in_memory(&mut self) {
        self.memory[0x050..0x0a0].clone_from_slice(&FONT);
    }

    fn with_font_in_memory(mut self) -> Self {
        self.place_font_in_memory();
        self
    }

    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
            display: [0; 16],
            program_counter: 0,
            index: 0,
            stack: vec![],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            variables: [0; VARIABLE_AMOUNT],
        }
            .with_font_in_memory()
    }

    fn load_program_into_memory(&mut self, buffer: Vec<u8>) {
        self.memory[200..(200 + buffer.len())].clone_from_slice(buffer.as_slice());
    }

    pub fn load_program(&mut self, filename: &Path) {
        let mut f = File::open(&filename).expect("No file found");
        let metadata = fs::metadata(&filename).expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("Buffer overflow");
        self.load_program_into_memory(buffer);
    }

    pub fn get_display(&self) -> &[u32; 16] {
        &self.display
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

const MEM_SIZE: usize = 2 << 11;
const VARIABLE_AMOUNT: usize = 16;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
