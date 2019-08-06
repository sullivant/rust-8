use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct Cpu {
    // Memory
    pub memory: [u8; 4096],
    pub opcode: u8,

    // Registers
    pub v: [u8; 16],
    pub i: u8,     // Index register
    pub pc: usize, // Program Counter

    // Array of graphics pixels ( 64 x 32 )
    pub gfx: [u8; (64 * 32)],

    // Some timers
    pub delay_timer: u8,
    pub sound_timer: u8,

    // Stack and stack pointer
    pub stack: [u8; 16],
    pub sp: u8,
}
impl Cpu {
    pub fn initialize(&mut self) {
        self.pc = 0x200;
        self.opcode = 0x00;
        self.i = 0x00;

        self.gfx = [0; (64 * 32)];
        self.stack = [0; 16];
        self.sp = 0;
        self.v = [0; 16];
        self.memory = [0; 4096];

        self.delay_timer = 0;
        self.sound_timer = 0;
    }

    // Load the rom into memory, with the 0x200 offset
    pub fn load_rom(&mut self, file: String) -> io::Result<()> {
        let rom = File::open(file)?;
        for (i, b) in rom.bytes().enumerate() {
            self.memory[0x200 + i] = b.unwrap();
        }

        Ok(())
    }

    // Reads a word from memory located at program counter
    pub fn read_word(&mut self) -> u16 {
        //TODO: Look into ByteOrder crate
        let w: u16 = (u16::from(self.memory[self.pc]) << 8) | u16::from(self.memory[self.pc + 1]);
        w
    }

    // TODO: Output state control for sound/graphics
    pub fn tick(&mut self) {
        println!("{:X}", self.read_word());
        self.pc += 2;
    }
}
