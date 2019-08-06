mod cpu;

use std::io;

use cpu::Cpu;

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

    cpu.tick();

    Ok(())
}
