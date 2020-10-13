mod cpu;
mod display;
mod fonts;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
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
    cpu.load_rom("./data/PONG".to_string()).unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            println!("event: {:?}", event);
            match event {
                Event::Quit { .. }
                | Event::MouseButtonDown { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        // The rest of the game loop goes here...
        cpu.tick();
        //cpu.dump_regs();

        if cpu.gfx_updated {
            println!("Updating canvas");
            // Draw canvas here
            //canvas.clear();
            canvas.present();
            cpu.gfx_updated = false;
        }
    }

    // for _i in 1..5 {
    //     cpu.tick();
    //     cpu.dump_regs();
    // }

    Ok(())
}
