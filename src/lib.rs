mod cpu;
mod fonts;

pub use cpu::Cpu;
use std::io;

pub const OPCODE_SIZE: usize = 2;
pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;

pub fn go() -> io::Result<()> {
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

    for _i in 1..5 {
        cpu.tick();
        cpu.dump_regs();
    }

    Ok(())
}
