use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Color, DrawMode, Text};
use ggez::*;
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
pub const DISP_SCALE: f32 = 10.0;
pub const DISP_WIDTH: f32 = 640.0;
pub const DISP_HEIGHT: f32 = 320.0;

#[derive(StructOpt)]
struct Cli {
    /// The input rom to look for
    rom: String,
}

pub struct App {
    // TODO: Make this actually be the cpu.gfx
    vbuff: [[u8; C8_WIDTH as usize]; C8_HEIGHT as usize],
    dt: std::time::Duration,
    cpu: Cpu,
}

impl App {
    fn new() -> GameResult<App> {
        let dt = std::time::Duration::new(0, 0);
        let vbuff = [[0; C8_WIDTH]; C8_HEIGHT];
        let mut cpu = Cpu::new();

        let args = Cli::from_args();
        let mut rom_file = "./data/".to_string();
        rom_file += &args.rom;
        match cpu.load_rom(rom_file.clone()) {
            Ok(_) => println!("Loaded rom file: {}", rom_file),
            Err(err) => {
                panic!("Unable to load rom file: {}", err);
            }
        }

        Ok(App { dt, vbuff, cpu })
    }
}

impl ggez::event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Frame count timer
        self.dt = timer::delta(ctx);

        // Tick the cpu
        self.cpu.tick(false);

        // Copy the cpu's graphics array over to the rendering
        // system's copy
        self.vbuff = self.cpu.gfx;

        // Let our family know we are ok
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        let black = graphics::Color::new(0.0, 0.0, 0.0, 1.0);

        for (y, row) in self.vbuff.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                let x = (x as f32) * DISP_SCALE;
                let y = (y as f32) * DISP_SCALE;

                if *val == 1 {
                    // we need to draw a rectangle there

                    // pub fn new_rectangle(
                    //     ctx: &mut Context,
                    //     mode: DrawMode,
                    //     bounds: Rect,
                    //     color: Color,
                    // ) -> GameResult<Mesh> {
                    //     let mut mb = MeshBuilder::new();
                    //     let _ = mb.rectangle(mode, bounds, color);
                    //     mb.build(ctx)
                    // }

                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(x, y, DISP_SCALE, DISP_SCALE),
                        black,
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn go() -> GameResult {
    // Create a window.
    let main_window = ggez::ContextBuilder::new("main_window", "Thomas")
        .window_setup(WindowSetup::default().title("CHIP8"))
        .window_mode(
            WindowMode::default()
                .dimensions(DISP_WIDTH, DISP_HEIGHT)
                .resizable(true),
        );

    // Build our context
    let (mut ctx, mut event_loop) = main_window.build()?;

    // Build our application
    let mut app = App::new()?;

    // Run the application
    event::run(&mut ctx, &mut event_loop, &mut app)
}
