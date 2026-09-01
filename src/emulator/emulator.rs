use std::sync::Mutex;

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; MEM_SIZE],
    display: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    program_counter: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: Mutex<u8>,
    sound_timer: Mutex<u8>,
    variables: [u8; VARIABLE_AMOUNT],
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
            display: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            program_counter: 0,
            index: 0,
            stack: vec![],
            delay_timer: Mutex::new(0),
            sound_timer: Mutex::new(0),
            variables: [0; VARIABLE_AMOUNT],
        }
    }
}

const DISPLAY_WIDTH: usize = 32;
const DISPLAY_HEIGHT: usize = 64;
const MEM_SIZE: usize = 2 << 11;
const VARIABLE_AMOUNT: usize = 16;
