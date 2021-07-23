use ggez::conf::{WindowMode, WindowSetup};
//use ggez::event;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam, Text};
use ggez::input::keyboard;
use ggez::*;
use glam::Vec2;
use nalgebra as na;
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

type Point2 = na::Point2<f32>;

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
    tick_once: bool,
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
            tick_once: false,
        })
    }

    // Just updates the informational text to display in debug mode
    fn update_info_text(&mut self) {
        self.texts.insert(
            "2_opcode",
            Text::new(format!("OP:{:#04x}", self.cpu.opcode)),
        );

        let nibbles = self.cpu.get_nibbles(self.cpu.opcode as u16);
        let nnn = (self.cpu.opcode as u16 & 0x0FFF) as usize;
        let kk = (self.cpu.opcode as u16 & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        self.texts.insert(
            "3_pc",
            Text::new(format!(
                "n:{:#04x},{:#04x},{:#04x},{:#04x} nnn:{:?} kk:{:?} x,y,n: {:?},{:?},{:?} I:{:?} PC:{:?}",
                nibbles.0, nibbles.1,nibbles.2,nibbles.3, nnn, kk, x, y, n, self.cpu.i, self.cpu.pc
            )),
        );
        self.texts.insert(
            "4_v",
            Text::new(format!("v:{:?} v[x]:{:?}", self.cpu.v, self.cpu.v[x])),
        );
        self.texts.insert(
            "5_kt",
            Text::new(format!(
                "rt/kt:{:?}/{:?} : {:?}",
                self.cpu.input.read_keys,
                self.cpu.input.key_target,
                self.cpu.input.dump_keys()
            )),
        );
    }
}

impl ggez::event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Frame count timer
        self.dt = timer::delta(ctx);
        while timer::check_update_time(ctx, 60) {
            // Tick the cpu
            // If we are not in single tick mode (pause_tick = true) then tick away
            if !self.cpu.pause_tick {
                self.cpu.tick(false);
            } else {
                // We are single ticking, wait until we have a space.
                if self.tick_once {
                    self.cpu.tick(false);
                    self.tick_once = false;
                }
            }
            // Update the text array of mapped objects with fresh values
            self.update_info_text();
        }
        // Let our family know we are ok
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        match key {
            // Quit if Shift+Ctrl+Q is pressed.
            KeyCode::Escape => {
                println!("Terminating!");
                event::quit(ctx);
            }
            KeyCode::F1 => {
                self.cpu.pause_tick = if self.cpu.pause_tick { false } else { true };
            }
            KeyCode::Space => {
                self.tick_once = true;
            }
            _ => (),
        }
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

        // Draw a border line above info area
        let mut line = graphics::Mesh::new_line(
            ctx,
            &[
                na::Point2::new(0.0, 0.0),
                na::Point2::new(DISP_WIDTH + DISP_WIDTH_INFO_AREA, 0.0),
            ],
            2.0,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &line, ([0.0, height],))?;

        line = graphics::Mesh::new_line(
            ctx,
            &[na::Point2::new(0.0, 0.0), na::Point2::new(0.0, height)],
            2.0,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &line, ([DISP_WIDTH, 0.0],))?;

        // A FPS timer (not a mapped obj because it changes rapidly)
        height += 2.0;
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
