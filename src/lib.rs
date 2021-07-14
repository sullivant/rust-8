use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Color, DrawParam, Text};
use ggez::*;
use glam::Vec2;
use std::collections::BTreeMap;
use std::env;
use std::path;
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
pub const DISP_HEIGHT_INFO_AREA: f32 = 200.0; // The added bottom info area for text
pub const DISP_WIDTH_INFO_AREA: f32 = 300.0; // The added right side info area for text

#[derive(StructOpt)]
struct Cli {
    /// The input rom to look for
    rom: String,
}

pub struct App {
    dt: std::time::Duration,
    cpu: Cpu,
    cell: graphics::Mesh,
    texts: BTreeMap<&'static str, Text>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let dt = std::time::Duration::new(0, 0);
        let black = graphics::Color::new(0.0, 0.0, 0.0, 1.0);

        // Generate our CPU
        let mut cpu = Cpu::new();

        // Load the ROM intro the CPU
        let args = Cli::from_args();
        let mut rom_file = "./data/".to_string();
        rom_file += &args.rom;
        match cpu.load_rom(rom_file.clone()) {
            Ok(_) => println!("Loaded rom file: {}", rom_file),
            Err(err) => {
                panic!("Unable to load rom file: {}", err);
            }
        }

        // Setup a "cell"/pixel for the engine to use
        let cell = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, DISP_SCALE, DISP_SCALE),
            black,
        )?;

        // Setup some texts for update later
        let mut texts = BTreeMap::new();
        // Store the text in `App`s map, for drawing in main loop.
        texts.insert("1_romname", Text::new(format!("ROM Loaded: {}", rom_file)));

        // Return a good version of the app object
        Ok(App {
            dt,
            cpu,
            cell,
            texts,
        })
    }
}

impl ggez::event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Frame count timer
        self.dt = timer::delta(ctx);
        while timer::check_update_time(ctx, 60) {
            // Tick the cpu
            self.cpu.tick(false);

            // Update the text array of mapped objects with fresh values
            self.texts.insert(
                "2_opcode",
                Text::new(format!("OP:{:?} PC:{:?}", self.cpu.opcode, self.cpu.pc)),
            );
            self.texts
                .insert("3_pc", Text::new(format!("pc:{:?}", self.cpu.pc)));
        }
        // Let our family know we are ok
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        let black = graphics::Color::new(0.0, 0.0, 0.0, 1.0);

        for (y, row) in self.cpu.gfx.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                let x = (x as f32) * DISP_SCALE;
                let y = (y as f32) * DISP_SCALE;

                if *val == 1 {
                    graphics::draw(ctx, &self.cell, (ggez::mint::Point2 { x, y },))?;
                }
            }
        }

        // Draw text objects/details
        // Create a little FPS text and display it in the info area
        let mut height = DISP_HEIGHT; // Start at the top of the info area

        // A FPS timer (not a mapped obj because it changes rapidly)
        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {}", fps));
        graphics::draw(ctx, &fps_display, (Vec2::new(0.0, height), black))?;

        // Draw the mapped text objects, too
        height += 2.0 + fps_display.height(ctx) as f32; // Prep height to be used for mapped objs
        for text in self.texts.values() {
            graphics::queue_text(ctx, text, Vec2::new(0.0, height), Some(black));
            height += 2.0 + text.height(ctx) as f32;
        }

        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn go() -> GameResult {
    // Create a window.
    let mut main_window = ContextBuilder::new("mygame", "myname");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let path = path::PathBuf::from(manifest_dir).join("resources");
        println!("Adding 'resources' path {:?}", path);
        main_window = main_window.add_resource_path(path);
    }

    // let main_window = ggez::ContextBuilder::new("main_window", "Thomas")
    //     .window_setup(WindowSetup::default().title("CHIP8"))
    //     .window_mode(
    //         WindowMode::default()
    //             .dimensions(
    //                 DISP_WIDTH + DISP_WIDTH_INFO_AREA,
    //                 DISP_HEIGHT + DISP_HEIGHT_INFO_AREA,
    //             )
    //             .resizable(true),
    //     );

    // Build our context
    let (mut ctx, mut event_loop) = main_window.build().unwrap();

    // Build our application
    let mut app = App::new(&mut ctx)?;

    // Run the application
    event::run(&mut ctx, &mut event_loop, &mut app)
}
