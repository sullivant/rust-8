mod cpu;
mod fonts;

pub use cpu::Cpu;
use std::io;

pub const OPCODE_SIZE: usize = 2;
pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;

pub fn go() -> io::Result<()> {
    let mut cpu = Cpu::new();
    cpu.load_rom("./data/PONG".to_string())?;

    for _i in 1..5 {
        cpu.tick();
        cpu.dump_regs();
    }

    Ok(())
}
