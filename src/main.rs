#![warn(clippy::unwrap_used)]

use anyhow::{Result, Ok, anyhow, Context};
use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, TextureValueError, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::ttf::{FontStyle, Sdl2TtfContext};
use sdl2::video::WindowContext;

/// [`Sign`] to represent the players.
#[derive(Copy, Clone)]
enum Sign {
    X,
    O,
}

impl From<Sign> for &str {
    fn from(sign: Sign) -> Self {
        match sign {
            Sign::X => "X",
            Sign::O => "O",
        }
    }
}

/// Represents a [`Cell`] of a [`Field`].
struct Cell(Option<Sign>);

/// Contains many [`Cell`]s to represent a [`Field`].
struct Field(Vec<Vec<Cell>>);

impl Field {
    /// Creates an empty [`Field`].
    fn empty(size: usize) -> Self {
        Self((0..size).map(|_row| {
            let mut row_vec: Vec<Cell> = Vec::new();
            for _col in 0..size {
                row_vec.push(Cell(None));
            }
            row_vec
        }).collect::<Vec<Vec<Cell>>>())
    }

    /// Draw the [`Field`] to the given `canvas`.
    fn draw(&self, font: &Font, canvas: &mut WindowCanvas) -> Result<()> {
        let window_size = canvas.window().size();
        let cell_size = window_size.0 / 5;
        let padding = cell_size / 4;

        for (row_idx, row) in self.0.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // Display Sign
                let texture_creator = canvas.texture_creator();
                let sign_text = match cell.0 {
                    Some(sign) => sign.into(),
                    None => " ",
                };
                let sign_texture = get_text_texture(sign_text, font, &texture_creator).context("Creating texture for player Sign")?;
                let target = Rect::new(0, 0, 200, 200);
                canvas.copy(&sign_texture, None, Some(target)).expect("Displaying texture for player Sign"); //TODO: Really do not want to use expect here

                canvas.set_draw_color(Color::RGB(200, 0, 255));
                canvas.fill_rect(Rect::new(
                    (cell_size * col_idx as u32 + padding * col_idx as u32) as i32,
                    (cell_size * row_idx as u32 + padding * row_idx as u32) as i32,
                    cell_size,
                    cell_size,
                )).expect("Possibly graphic driver failure!");
                canvas.set_draw_color(BACKGROUND_COLOR);
            }
        }

        Ok(())
    }
}

struct GameState<'a> {
    font: Font<'a, 'a>,
    field: Field,
}

const BACKGROUND_COLOR: Color = Color::RGB(69, 69, 69);

fn main() {
    let (mut canvas, mut event_pump, ttf_context) = setup_sdl();

    // Setup GameState
    let font = ttf_context.load_font("assets/ComicSansMS3.ttf", 28).expect("Loading font");
    let mut game_state = GameState {
        font,
        field: Field::empty(3),
    };
    game_state.field.0[0][0] = Cell(Some(Sign::X)); // TODO: temp

    // Game Loop
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        update(&mut canvas, &mut game_state).expect("Failed updating the game");
    }
}

/// Updates the game and draws to the window
fn update(canvas: &mut WindowCanvas, game_state: &mut GameState) -> Result<()> {
    canvas.clear();

    game_state.field.draw(&game_state.font, canvas).context("Drawing game Field")?;

    canvas.present();
    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

    Ok(())
}

/// Setup everything that has to do with SDL2
fn setup_sdl() -> (WindowCanvas, EventPump, Sdl2TtfContext) {
    let sdl_context = sdl2::init().expect("Initializing SDL2");
    let video_subsystem = sdl_context.video().expect("Initializing Video Subsystem");
    let ttf_context = sdl2::ttf::init().expect("Initializing TTF Context");

    let window = video_subsystem.window("tic_tac_toe_rs", 600, 600)
        .position_centered()
        .resizable()
        .build()
        .expect("Window creation went terribly wrong!");

    let mut canvas = window.into_canvas().build().expect("Building Canvas");

    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump().expect("Getting Event Dump");

    (canvas, event_pump, ttf_context)
}

fn get_text_texture<'a>(
    text: impl Into<&'a str>,
    font: &'a Font<'a, 'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>> {
    let surface = font
        .render(text.into())
        .blended(Color::RGB(0, 255, 0))?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)?;

    Ok(texture)
}
