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
pub const DISP_SCALE: f64 = 20.0;

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

    fn update(&mut self, args: &UpdateArgs) {}
}

//pub fn go() -> io::Result<()> {
pub fn go() -> Result<(), String> {
    let mut cpu = Cpu::new();
    cpu.load_rom("./data/C8TEST".to_string()).unwrap();

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
        vbuff: [[0; C8_WIDTH]; C8_HEIGHT],
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if cpu.input.read_keys {
            println!("Reading keys");
            // Get keypad if asked to get it; waiting until we get it.
            if let Some(Button::Keyboard(key)) = e.press_args() {
                let key_pressed = match key {
                    Key::NumPad1 => Some(0x01),
                    Key::NumPad2 => Some(0x02),
                    Key::NumPad3 => Some(0x03),
                    Key::NumPad4 => Some(0x0c),
                    Key::Q => Some(0x04),
                    Key::W => Some(0x05),
                    Key::E => Some(0x06),
                    Key::R => Some(0x0d),
                    Key::A => Some(0x07),
                    Key::S => Some(0x08),
                    Key::D => Some(0x09),
                    Key::F => Some(0x0e),
                    Key::Z => Some(0x0a),
                    Key::X => Some(0x00),
                    Key::C => Some(0x0b),
                    Key::V => Some(0x0f),
                    _ => None,
                };
                if let Some(i) = key_pressed {
                    println!("Key pressed.");
                    cpu.input.keys[i] = true;
                }
            };
            if let Some(Button::Keyboard(key)) = e.release_args() {
                let key_pressed = match key {
                    Key::NumPad1 => Some(0x01),
                    Key::NumPad2 => Some(0x02),
                    Key::NumPad3 => Some(0x03),
                    Key::NumPad4 => Some(0x0c),
                    Key::Q => Some(0x04),
                    Key::W => Some(0x05),
                    Key::E => Some(0x06),
                    Key::R => Some(0x0d),
                    Key::A => Some(0x07),
                    Key::S => Some(0x08),
                    Key::D => Some(0x09),
                    Key::F => Some(0x0e),
                    Key::Z => Some(0x0a),
                    Key::X => Some(0x00),
                    Key::C => Some(0x0b),
                    Key::V => Some(0x0f),
                    _ => None,
                };
                if let Some(i) = key_pressed {
                    println!("Key released.");
                    cpu.input.keys[i] = false;
                }
            };
            for i in 0..cpu.input.keys.len() {
                if cpu.input.keys[i] {
                    cpu.input.read_keys = false;
                    cpu.v[cpu.input.key_target] = i as u8;
                    break;
                }
            }
            continue;
        }

        cpu.tick(true);

        // Copy the cpu's graphics array over to the rendering
        // system's copy
        app.vbuff = cpu.gfx;

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        cpu.gfx_updated = false;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    //        cpu.dump_regs();

    Ok(())
}
