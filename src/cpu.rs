use super::{C8_HEIGHT, C8_WIDTH, OPCODE_SIZE};
use crate::fonts::FONT_SET;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}
pub struct Input {
    // There are 16 keys
    pub keys: [bool; 16],
    pub read_keys: bool, // If true, tick() will read the key and store it into key_target
    pub key_target: usize, // Set on op fx0a, where in V[] to store this key
}
impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
impl Input {
    pub fn new() -> Input {
        Input {
            keys: [false; 16],
            read_keys: false,
            key_target: 0,
        }
    }
    pub fn dump_keys(&self) {
        for (i, r) in self.keys.iter().enumerate() {
            println!("key {:X}: {}", i, r);
        }
    }
}

pub struct Cpu {
    // Memory
    pub memory: [u8; 4096],
    pub opcode: u8,

    // Registers
    pub v: [u8; 16],
    pub i: usize,  // Index register
    pub pc: usize, // Program Counter

    // Array of graphics pixels ( 64 x 32 )
    pub gfx: [[u8; C8_WIDTH]; C8_HEIGHT],
    //pub gfx: [u8; C8_WIDTH * C8_HEIGHT],
    pub gfx_updated: bool,

    // Some timers
    pub delay_timer: u8,
    pub sound_timer: u8,

    // Stack and stack pointer
    pub stack: [usize; 16],
    pub sp: usize,

    // Input and keyboard
    pub input: Input,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            memory: [0; 4096],
            opcode: 0x00,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [[0; C8_WIDTH]; C8_HEIGHT],
            gfx_updated: false,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            input: Input::new(),
        };
        cpu.load_fonts();
        cpu
    }

    #[allow(dead_code)]
    pub fn dump_ram(&mut self) {
        for (i, r) in self.memory.iter().enumerate() {
            println!("{:X}: {:X}", i, r);
        }
    }

    #[allow(dead_code)]
    pub fn dump_regs(&mut self) {
        println!("  i: {:?}", self.i);
        println!("  v: {:?}", self.v);
        println!(" sp: {:?}", self.stack);
    }

    #[allow(dead_code)]
    pub fn dump_gfx(&mut self) {
        println!("  g: {:?}", self.gfx);
    }

    // Loads to font set into ram
    fn load_fonts(&mut self) {
        for (i, f) in FONT_SET.iter().enumerate() {
            self.memory[i] = *f;
        }
    }

    // Load the rom into memory, with the 0x200 offset
    pub fn load_rom(&mut self, file: String) -> Result<(), std::io::Error> {
        let rom = File::open(file)?;
        for (i, b) in rom.bytes().enumerate() {
            self.memory[0x200 + i] = b.unwrap();
        }

        Ok(())
    }

    // Reads a word from memory located at program counter
    pub fn read_word(&mut self, dump_regs: bool) -> u16 {
        let hi = self.memory[self.pc] as u16;
        let lo = self.memory[self.pc + 1] as u16;
        let w: u16 = (u16::from(self.memory[self.pc]) << 8) | u16::from(self.memory[self.pc + 1]);
        if dump_regs {
            println!(
                "Instruction read {:#X}:{:#X}: hi:{:#X} lo:{:#X} ",
                self.pc, w, hi, lo
            );
        }

        w
    }

    // TODO: Output state control for sound/graphics
    pub fn tick(&mut self, dump_regs: bool) {
        // Store the key pressed into the expected key_target
        for i in 0..self.input.keys.len() {
            if self.input.keys[i] {
                self.input.read_keys = false;
                self.v[self.input.key_target] = i as u8;
                break;
            }
        }

        // Decrement the delay counter if it is above zero
        self.delay_timer = if self.delay_timer > 0 {
            self.delay_timer - 1
        } else {
            0
        };

        let opcode = self.read_word(dump_regs);
        self.run_opcode(opcode, Some(dump_regs));
        if dump_regs {
            self.dump_regs();
        }
    }

    pub fn run_opcode(&mut self, opcode: u16, dump_regs: Option<bool>) {
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

        if dump_regs.unwrap_or(false) {
            println!("Running opcode: {:X}", opcode);
            println!("  Nibbles: {:?}", nibbles);
            println!(
                "  nnn:{} / kk:{}|{:X} / x,y,n: {},{},{}",
                nnn, kk, kk, x, y, n
            );
        }

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
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),   // Puts value Vy into Vx
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),   // Bitwise OR of Vx and Vy; result in Vx
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),   // Bitwise AND of Vx and Vy; result in Vx
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),   // Bitwise XOR of Vx and Vy; result in Vx
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),   // Vx = Vx + Vy; if carry set VF
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),   // Vx = Vx - Vy; if carry set VF
            (0x08, _, _, 0x06) => self.op_8x06(x),      // SHR Vx {, Vy}
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),   // SUB Vx from Vy
            (0x08, _, _, 0x0E) => self.op_8x0e(x),      // Vx *= 2; with VF set if MSB Vx = 1
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),   // Skip next if Vx != Vy
            (0x0A, _, _, _) => self.op_annn(nnn),       // Load nnn into register I
            (0x0B, _, _, _) => self.op_bnnn(nnn),       // Jump to nnn+v[0]
            (0x0C, _, _, _) => self.op_cxkk(x, kk),     // Set Vx = random byte AND kk.
            (0x0D, _, _, _) => self.op_dxyn(x, y, n),   // Display n-byte sprite
            (0x0E, _, 0x09, 0x0E) => self.op_ex9e(x),   // Skip if key at v[x] is pressed
            (0x0E, _, 0x0A, 0x01) => self.op_exa1(x),   // Skip if key at v[x] is not pressed
            (0x0F, _, 0x00, 0x07) => self.op_fx07(x),   // Vx = Dt value
            (0x0F, _, 0x00, 0x0A) => self.op_fx0a(x),   // Store keypress into v[x]
            (0x0F, _, 0x01, 0x05) => self.op_fx15(x),   // Dt = Vx
            (0x0F, _, 0x01, 0x08) => self.op_fx18(x),   // St = Vx
            (0x0F, _, 0x01, 0x0E) => self.op_fx1e(x),   // I = I + Vx.
            (0x0F, _, 0x02, 0x09) => self.op_fx29(x),   // I = location of sprite for digit Vx.
            (0x0F, _, 0x03, 0x03) => self.op_fx33(x), // BCD rep of Vx in memory locations I, I+1, and I+2.
            (0x0F, _, 0x05, 0x05) => self.op_fx55(x), // Store V0 through Vx in memory starting at I.
            (0x0F, _, 0x06, 0x05) => self.op_fx65(x), // Read V0 through Vx from memory starting at I.
            _ => ProgramCounter::Next,
        };

        match pc_change {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Skip => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::Jump(p) => self.pc = p,
        }
    }

    // Clear screen
    // TODO: Implement graphics
    fn op_00e0(&mut self) -> ProgramCounter {
        for row in self.gfx.iter_mut() {
            for elem in row.iter_mut() {
                *elem = 0;
            }
        }
        self.gfx_updated = true;
        ProgramCounter::Next
    }

    // Subtract 1 from sp and jump to address in stack
    fn op_00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp])
    }

    // PC Jumps to location NNN
    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    // Call subroutine at NNN
    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(nnn)
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
        self.v[x] = self.v[y];
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

    // Vx = Vx + Vy; if carry set VF
    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let vx = self.v[x];
        let oa: (u8, bool) = vx.overflowing_add(self.v[y]);
        self.v[x] = oa.0;
        if oa.1 {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        ProgramCounter::Next
    }

    // Vx = Vx - Vy; if no carry, set VF
    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        let vx = self.v[x];
        let oa: (u8, bool) = vx.overflowing_sub(self.v[y]);
        self.v[x] = oa.0;
        if oa.1 {
            self.v[0xf] = 0;
        } else {
            self.v[0xf] = 1;
        }

        ProgramCounter::Next
    }

    // Vx = Vx SHR 1
    // If LSB of Vx = 1 then VF = 1 else VF = 0; then Vx
    // is divided by 2.
    fn op_8x06(&mut self, x: usize) -> ProgramCounter {
        self.v[0x0F] = self.v[x] & 0b01; // And with 1 to get final bit
        self.v[x] >>= 1; // Shift right 1, dividing by 2
        ProgramCounter::Next
    }

    // SUBN Vx, Vy
    // If Vy > Vx, then VF is set to 1, else VF = 0. Then Vx is subtracted
    // from Vy, and the results stored in Vx.
    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0x0F] = if self.v[y] > self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        ProgramCounter::Next
    }

    // If Most Significant Bit of V[x] = 1 then set V[F] = 1 else V[F] = 0
    // Multiply V[x] by 2
    fn op_8x0e(&mut self, x: usize) -> ProgramCounter {
        self.v[0x0F] = (self.v[x] & 0b1000_0000) >> 7; // Bitmask with shift for 1 or 0
        self.v[x] <<= 1; // Multiply by 2
        ProgramCounter::Next
    }

    // Skip if Vx != Vy
    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] != self.v[y] {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }

    // Load nnn into register I
    fn op_annn(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn op_bnnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn + self.v[0] as usize)
    }

    // Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        let v = rng.gen_range(0 as u16, 256 as u16) as u8; // Inclusive of low, exclusive of high, so 0-255
        self.v[x] = kk.wrapping_add(v);

        ProgramCounter::Next
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // TODO: Separate this into a display module?
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> ProgramCounter {
        // Set VF to zero to start
        self.v[0x0F] = 0;

        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % C8_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % C8_WIDTH;
                let color = (self.memory[self.i + byte] >> (7 - bit)) & 1;
                self.v[0x0F] |= color & self.gfx[y][x];
                self.gfx[y][x] ^= color;
            }
        }

        //self.gfx[vx as usize][vy as usize] = 1;
        self.gfx_updated = true;
        ProgramCounter::Next
    }

    // Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) -> ProgramCounter {
        if self.input.keys[self.v[x] as usize] {
            return ProgramCounter::Skip;
        }

        ProgramCounter::Next
    }

    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) -> ProgramCounter {
        if !self.input.keys[self.v[x] as usize] {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }

    // Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.v[x] = self.delay_timer;
        ProgramCounter::Next
    }

    // Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) -> ProgramCounter {
        self.input.read_keys = true; // Tell tick() to read a key
        self.input.key_target = x; // And store it into V[x]
        println!("~~ Reading keys and storing in v[{}]", x);
        ProgramCounter::Next
    }

    // Set Dt = Vx
    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.v[x];
        ProgramCounter::Next
    }

    // Set sound timer = Vx
    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.v[x];
        ProgramCounter::Next
    }

    // I = I + Vx
    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.i = self.i + self.v[x] as usize;
        ProgramCounter::Next
    }

    // I = location of sprite for digit Vx. Sprites are 5 bytes long each
    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.i = (self.v[x] as usize) * 5;
        ProgramCounter::Next
    }

    // BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        self.memory[self.i] = self.get_digit(self.v[x], 3); // hundreds
        self.memory[self.i + 1] = self.get_digit(self.v[x], 2); // tens
        self.memory[self.i + 2] = self.get_digit(self.v[x], 1); // ones
        ProgramCounter::Next
    }

    // Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        for l in 0..x + 1 {
            self.memory[self.i + l] = self.v[l];
        }
        ProgramCounter::Next
    }

    // Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        for l in 0..x + 1 {
            self.v[l] = self.memory[self.i + l];
        }
        ProgramCounter::Next
    }

    pub fn get_digit(&mut self, number: u8, digit: usize) -> u8 {
        let vec: Vec<u32> = number
            .to_string()
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .rev()
            .collect();

        if digit > vec.len() {
            return 0;
        }
        vec[digit - 1] as u8
    }
}
