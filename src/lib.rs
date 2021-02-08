extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;

use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston_window::*;
use std::time::Duration;
use structopt::StructOpt;

mod cpu;
mod display;
mod fonts;

pub use cpu::Cpu;
//use display::DisplayDriver;
//use std::io;

pub const OPCODE_SIZE: usize = 2;
pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;
pub const DISP_SCALE: f64 = 20.0;

#[derive(StructOpt)]
struct Cli {
    /// The input rom to look for
    rom: String,
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.

    // TODO: Make this actually be the cpu.gfx
    vbuff: [[u8; C8_WIDTH as usize]; C8_HEIGHT as usize],
}
impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let pixel = rectangle::square(0.0, 0.0, DISP_SCALE);

        self.gl.draw(args.viewport(), |_c, gl| {
            // Clear the screen.
            clear(RED, gl);
        });

        // for each of the pixels in gfx, draw them as a black dot on gl
        for (y, row) in self.vbuff.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                let x = (x as f64) * DISP_SCALE;
                let y = (y as f64) * DISP_SCALE;

                if *val == 1 {
                    // We need to draw a black rect there
                    self.gl.draw(args.viewport(), |c, gl| {
                        let transform = c.transform.trans(x, y);
                        rectangle(BLACK, pixel, transform, gl);
                    });
                }
            }
        }
    }
}

pub fn go() -> Result<(), String> {
    let args = Cli::from_args();
    let mut rom_file = "./data/".to_string();
    rom_file += &args.rom;

    // Create a new game and run it.
    let opengl = OpenGL::V3_2;
    let mut events = Events::new(EventSettings::new());

    // This processes opcodes, etc.
    let mut cpu = Cpu::new();

    match cpu.load_rom(rom_file.clone()) {
        Ok(_) => println!("Loaded rom file: {}", rom_file),
        Err(err) => {
            println!("Unable to load rom file: {}", err);
            return Ok(());
        }
    }

    // Create a window.
    let mut main_window: PistonWindow = WindowSettings::new(
        "rust-8",
        [C8_WIDTH as f64 * DISP_SCALE, C8_HEIGHT as f64 * DISP_SCALE],
    )
    .graphics_api(opengl)
    .vsync(true)
    .exit_on_esc(true)
    .build()
    .unwrap();
    let mut app = App {
        gl: GlGraphics::new(opengl),
        vbuff: [[0; C8_WIDTH]; C8_HEIGHT],
    };

    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    while let Some(e) = events.next(&mut main_window) {
        cpu.tick(false);

        // Copy the cpu's graphics array over to the rendering
        // system's copy
        app.vbuff = cpu.gfx;

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
    }

    println!("Exiting application.");

    Ok(())
}
