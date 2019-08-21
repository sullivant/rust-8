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
    pub stack: [usize; 16],
    pub sp: usize,
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
        println!("  v: {:?}", self.v);
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
            // Skipping 0NNN (Jump to machine code at location NNN)
            (0x00, 0x00, 0x0E, 0x00) => self.op_00e0(), // Clears the screen
            (0x00, 0x00, 0x0E, 0x0E) => self.op_00ee(), // Set PC to addr at top of stack and sub 1 from sp.
            (0x01, _, _, _) => self.op_1nnn(nnn),       // PC Jumps to location at nnn
            (0x02, _, _, _) => self.op_2nnn(nnn),       // Call subroutine at nnn
            (0x03, _, _, _) => self.op_3xkk(x, kk),     // Skip if Vx = kk
            (0x04, _, _, _) => self.op_4xkk(x, kk),     // Skip if Vx != kk
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),   // Skip if Vx = Vy
            (0x06, _, _, _) => self.op_6xkk(x, kk),     // Puts value kk into register Vx
            (0x07, _, _, _) => self.op_7xkk(x, kk),     // Sets Vx = Vx + kk with overflow
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),   // Puts value Vx into Vx
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),   // Bitwise OR of Vx and Vy; result in Vx
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),   // Bitwise AND of Vx and Vy; result in Vx
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),   // Bitwise XOR of Vx and Vy; result in Vx
            _ => ProgramCounter::Next,
        };

        match pc_change {
            ProgramCounter::Next => self.pc += 2,
            ProgramCounter::Skip => self.pc += 4,
            ProgramCounter::Jump(p) => self.pc = p,
        }
    }

    // Clear screen
    // TODO: Implement graphics
    fn op_00e0(&mut self) -> ProgramCounter {
        ProgramCounter::Next
    }

    // Jump program counter to address at stack[sp] then subtract 1 from sp
    fn op_00ee(&mut self) -> ProgramCounter {
        let p = self.sp;
        self.sp -= 1;
        ProgramCounter::Jump(p)
    }

    // PC Jumps to location NNN
    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    // Call subroutine at NNN
    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = nnn;
        ProgramCounter::Next
    }

    // Skip next if Vx = kk
    fn op_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.v[x] == kk {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }

    // Skip next if Vx != kk
    fn op_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.v[x] != kk {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }

    // Skip next if Vx = Vy
    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] == self.v[y] {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = kk;
        ProgramCounter::Next
    }

    // Set Vx = Vx + kk (overflow mod 256 if needed)
    fn op_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let vx = self.v[x];
        let r: (u8, bool) = vx.overflowing_add(kk);
        self.v[x] = r.0;

        ProgramCounter::Next
    }

    // Puts Vy into Vx
    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[y] = self.v[x];
        ProgramCounter::Next
    }

    // Bitwise OR of Vx and Vy with result in Vx
    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] |= self.v[y];
        ProgramCounter::Next
    }

    // Bitwise AND of Vx and Vy with result in Vx
    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];
        ProgramCounter::Next
    }

    // Bitwise XOR of Vx and Vy with result in Vx
    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];
        ProgramCounter::Next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_7xkk() {
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

        let mut x: usize = 0;
        assert_eq!(cpu.v[x], 0x00);

        // Test add without overflow
        cpu.op_7xkk(x, 0x01);
        assert_eq!(cpu.v[x], 0x01);

        // Test add with overflow on a different register
        x = 1;
        cpu.op_7xkk(x, u8::max_value());
        assert_eq!(cpu.v[x], u8::max_value());
        cpu.op_7xkk(x, 0x02);
        assert_eq!(cpu.v[x], 0x01);
    }

    #[test]
    fn test_op_8xy1() {
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

        // set v[0] to b0001
        cpu.v[0] = 0b0001;
        // set v[1] to b1000
        cpu.v[1] = 0b1000;

        // Should bitwise OR v[0] and v[1]
        cpu.op_8xy1(0, 1);

        assert_eq!(cpu.v[0], 0b1001);
    }

    #[test]
    fn test_op_8xy2() {
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

        // set v[0] to b1001
        cpu.v[0] = 0b1001;
        // set v[1] to b1011
        cpu.v[1] = 0b1011;

        // Should bitwise OR v[0] and v[1]
        cpu.op_8xy2(0, 1);

        assert_eq!(cpu.v[0], 0b1001);
    }

    #[test]
    fn test_op_8xy3() {
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

        // set v[0] to b1001
        cpu.v[0] = 0b1101;
        // set v[1] to b1011
        cpu.v[1] = 0b1011;

        // Should bitwise OR v[0] and v[1]
        cpu.op_8xy3(0, 1);

        assert_eq!(cpu.v[0], 0b0110);
    }
}
