use std::fs::File;
use std::io;
use std::io::prelude::*;

use crate::fonts::FONT_SET;

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

pub struct Cpu {
    // Memory
    pub memory: [u8; 4096],
    pub opcode: u8,

    // Registers
    pub v: [u8; 16],
    pub i: u16,    // Index register
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
        self.pc = 0x200; // Starts at 0x200 because 0x00 to 0x1FF is other data
        self.opcode = 0x00;

        // Registers
        self.i = 0x00;
        self.v = [0; 16];

        self.gfx = [0; (64 * 32)];
        self.stack = [0; 16];
        self.sp = 0;
        self.memory = [0; 4096];

        self.delay_timer = 0;
        self.sound_timer = 0;

        self.load_fonts();
    }

    #[allow(dead_code)]
    pub fn dump_ram(&mut self) {
        for (i, r) in self.memory.iter().enumerate() {
            println!("{:X}: {:X}", i, r);
        }
    }

    #[allow(dead_code)]
    pub fn dump_regs(&mut self) {
        println!("Registers:");
        println!("v: {:?}", self.v);
    }

    // Loads to font set into ram
    fn load_fonts(&mut self) {
        for (i, f) in FONT_SET.iter().enumerate() {
            self.memory[i] = *f;
        }
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
    // TODO: Read keypad on each tick
    pub fn tick(&mut self) {
        let opcode = self.read_word();
        self.run_opcode(opcode);
    }

    fn run_opcode(&mut self, opcode: u16) {
        // Break the opcode into its distinct parts so we can determine what
        // to do with what and where
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        println!("Running opcode: {:X}", opcode);
        println!("  Nibbles: {:?}", nibbles);
        println!("  nnn:{} / kk:{} / x,y,n: {},{},{}", nnn, kk, x, y, n);

        //After each runcode we need to update our program counter so we can read
        //a specific opcode out of ram.  Sometimes it's just "the next one", sometimes
        //it's a jump, or a return, etc.
        //
        //Therefore each opcode's function will return either "next", "skip", or "jump"
        //perfect use of an Enum here.

        let pc_change: ProgramCounter = match nibbles {
            (0x06, _, _, _) => self.op_6xkk(x, kk), // Puts value kk into register Vx
            _ => ProgramCounter::Next,
        };

        match pc_change {
            ProgramCounter::Next => self.pc += 2,
            ProgramCounter::Skip => self.pc += 4,
            ProgramCounter::Jump(a) => self.pc = a,
        }
    }

    // Vx = kk
    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = kk;
        ProgramCounter::Next
    }
}
