use crate::emulator::calculations::{get_code, get_n, get_nn, get_nnn, get_x, get_y};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; MEM_SIZE],
    display: Arc<Mutex<[u32; 16]>>,
    program_counter: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    variables: [u8; REGISTER_COUNT],
    finish_requested: bool,
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
            display: Arc::new(Mutex::new([0; 16])),
            program_counter: 0x200,
            index: 0,
            stack: vec![],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            variables: [0; REGISTER_COUNT],
            finish_requested: false,
        }
            .with_font_in_memory()
    }

    fn load_program_into_memory(&mut self, buffer: Vec<u8>) {
        self.memory[INITIAL_PC_LOCATION..(INITIAL_PC_LOCATION + buffer.len())].clone_from_slice(buffer.as_slice());
    }

    pub fn load_program(&mut self, filename: &Path) {
        let mut f = File::open(filename).expect("No file found");
        let metadata = fs::metadata(filename).expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("Buffer overflow");
        self.load_program_into_memory(buffer);
    }

    pub fn get_display(&self) -> Arc<Mutex<[u32; 16]>> {
        Arc::clone(&self.display)
    }

    fn get_current_command_code(&self) -> u16 {
        let pc = self.program_counter as usize;
        ((self.memory[pc] as u16) << 8) | self.memory[pc + 1] as u16
    }

    fn get_current_index_value(&self) -> u8 {
        self.get_shifted_index_value(0)
    }

    fn get_shifted_index_value(&self, shift: usize) -> u8 {
        self.memory[self.index as usize + shift]
    }

    fn execute_command(&mut self, command: &u16) {
        let pc = self.program_counter;
        let id = self.index;
        println!("pc {pc:#06x} | executing {command:#06x} | index {id:#06x}"); // TODO: Replace with proper logic

        match get_code(command) {
            0x0 => self.execute_0_command(command),
            0x1 => self.execute_jump_command(command),
            0x2 => todo!(),
            0x3 => todo!(),
            0x4 => todo!(),
            0x5 => todo!(),
            0x6 => self.execute_set_register_command(command),
            0x7 => self.execute_add_to_register_command(command),
            0x8 => todo!(),
            0x9 => todo!(),
            0xa => self.execute_set_index_command(command),
            0xb => todo!(),
            0xc => todo!(),
            0xd => self.execute_draw_command(command),
            0xe => todo!(),
            0xf => todo!(),
            _ => unreachable!(),
        };
    }

    fn execute_current_command(&mut self) {
        let command = self.get_current_command_code();

        self.execute_command(&command);

        if get_code(&command) != 0x1 {
            self.program_counter += 2;
        }
    }

    pub fn begin_execution(&mut self) {
        self.program_counter = INITIAL_PC_LOCATION as u16;

        while !self.finish_requested {
            self.execute_current_command();
        }
    }

    fn execute_0_command(&mut self, command: &u16) {
        match command {
            0x00e0 => self.execute_clear_command(),
            0x0ee0 => todo!(),
            _ => (),
        };
    }

    fn execute_clear_command(&mut self) {
        *self.display.lock().unwrap() = [0; 16];
    }

    fn execute_jump_command(&mut self, command: &u16) {
        let address = get_nnn(command);
        self.program_counter = address;
    }

    fn execute_set_register_command(&mut self, command: &u16) {
        let variable = get_x(command);
        let value = get_nn(command);

        self.variables[variable] = value;
    }

    fn execute_add_to_register_command(&mut self, command: &u16) {
        let variable = get_x(command);
        let value = get_nn(command);

        self.variables[variable] = self.variables[variable].saturating_add(value);
    }

    fn execute_set_index_command(&mut self, command: &u16) {
        let value = get_nnn(command);
        self.index = value;
    }

    fn execute_draw_command(&mut self, command: &u16) {
        let x = get_x(command) % DISPLAY_WIDTH as usize;
        let y = get_y(command) % DISPLAY_HEIGHT as usize;
        let n = get_n(command) as usize;

        for (i, idx) in (y..y + n).enumerate() {
            if idx >= DISPLAY_HEIGHT as usize {
                break;
            }

            let mut sprite = (self.get_shifted_index_value(i) as u32) << 26;
            sprite >>= x;
            (*self.display.lock().unwrap())[idx] ^= sprite;
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

pub const DISPLAY_WIDTH: u32 = 32;
pub const DISPLAY_HEIGHT: u32 = 16;
const MEM_SIZE: usize = 2 << 11;
const REGISTER_COUNT: usize = 16;
const INITIAL_PC_LOCATION: usize = 0x200;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_execute_current_command() {
        let mut emulator = Emulator::new();

        emulator.memory[0x200] = 0x00;
        emulator.memory[0x201] = 0xe0;

        *emulator.display.lock().unwrap() = [0xffff_ffff; 16];
        emulator.execute_current_command();

        assert_eq!(*emulator.display.lock().unwrap(), [0x0000_0000; 16]);
    }

    #[test]
    fn test_execute_clear_command() {
        let mut emulator = Emulator::new();

        *emulator.display.lock().unwrap() = [0xffff_ffff; 16];
        emulator.execute_clear_command();

        assert_eq!(*emulator.display.lock().unwrap(), [0x0000_0000; 16]);
    }

    #[test]
    fn test_execute_jump_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0x1abc;
        emulator.execute_jump_command(&command);

        assert_eq!(emulator.program_counter, 0x0abc);
    }

    #[test]
    fn text_execute_set_register_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0x6abc;
        emulator.execute_set_register_command(&command);

        assert_eq!(emulator.variables[0xa], 0xbc);
    }

    #[test]
    fn test_execute_add_to_register_command() {
        let mut emulator = Emulator::new();

        assert_eq!(emulator.variables[0xa], 0x00);

        let command: u16 = 0x7abc;
        emulator.execute_add_to_register_command(&command);

        assert_eq!(emulator.variables[0xa], 0xbc);

        let command: u16 = 0x7a01;
        emulator.execute_add_to_register_command(&command);

        assert_eq!(emulator.variables[0xa], 0xbd);

        let command: u16 = 0x7abc;
        emulator.execute_add_to_register_command(&command);

        assert_eq!(emulator.variables[0xa], 0xff);
    }

    #[test]
    fn test_execute_set_index_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0xaabc;
        emulator.execute_set_index_command(&command);

        assert_eq!(emulator.index, 0xabc);
    }
}
