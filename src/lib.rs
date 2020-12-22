extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
//use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::input::*;
use piston::window::WindowSettings;
use std::time::Duration;

mod cpu;
mod display;
mod fonts;

pub use cpu::Cpu;
//use display::DisplayDriver;
//use std::io;

pub const OPCODE_SIZE: usize = 2;
pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;
pub const DISP_SCALE: f64 = 10.0;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.

    // TODO: Make this actually be the cpu.gfx
    vbuff: [[u8; C8_WIDTH as usize]; C8_HEIGHT as usize],
}
impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let pixel = rectangle::square(0.0, 0.0, DISP_SCALE);

        self.gl.draw(args.viewport(), |_c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
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

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 4.0 * args.dt;
    }
}

//pub fn go() -> io::Result<()> {
pub fn go() -> Result<(), String> {
    let mut cpu = Cpu::new();
    cpu.load_rom("./data/IBM".to_string()).unwrap();

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "rust-8",
        [C8_WIDTH as f64 * DISP_SCALE, C8_HEIGHT as f64 * DISP_SCALE],
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        vbuff: [[0; C8_WIDTH]; C8_HEIGHT],
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        // Just make a few default things turn on
        cpu.gfx[0][0] = 1;
        cpu.gfx[0][C8_WIDTH - 1] = 1;
        cpu.gfx[C8_HEIGHT - 1][0] = 1;
        cpu.gfx[C8_HEIGHT - 1][C8_WIDTH - 1] = 1;

        cpu.tick(true);

        // Copy the cpu's graphics array over to the rendering
        // system's copy
        app.vbuff = cpu.gfx;

        if cpu.gfx_updated {
            if let Some(args) = e.render_args() {
                app.render(&args);
            }

            if let Some(args) = e.update_args() {
                app.update(&args);
            }
        }
        cpu.gfx_updated = false;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    //        cpu.dump_regs();

    Ok(())
}
