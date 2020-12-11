mod cpu;
mod display;
mod fonts;

use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

pub use cpu::Cpu;
//use display::DisplayDriver;
//use std::io;

pub const OPCODE_SIZE: usize = 2;
pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;

//pub fn go() -> io::Result<()> {
pub fn go() -> Result<(), String> {
    let mut cpu = Cpu::new();
    cpu.load_rom("./data/IBM".to_string()).unwrap();

    let mut buffer: Vec<u32> = vec![0; C8_WIDTH * C8_HEIGHT];

    let mut window = Window::new(
        "Test",
        C8_WIDTH,
        C8_HEIGHT,
        minifb::WindowOptions {
            resize: true, // TODO allow resize
            scale: minifb::Scale::X8,
            ..minifb::WindowOptions::default()
        },
    )
    //WindowOptions::default())
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, x) in cpu.gfx.iter_mut().enumerate() {
            if *x != 0 {
                buffer[i] = from_u8_rgb(255, 255, 255);
            }
            //*x = from_u8_rgb(255, 255, 255); // write something more funny here!
        }

        //for (i, x) in buffer.iter_mut().enumerate() {}

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        cpu.tick();
        cpu.dump_regs();

        println!("!!! ONE TICK !!!");

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        //window
        //    .update_with_buffer(&buffer, C8_WIDTH, C8_HEIGHT)
        //    .unwrap();
    }

    Ok(())
}

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
