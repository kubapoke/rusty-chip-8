use std::env::var;
use crate::emulator::calculations::{get_code, get_n, get_nn, get_nnn, get_x, get_y};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::fs;
use crate::emulator::config::EmulatorConfig;

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; MEM_SIZE],
    display: Arc<Mutex<[u64; 32]>>,
    program_counter: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    variables: [u8; REGISTER_COUNT],
    inputs: u16,
    config: EmulatorConfig,
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
            display: Arc::new(Mutex::new([0; 32])),
            program_counter: 0x200,
            index: 0,
            stack: vec![],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            variables: [0; REGISTER_COUNT],
            inputs: 0,
            config: EmulatorConfig::default(),
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

    pub fn get_display(&self) -> Arc<Mutex<[u64; 32]>> {
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
            0x2 => self.execute_call_subroutine_command(command),
            0x3 => self.execute_jump_if_equal_command(command),
            0x4 => self.execute_jump_if_not_equal_command(command),
            0x5 => self.execute_jump_if_registers_equal_command(command),
            0x6 => self.execute_set_register_command(command),
            0x7 => self.execute_add_to_register_command(command),
            0x8 => self.execute_math_command(command),
            0x9 => self.execute_jump_if_registers_not_equal_command(command),
            0xa => self.execute_set_index_command(command),
            0xb => todo!(),
            0xc => todo!(),
            0xd => self.execute_draw_command(command),
            0xe => todo!(),
            0xf => todo!(),
            _ => panic!("Invalid command"),
        };
    }

    fn execute_current_command(&mut self) {
        let command = self.get_current_command_code();

        self.execute_command(&command);

        let code = get_code(&command);
        if code != 0x1 && code != 0x2 {
            self.program_counter += PROGRAM_COUNTER_MOVE;
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
            0x00ee => self.execute_return_from_subroutine_command(),
            _ => (),
        };
    }

    fn execute_clear_command(&mut self) {
        *self.display.lock().unwrap() = [0; 32];
    }

    fn execute_return_from_subroutine_command(&mut self) {
        let return_address = self.stack.pop().expect("The stack is empty; nowhere to return to");
        self.program_counter = return_address;
    }

    fn execute_jump_command(&mut self, command: &u16) {
        let address = get_nnn(command);
        self.program_counter = address;
    }

    fn execute_call_subroutine_command(&mut self, command: &u16) {
        self.stack.push(self.program_counter);
        let address = get_nnn(command);
        self.program_counter = address;
    }

    fn execute_jump_if_equal_command(&mut self, command: &u16) {
        let lhs = self.variables[get_x(command)];
        let rhs = get_nn(command);

        if (lhs == rhs) {
            self.program_counter += 2;
        }
    }

    fn execute_jump_if_not_equal_command(&mut self, command: &u16) {
        let lhs = self.variables[get_x(command)];
        let rhs = get_nn(command);

        if (lhs != rhs) {
            self.program_counter += 2;
        }
    }

    fn execute_jump_if_registers_equal_command(&mut self, command: &u16) {
        let lhs = self.variables[get_x(command)];
        let rhs = self.variables[get_y(command)];

        if (lhs == rhs) {
            self.program_counter += 2;
        }
    }

    fn execute_jump_if_registers_not_equal_command(&mut self, command: &u16) {
        let lhs = self.variables[get_x(command)];
        let rhs = self.variables[get_y(command)];

        if (lhs != rhs) {
            self.program_counter += 2;
        }
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

    fn execute_math_command(&mut self, command: &u16) {
        match get_n(command) {
            0x0 => self.execute_assign_command(command),
            0x1 => todo!(),
            0x2 => todo!(),
            0x3 => todo!(),
            0x4 => todo!(),
            0x5 => todo!(),
            0x6 => todo!(),
            0x7 => todo!(),
            0xe => todo!(),
            _ => panic!("Invalid command"),
        }
    }

    fn execute_assign_command(&mut self, command: &u16) {
        let x = get_x(command);
        let y = get_y(command);
        self.variables[x] = self.variables[y];
    }

    fn execute_set_index_command(&mut self, command: &u16) {
        let value = get_nnn(command);
        self.index = value;
    }

    fn execute_draw_command(&mut self, command: &u16) {
        let x = get_x(command);
        let vx = self.variables[x] & (DISPLAY_WIDTH - 1) as u8;
        let y = get_y(command);
        let vy = self.variables[y] & (DISPLAY_HEIGHT - 1) as u8;
        let n = get_n(command) as usize;

        for (i, idx) in (vy as usize..vy as usize + n).enumerate() {
            if idx >= DISPLAY_HEIGHT as usize {
                break;
            }

            let mut sprite = (self.get_shifted_index_value(i) as u64) << 56;
            sprite >>= vx;

            if (*self.display.lock().unwrap())[idx] & sprite != 0 {
                self.variables[0xf] = 1
            } else {
                self.variables[0xf] = 0;
            }

            (*self.display.lock().unwrap())[idx] ^= sprite;
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
const MEM_SIZE: usize = 2 << 11;
const REGISTER_COUNT: usize = 16;
const INITIAL_PC_LOCATION: usize = 0x200;
const PROGRAM_COUNTER_MOVE: u16 = 2;
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
    fn test_current_command() {
        let mut emulator = Emulator::new();

        emulator.memory[0x200] = 0x00;
        emulator.memory[0x201] = 0xe0;

        *emulator.display.lock().unwrap() = [0xffff_ffff_ffff_ffff; 32];
        emulator.execute_current_command();

        assert_eq!(*emulator.display.lock().unwrap(), [0x0000_0000_0000_0000; 32]);
    }

    #[test]
    fn test_clear_command() {
        let mut emulator = Emulator::new();

        *emulator.display.lock().unwrap() = [0xffff_ffff_ffff_ffff; 32];
        emulator.execute_clear_command();

        assert_eq!(*emulator.display.lock().unwrap(), [0x0000_0000_0000_0000; 32]);
    }

    #[test]
    fn test_subroutine_execution() {
        let mut emulator = Emulator::new();

        emulator.memory[0x200..=0x201].clone_from_slice(&[0x24, 0x00]);
        emulator.memory[0x402..=0x403].clone_from_slice(&[0x00, 0xee]);

        assert_eq!(emulator.program_counter, 0x200);
        emulator.execute_current_command();
        assert_eq!(emulator.program_counter, 0x400);
        emulator.execute_current_command();
        assert_eq!(emulator.program_counter, 0x402);
        emulator.execute_current_command();
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn test_jump_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0x1abc;
        emulator.execute_jump_command(&command);

        assert_eq!(emulator.program_counter, 0x0abc);
    }

    #[test]
    fn test_jump_if_equal_command() {
        let mut emulator = Emulator::new();
        emulator.variables[0] = 1;

        let command: u16 = 0x3000;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x200);

        let command: u16 = 0x3001;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn test_jump_if_not_equal_command() {
        let mut emulator = Emulator::new();
        emulator.variables[0] = 1;

        let command: u16 = 0x4000;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_not_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x202);

        let command: u16 = 0x4001;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_not_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x200);
    }

    #[test]
    fn test_jump_if_registers_equal_command() {
        let mut emulator = Emulator::new();
        emulator.variables[0] = 0;
        emulator.variables[1] = 0;

        let command: u16 = 0x5010;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x202);

        emulator.variables[0] = 0;
        emulator.variables[1] = 1;

        let command: u16 = 0x5010;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x200);
    }

    #[test]
    fn test_jump_if_registers_not_equal_command() {
        let mut emulator = Emulator::new();
        emulator.variables[0] = 0;
        emulator.variables[1] = 0;

        let command: u16 = 0x5010;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_not_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x200);

        emulator.variables[0] = 0;
        emulator.variables[1] = 1;

        let command: u16 = 0x5010;
        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_not_equal_command(&command);
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn text_execute_set_register_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0x6abc;
        emulator.execute_set_register_command(&command);

        assert_eq!(emulator.variables[0xa], 0xbc);
    }

    #[test]
    fn test_add_to_register_command() {
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
    fn test_assign_command() {
        let mut emulator = Emulator::new();
        
        emulator.variables[0] = 0;
        emulator.variables[1] = 1;
        
        let command: u16 = 0x8010;
        emulator.execute_assign_command(&command);

        assert_eq!(emulator.variables[0], 1);
    }

    #[test]
    fn test_set_index_command() {
        let mut emulator = Emulator::new();

        let command: u16 = 0xaabc;
        emulator.execute_set_index_command(&command);

        assert_eq!(emulator.index, 0xabc);
    }

    #[test]
    fn test_draw_command() {
        let mut emulator = Emulator::new();

        emulator.execute_set_index_command(&0x0200);
        emulator.memory[0x0200..0x0208].clone_from_slice(&[0b1111_1111; 8]);

        emulator.execute_set_register_command(&0x6602);
        emulator.execute_set_register_command(&0x6701);

        let command: u16 = 0xd678;
        emulator.execute_draw_command(&command);

        assert_eq!(emulator.variables[0xf], 0);
        assert_eq!(*emulator.display.lock().unwrap(),
                   [
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0011_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                   ]
        );

        emulator.execute_draw_command(&command);

        assert_eq!(emulator.variables[0xf], 1);
        assert_eq!(*emulator.display.lock().unwrap(),
                   [
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                       0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                   ]
        );
    }
}
