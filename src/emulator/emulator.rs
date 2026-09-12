use crate::emulator::calculations::{extract_values, get_code};
use crate::emulator::config::{AddToIndexBehaviour, EmulatorConfig, MemoryOperationBehaviour, OffsetJumpBehaviour, ShiftBehaviour};
use crate::emulator::timers::{delay_timer_work, sound_timer_work};
use rand::RngExt;
use rand::prelude::SmallRng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs, thread};

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; MEM_SIZE],
    display: [u64; 32],
    program_counter: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: Arc<AtomicU8>,
    sound_timer: Arc<AtomicU8>,
    variables: [u8; REGISTER_COUNT],
    inputs: u16,
    display_sender: Option<Sender<(u8, u64)>>,
    input_receiver: Option<Receiver<(u8, bool)>>,
    rng: SmallRng,
    config: EmulatorConfig,
}

impl Emulator {
    pub fn new(
        display_sender: Option<Sender<(u8, u64)>>,
        input_receiver: Option<Receiver<(u8, bool)>>,
    ) -> Self {
        Self {
            memory: [0; MEM_SIZE],
            display: [0; 32],
            program_counter: 0x200,
            index: 0,
            stack: vec![],
            delay_timer: Arc::new(AtomicU8::new(0)),
            sound_timer: Arc::new(AtomicU8::new(0)),
            variables: [0; REGISTER_COUNT],
            inputs: 0,
            display_sender,
            input_receiver,
            rng: rand::make_rng(),
            config: EmulatorConfig::default(),
        }
            .with_font_in_memory()
            .with_timers_initialized()
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

    fn place_font_in_memory(&mut self) {
        self.memory[FONT_START..FONT_END].clone_from_slice(&FONT);
    }

    fn with_font_in_memory(mut self) -> Self {
        self.place_font_in_memory();
        self
    }

    fn initialize_timers(&mut self) {
        let timer = Arc::clone(&self.delay_timer);
        _ = thread::spawn(|| {
            delay_timer_work(timer);
        });

        let timer = Arc::clone(&self.sound_timer);
        _ = thread::spawn(|| {
            sound_timer_work(timer);
        });
    }

    fn with_timers_initialized(mut self) -> Self {
        self.initialize_timers();
        self
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

        let (code, x, y, n, nn, nnn) = extract_values(command);

        match code {
            0x0 => self.execute_0_command(command),
            0x1 => self.execute_jump_command(nnn),
            0x2 => self.execute_call_subroutine_command(nnn),
            0x3 => self.execute_skip_if_equal_command(x, nn),
            0x4 => self.execute_skip_if_not_equal_command(x, nn),
            0x5 => self.execute_jump_if_registers_equal_command(x, y),
            0x6 => self.execute_set_register_command(x, nn),
            0x7 => self.execute_add_constant_command(x, nn),
            0x8 => self.execute_math_command(x, y, n),
            0x9 => self.execute_skip_if_registers_not_equal_command(x, y),
            0xa => self.execute_set_index_command(nnn),
            0xb => self.execute_jump_with_offset_command(x, nnn),
            0xc => self.execute_random_command(x, nn),
            0xd => self.execute_draw_command(x, y, n),
            0xe => self.execute_skip_if_key_commands(x, nn),
            0xf => self.execute_f_command(x, nn),
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

        loop {
            let start = Instant::now();

            self.execute_current_command();
            self.process_key_events();

            let end = Instant::now();
            sleep(Duration::from_micros(MAX_MICROS_WAIT).saturating_sub(end.duration_since(start)));
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
        self.display = [0; 32];

        if let Some(sender) = &self.display_sender {
            for i in 0..32 {
                sender.send((i as u8, self.display[i])).expect("Failed to send display data");
            }
        }
    }

    fn execute_return_from_subroutine_command(&mut self) {
        let return_address = self.stack.pop().expect("The stack is empty; nowhere to return to");
        self.program_counter = return_address;
    }

    fn execute_jump_command(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    fn execute_call_subroutine_command(&mut self, nnn: u16) {
        self.stack.push(self.program_counter);
        let address = nnn;
        self.program_counter = address;
    }

    fn execute_skip_if_equal_command(&mut self, x: usize, nn: u8) {
        let lhs = self.variables[x];
        let rhs = nn;

        if lhs == rhs {
            self.program_counter += 2;
        }
    }

    fn execute_skip_if_not_equal_command(&mut self, x: usize, nn: u8) {
        let lhs = self.variables[x];
        let rhs = nn;

        if lhs != rhs {
            self.program_counter += 2;
        }
    }

    fn execute_jump_if_registers_equal_command(&mut self, x: usize, y: usize) {
        let lhs = self.variables[x];
        let rhs = self.variables[y];

        if lhs == rhs {
            self.program_counter += 2;
        }
    }

    fn execute_skip_if_registers_not_equal_command(&mut self, x: usize, y: usize) {
        let lhs = self.variables[x];
        let rhs = self.variables[y];

        if lhs != rhs {
            self.program_counter += 2;
        }
    }

    fn execute_set_register_command(&mut self, x: usize, nn: u8) {
        self.variables[x] = nn;
    }

    fn execute_add_constant_command(&mut self, x: usize, nn: u8) {
        self.variables[x] = self.variables[x].wrapping_add(nn);
    }

    fn execute_math_command(&mut self, x: usize, y: usize, n: u8) {
        match n {
            0x0 => self.execute_assign_command(x, y),
            0x1 => self.execute_or_command(x, y),
            0x2 => self.execute_and_command(x, y),
            0x3 => self.execute_xor_command(x, y),
            0x4 => self.execute_add_command(x, y),
            0x5 => self.execute_subtract_command(x, y),
            0x6 => self.execute_shift_right_command(x, y),
            0x7 => self.execute_replace_and_subtract_command(x, y),
            0xe => self.execute_shift_left_command(x, y),
            _ => panic!("Invalid command"),
        }
    }

    fn execute_assign_command(&mut self, x: usize, y: usize) {
        self.variables[x] = self.variables[y];
    }

    fn execute_or_command(&mut self, x: usize, y: usize) {
        self.variables[x] |= self.variables[y];
    }

    fn execute_and_command(&mut self, x: usize, y: usize) {
        self.variables[x] &= self.variables[y];
    }

    fn execute_xor_command(&mut self, x: usize, y: usize) {
        self.variables[x] ^= self.variables[y];
    }

    fn execute_add_command(&mut self, x: usize, y: usize) {
        let (res, carry) = self.variables[x].overflowing_add(self.variables[y]);
        self.variables[x] = res;
        self.variables[0xf] = carry as u8;
    }

    fn execute_subtract_command(&mut self, x: usize, y: usize) {
        let carry = (self.variables[x] >= self.variables[y]) as u8;
        self.variables[x] = self.variables[x].wrapping_sub(self.variables[y]);
        self.variables[0xf] = carry;
    }

    fn execute_replace_and_subtract_command(&mut self, x: usize, y: usize) {
        let carry = (self.variables[x] <= self.variables[y]) as u8;
        self.variables[x] = self.variables[y].wrapping_sub(self.variables[x]);
        self.variables[0xf] = carry;
    }

    fn execute_shift_right_command(&mut self, x: usize, y: usize) {
        if self.config.shift_behaviour == ShiftBehaviour::CopyVY {
            self.variables[x] = self.variables[y];
        }
        let carry = (self.variables[x] & 0b0000_0001 != 0) as u8;
        self.variables[x] >>= 1;
        self.variables[0xf] = carry;
    }

    fn execute_shift_left_command(&mut self, x: usize, y: usize) {
        if self.config.shift_behaviour == ShiftBehaviour::CopyVY {
            self.variables[x] = self.variables[y];
        }
        let carry = (self.variables[x] & 0b1000_0000 != 0) as u8;
        self.variables[x] <<= 1;
        self.variables[0xf] = carry;
    }

    fn execute_set_index_command(&mut self, nnn: u16) {
        self.index = nnn;
    }

    fn execute_jump_with_offset_command(&mut self, x: usize, nnn: u16) {
        self.program_counter = if self.config.offset_jump_behaviour == OffsetJumpBehaviour::AddV0 {
            nnn + self.variables[0] as u16
        } else {
            nnn + self.variables[x] as u16
        }
    }

    fn execute_random_command(&mut self, x: usize, nn: u8) {
        let random: u8 = self.rng.random();
        self.variables[x] = nn & random;
    }

    fn execute_draw_command(&mut self, x: usize, y: usize, n: u8) {
        let vx = self.variables[x] & (DISPLAY_WIDTH - 1) as u8;
        let vy = self.variables[y] & (DISPLAY_HEIGHT - 1) as u8;
        let n = n as usize;
        self.variables[0xf] = 0;

        for (i, idx) in (vy as usize..vy as usize + n).enumerate() {
            if idx >= DISPLAY_HEIGHT as usize {
                break;
            }

            let mut sprite = (self.get_shifted_index_value(i) as u64) << 56;
            sprite >>= vx;

            if self.display[idx] & sprite != 0 {
                self.variables[0xf] = 1
            }

            self.display[idx] ^= sprite;
            if let Some(sender) = &self.display_sender {
                sender.send((idx as u8, self.display[idx])).expect("Failed to send display data");
            }
        }
    }

    fn process_key_events(&mut self) {
        if let Some(receiver) = &self.input_receiver {
            while let Ok((key, pressed)) = receiver.try_recv() {
                match pressed {
                    true => { self.inputs |= 1 << key }
                    false => { self.inputs &= !(1 << key) }
                }
            }
        }
    }

    fn execute_skip_if_key_commands(&mut self, x: usize, nn: u8) {
        match nn {
            0x9e => self.execute_skip_if_key_pressed_command(x),
            0xa1 => self.execute_skip_if_key_not_pressed_command(x),
            _ => panic!("Invalid command"),
        }
    }

    fn execute_skip_if_key_pressed_command(&mut self, x: usize) {
        if self.inputs & (1 << self.variables[x]) != 0 {
            self.program_counter += 2;
        }
    }

    fn execute_skip_if_key_not_pressed_command(&mut self, x: usize) {
        if self.inputs & (1 << self.variables[x]) == 0 {
            self.program_counter += 2;
        }
    }

    fn execute_f_command(&mut self, x: usize, nn: u8) {
        match nn {
            0x07 => self.execute_get_delay_timer_command(x),
            0x15 => self.execute_set_delay_timer_command(x),
            0x18 => self.execute_set_sound_timer_command(x),
            0x1e => self.execute_add_to_index_command(x),
            0x0a => self.execute_get_key_command(x),
            0x29 => self.execute_get_font_character_command(x),
            0x33 => self.execute_binary_coded_decimal_command(x),
            0x55 => self.execute_store_registers_in_memory_command(x),
            0x65 => self.execute_load_registers_from_memory_command(x),
            _ => panic!("Invalid command"),
        }
    }

    fn execute_get_delay_timer_command(&mut self, x: usize) {
        self.variables[x] = self.delay_timer.load(Ordering::Relaxed);
    }

    fn execute_set_delay_timer_command(&mut self, x: usize) {
        self.delay_timer.store(self.variables[x], Ordering::Relaxed);
    }

    fn execute_set_sound_timer_command(&mut self, x: usize) {
        self.sound_timer.store(self.variables[x], Ordering::Relaxed);
    }

    fn execute_add_to_index_command(&mut self, x: usize) {
        let (res, carry) = self.index.overflowing_add(self.variables[x] as u16);

        if self.config.add_to_index_behaviour == AddToIndexBehaviour::Overflow
            && (carry || (res > 0xfff && self.index <= 0xfff)) {
            self.variables[0xf] = 0x1;
        }

        self.index = res;
    }

    fn execute_get_key_command(&mut self, x: usize) {
        loop {
            if let Some(receiver) = &self.input_receiver
                && let Ok((key, pressed)) = receiver.recv() {
                match pressed {
                    true => { self.inputs |= 1 << key }
                    false => { self.inputs &= !(1 << key) }
                }

                if !pressed {
                    self.variables[x] = key;
                    break;
                }
            }
        }
    }

    fn execute_get_font_character_command(&mut self, x: usize) {
        let x = x & 0xf;
        self.index = (FONT_START + (5 * x)) as u16;
    }

    fn execute_binary_coded_decimal_command(&mut self, x: usize) {
        let num = self.variables[x];
        self.memory[self.index as usize] = num / 100;
        self.memory[self.index as usize + 1] = (num % 100) / 10;
        self.memory[self.index as usize + 2] = num % 10;
    }

    fn execute_store_registers_in_memory_command(&mut self, x: usize) {
        let index = self.index as usize;
        self.memory[index..=index + x].clone_from_slice(&self.variables[..=x]);

        if self.config.memory_operation_behaviour == MemoryOperationBehaviour::MoveIndex {
            self.index += x as u16 + 1;
        }
    }

    fn execute_load_registers_from_memory_command(&mut self, x: usize) {
        let index = self.index as usize;
        self.variables[..=x].clone_from_slice(&self.memory[index..=index + x]);

        if self.config.memory_operation_behaviour == MemoryOperationBehaviour::MoveIndex {
            self.index += x as u16 + 1;
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new(None, None)
    }
}

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
const MEM_SIZE: usize = 2 << 11;
const REGISTER_COUNT: usize = 16;
const INITIAL_PC_LOCATION: usize = 0x200;
const PROGRAM_COUNTER_MOVE: u16 = 2;
const MAX_MICROS_WAIT: u64 = 1428;
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
const FONT_START: usize = 0x050;
const FONT_END: usize = 0x0a0;

#[cfg(test)]
mod test {
    use super::*;
    use crate::emulator::config::{MemoryOperationBehaviour, OffsetJumpBehaviour, ShiftBehaviour};

    #[test]
    fn test_current_command() {
        let mut emulator = Emulator::default();

        emulator.memory[0x200] = 0x00;
        emulator.memory[0x201] = 0xe0;

        emulator.display = [0xffff_ffff_ffff_ffff; 32];
        emulator.execute_current_command();

        assert_eq!(emulator.display, [0x0000_0000_0000_0000; 32]);
    }

    #[test]
    fn test_clear_command() {
        let mut emulator = Emulator::default();

        emulator.display = [0xffff_ffff_ffff_ffff; 32];
        emulator.execute_clear_command();

        assert_eq!(emulator.display, [0x0000_0000_0000_0000; 32]);
    }

    #[test]
    fn test_subroutine_execution() {
        let mut emulator = Emulator::default();

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
        let mut emulator = Emulator::default();

        emulator.execute_jump_command(0xabc);

        assert_eq!(emulator.program_counter, 0xabc);
    }

    #[test]
    fn test_skip_if_equal_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0] = 1;

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_equal_command(0, 0);
        assert_eq!(emulator.program_counter, 0x200);

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn test_skip_if_not_equal_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0] = 1;

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_not_equal_command(0, 0);
        assert_eq!(emulator.program_counter, 0x202);

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_not_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x200);
    }

    #[test]
    fn test_jump_if_registers_equal_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0] = 0;
        emulator.variables[1] = 0;

        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x202);

        emulator.variables[0] = 0;
        emulator.variables[1] = 1;

        emulator.program_counter = 0x200;
        emulator.execute_jump_if_registers_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x200);
    }

    #[test]
    fn test_skip_if_registers_not_equal_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0] = 0;
        emulator.variables[1] = 0;

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_registers_not_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x200);

        emulator.variables[0] = 0;
        emulator.variables[1] = 1;

        emulator.program_counter = 0x200;
        emulator.execute_skip_if_registers_not_equal_command(0, 1);
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn text_execute_set_register_command() {
        let mut emulator = Emulator::default();

        emulator.execute_set_register_command(0xa, 0xbc);
        assert_eq!(emulator.variables[0xa], 0xbc);
    }

    #[test]
    fn test_add_constant_command() {
        let mut emulator = Emulator::default();
        assert_eq!(emulator.variables[0xa], 0x00);

        emulator.execute_add_constant_command(0xa, 0xbc);
        assert_eq!(emulator.variables[0xa], 0xbc);

        emulator.execute_add_constant_command(0xa, 0x01);
        assert_eq!(emulator.variables[0xa], 0xbd);

        emulator.execute_add_constant_command(0xa, 0xbc);
        assert_eq!(emulator.variables[0xa], 0x79);
    }

    #[test]
    fn test_assign_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0;
        emulator.variables[1] = 1;

        emulator.execute_assign_command(0, 1);
        assert_eq!(emulator.variables[0], 1);
        assert_eq!(emulator.variables[1], 1);
    }

    #[test]
    fn test_or_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0b1010_0000;
        emulator.variables[1] = 0b0101_0000;

        emulator.execute_or_command(0, 1);
        assert_eq!(emulator.variables[0], 0b1111_0000);
        assert_eq!(emulator.variables[1], 0b0101_0000);
    }

    #[test]
    fn test_and_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0b1010_0000;
        emulator.variables[1] = 0b1101_0000;

        emulator.execute_and_command(0, 1);
        assert_eq!(emulator.variables[0], 0b1000_0000);
        assert_eq!(emulator.variables[1], 0b1101_0000);
    }

    #[test]
    fn test_xor_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0b1010_0000;
        emulator.variables[1] = 0b1101_0000;

        emulator.execute_xor_command(0, 1);
        assert_eq!(emulator.variables[0], 0b0111_0000);
    }

    #[test]
    fn test_add_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0x12;
        emulator.variables[1] = 0x15;

        emulator.execute_add_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0x27);
        assert_eq!(emulator.variables[0x1], 0x15);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.variables[0] = 0xdd;
        emulator.variables[1] = 0xdd;

        emulator.execute_add_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0xba);
        assert_eq!(emulator.variables[0x1], 0xdd);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.variables[0] = 0xfe;
        emulator.variables[1] = 0x01;

        emulator.execute_add_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0xff);
        assert_eq!(emulator.variables[0x1], 0x01);
        assert_eq!(emulator.variables[0xf], 0);
    }

    #[test]
    fn test_subtract_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0x15;
        emulator.variables[1] = 0x12;

        emulator.execute_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0x03);
        assert_eq!(emulator.variables[0x1], 0x12);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.variables[0] = 0x12;
        emulator.variables[1] = 0x15;

        emulator.execute_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0xfd);
        assert_eq!(emulator.variables[0x1], 0x15);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.variables[0] = 0xff;
        emulator.variables[1] = 0xff;

        emulator.execute_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0x0);
        assert_eq!(emulator.variables[0x1], 0xff);
        assert_eq!(emulator.variables[0xf], 1);
    }

    #[test]
    fn test_replace_and_subtract_command() {
        let mut emulator = Emulator::default();

        emulator.variables[0] = 0x12;
        emulator.variables[1] = 0x15;

        emulator.execute_replace_and_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0x03);
        assert_eq!(emulator.variables[0x1], 0x15);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.variables[0] = 0x15;
        emulator.variables[1] = 0x12;

        emulator.execute_replace_and_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0xfd);
        assert_eq!(emulator.variables[0x1], 0x12);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.variables[0] = 0xff;
        emulator.variables[1] = 0xff;

        emulator.execute_replace_and_subtract_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0x0);
        assert_eq!(emulator.variables[0x1], 0xff);
        assert_eq!(emulator.variables[0xf], 1);
    }

    #[test]
    fn test_shift_right_command() {
        let mut emulator = Emulator::default();
        emulator.config.shift_behaviour = ShiftBehaviour::CopyVY;

        emulator.variables[0] = 0b1100_1100;
        emulator.variables[1] = 0b0011_0011;

        emulator.execute_shift_right_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0001_1001);
        assert_eq!(emulator.variables[0x1], 0b0011_0011);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.variables[0] = 0b0011_0011;
        emulator.variables[1] = 0b1100_1100;

        emulator.execute_shift_right_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0110_0110);
        assert_eq!(emulator.variables[0x1], 0b1100_1100);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.config.shift_behaviour = ShiftBehaviour::IgnoreVY;

        emulator.variables[0] = 0b1100_1100;
        emulator.variables[1] = 0b0011_0011;

        emulator.execute_shift_right_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0110_0110);
        assert_eq!(emulator.variables[0x1], 0b0011_0011);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.variables[0] = 0b0011_0011;
        emulator.variables[1] = 0b1100_1100;

        emulator.execute_shift_right_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0001_1001);
        assert_eq!(emulator.variables[0x1], 0b1100_1100);
        assert_eq!(emulator.variables[0xf], 1);
    }

    #[test]
    fn test_shift_left_command() {
        let mut emulator = Emulator::default();
        emulator.config.shift_behaviour = ShiftBehaviour::CopyVY;

        emulator.variables[0] = 0b1100_1100;
        emulator.variables[1] = 0b0011_0011;

        emulator.execute_shift_left_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0110_0110);
        assert_eq!(emulator.variables[0x1], 0b0011_0011);
        assert_eq!(emulator.variables[0xf], 0);

        emulator.variables[0] = 0b0011_0011;
        emulator.variables[1] = 0b1100_1100;

        emulator.execute_shift_left_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b1001_1000);
        assert_eq!(emulator.variables[0x1], 0b1100_1100);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.config.shift_behaviour = ShiftBehaviour::IgnoreVY;

        emulator.variables[0] = 0b1100_1100;
        emulator.variables[1] = 0b0011_0011;

        emulator.execute_shift_left_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b1001_1000);
        assert_eq!(emulator.variables[0x1], 0b0011_0011);
        assert_eq!(emulator.variables[0xf], 1);

        emulator.variables[0] = 0b0011_0011;
        emulator.variables[1] = 0b1100_1100;

        emulator.execute_shift_left_command(0, 1);
        assert_eq!(emulator.variables[0x0], 0b0110_0110);
        assert_eq!(emulator.variables[0x1], 0b1100_1100);
        assert_eq!(emulator.variables[0xf], 0);
    }

    #[test]
    fn test_set_index_command() {
        let mut emulator = Emulator::default();

        emulator.execute_set_index_command(0xabc);
        assert_eq!(emulator.index, 0xabc);
    }

    #[test]
    fn test_jump_with_offset_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0] = 0x001;
        emulator.variables[1] = 0x002;

        emulator.config.offset_jump_behaviour = OffsetJumpBehaviour::AddV0;

        emulator.program_counter = 0x123;
        emulator.execute_jump_with_offset_command(1, 0x100);
        assert_eq!(emulator.program_counter, 0x101);

        emulator.config.offset_jump_behaviour = OffsetJumpBehaviour::AddVX;

        emulator.program_counter = 0x123;
        emulator.execute_jump_with_offset_command(1, 0x100);
        assert_eq!(emulator.program_counter, 0x102);
    }

    #[test]
    fn test_random_command() {
        let mut emulator = Emulator::default();
        let nn = 0b1001_0110;

        for _ in 0..100 {
            emulator.execute_random_command(0, nn);
            assert_eq!(emulator.variables[0] & nn, emulator.variables[0]);
        }
    }

    #[test]
    fn test_draw_command() {
        let mut emulator = Emulator::default();

        emulator.execute_set_index_command(0x200);
        emulator.memory[0x0200..0x0208].clone_from_slice(&[0b1111_1111; 8]);

        emulator.execute_set_register_command(6, 0x02);
        emulator.execute_set_register_command(7, 0x01);

        emulator.execute_draw_command(6, 7, 8);

        assert_eq!(emulator.variables[0xf], 0);
        assert_eq!(emulator.display,
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

        emulator.execute_draw_command(6, 7, 8);

        assert_eq!(emulator.variables[0xf], 1);
        assert_eq!(emulator.display,
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

    #[test]
    fn test_skip_if_key_pressed_command() {
        let mut emulator = Emulator::default();
        emulator.variables[3] = 5;

        emulator.inputs = 0;
        emulator.program_counter = 0x200;

        emulator.execute_skip_if_key_pressed_command(3);
        assert_eq!(emulator.program_counter, 0x200);

        emulator.inputs = 1 << 5;
        emulator.program_counter = 0x200;

        emulator.execute_skip_if_key_pressed_command(3);
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test]
    fn test_skip_if_key_not_pressed_command() {
        let mut emulator = Emulator::default();
        emulator.variables[3] = 5;

        emulator.inputs = 0;
        emulator.program_counter = 0x200;

        emulator.execute_skip_if_key_not_pressed_command(3);
        assert_eq!(emulator.program_counter, 0x202);

        emulator.inputs = 1 << 5;
        emulator.program_counter = 0x200;

        emulator.execute_skip_if_key_not_pressed_command(3);
        assert_eq!(emulator.program_counter, 0x200);
    }

    #[test]
    fn test_add_to_index_command() {
        let mut emulator = Emulator::default();
        emulator.variables[0x1] = 0xff;

        emulator.config.add_to_index_behaviour = AddToIndexBehaviour::Overflow;

        emulator.index = 0x0;
        emulator.variables[0xf] = 0x0;

        for i in 1..=16 {
            emulator.execute_add_to_index_command(0x1);
            assert_eq!(emulator.index, 0xff * i);
            assert_eq!(emulator.variables[0xf], 0x0);
        }

        emulator.execute_add_to_index_command(0x1);
        assert_eq!(emulator.index, 0x10ef);
        assert_eq!(emulator.variables[0xf], 0x1);

        emulator.config.add_to_index_behaviour = AddToIndexBehaviour::NoOverflow;

        emulator.index = 0x0;
        emulator.variables[0xf] = 0x0;

        for i in 1..=16 {
            emulator.execute_add_to_index_command(0x1);
            assert_eq!(emulator.index, 0xff * i);
            assert_eq!(emulator.variables[0xf], 0x0);
        }

        emulator.execute_add_to_index_command(0x1);
        assert_eq!(emulator.index, 0x10ef);
        assert_eq!(emulator.variables[0xf], 0x0);
    }

    #[test]
    fn test_binary_coded_decimal_command() {
        let mut emulator = Emulator::default();

        emulator.index = 0x200;
        emulator.variables[0] = 255;

        emulator.execute_binary_coded_decimal_command(0);

        assert_eq!(emulator.memory[0x200], 2);
        assert_eq!(emulator.memory[0x201], 5);
        assert_eq!(emulator.memory[0x202], 5);
    }

    #[test]
    fn test_memory_commands() {
        let mut emulator = Emulator::default();
        let growing_array: [u8; REGISTER_COUNT] = core::array::from_fn(|i| i as u8 + 1);
        let zero_array = [0u8; REGISTER_COUNT];

        emulator.config.memory_operation_behaviour = MemoryOperationBehaviour::DontMoveIndex;
        emulator.index = 0x200;
        emulator.variables = growing_array;

        emulator.execute_store_registers_in_memory_command(0x9);
        assert_eq!(emulator.memory[0x200..=0x209], growing_array[..=0x9]);
        assert_eq!(emulator.memory[0x20a..=0x20f], zero_array[0xa..=0xf]);
        assert_eq!(emulator.index, 0x200);

        emulator.index = 0x200;
        emulator.variables = zero_array;
        emulator.memory[0x200..=0x20f].copy_from_slice(&growing_array);

        emulator.execute_load_registers_from_memory_command(0x9);
        assert_eq!(emulator.variables[0x0..=0x9], growing_array[..=0x9]);
        assert_eq!(emulator.variables[0xa..=0xf], zero_array[0xa..=0xf]);
        assert_eq!(emulator.index, 0x200);

        emulator.memory[0x200..=0x20f].copy_from_slice(&zero_array);

        emulator.config.memory_operation_behaviour = MemoryOperationBehaviour::MoveIndex;
        emulator.index = 0x200;
        emulator.variables = growing_array;

        emulator.execute_store_registers_in_memory_command(0x9);
        assert_eq!(emulator.memory[0x200..=0x209], growing_array[..=0x9]);
        assert_eq!(emulator.memory[0x20a..=0x20f], zero_array[0xa..=0xf]);
        assert_eq!(emulator.index, 0x20a);

        emulator.index = 0x200;
        emulator.variables = zero_array;
        emulator.memory[0x200..=0x20f].copy_from_slice(&growing_array);

        emulator.execute_load_registers_from_memory_command(0x9);
        assert_eq!(emulator.variables[0x0..=0x9], growing_array[..=0x9]);
        assert_eq!(emulator.variables[0xa..=0xf], zero_array[0xa..=0xf]);
        assert_eq!(emulator.index, 0x20a);
    }
}
