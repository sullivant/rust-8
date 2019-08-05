use std::fs::File;
use std::io;
use std::io::prelude::*;

struct Cpu {
    // Memory
    memory: [u8; 4096],
    opcode: u8,

    // Registers
    v: [u8; 16],
    i: u8,     // Index register
    pc: usize, // Program Counter

    // Array of graphics pixels ( 64 x 32 )
    gfx: [u8; (64 * 32)],

    // Some timers
    delay_timer: u8,
    sound_timer: u8,

    // Stack and stack pointer
    stack: [u8; 16],
    sp: u8,
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

    pub fn load_rom(&mut self, file: String) -> io::Result<()> {
        let rom = File::open(file)?;
        //let mut buffer: [u8; 1] = [0; 1];
        //rom.read(&mut buffer)?; // Read into buffer
        //println!("{:X}", buffer[0]);
        let mut i = 0;
        for b in rom.bytes() {
            self.memory[0x200 + i] = b.unwrap();
            i = i + 1;
        }

        Ok(())
    }

    // Reads a word from memory located at program counter
    // and increments program counter
    pub fn read_word(&mut self) -> u16 {
        //TODO: Look into ByteOrder crate
        let w: u16 = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        self.pc += 2;
        w
    }
}

pub trait BitReader {
    fn read_u32_be(&mut self) -> Result<u32, io::Error>;
    fn read_u32_le(&mut self) -> Result<u32, io::Error>;
}

fn main() -> io::Result<()> {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    cpu.load_rom("./data/PONG".to_string())?;
    println!("{:X}", cpu.read_word());
    println!("{:X}", cpu.read_word());

    Ok(())
}
