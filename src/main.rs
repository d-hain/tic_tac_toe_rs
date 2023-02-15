#![warn(clippy::unwrap_used)]

use std::ops::{Not};
use anyhow::{Result, Ok, Context};
use std::time::Duration;
use std::vec;
use rand::Rng;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;

/// [`Sign`] to represent the players.
#[derive(Copy, Clone, PartialEq)]
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

impl Not for Sign {
    type Output = Sign;

    fn not(self) -> Self::Output {
        match self {
            Sign::X => Sign::O,
            Sign::O => Sign::X,
        }
    }
}

/// Represents a [`Cell`] of a [`Field`].
#[derive(Clone)]
struct Cell(Option<Sign>);

impl Cell {
    /// Returns true if the [`Cell`] is [`None`].
    fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

/// Contains many [`Cell`]s to represent a [`Field`].
#[derive(Clone)]
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
    fn draw(&self, game_state: &mut GameState, canvas: &mut WindowCanvas) -> Result<()> {
        let window_size = canvas.window().size();
        let cell_size = window_size.0 / 5;
        let padding = cell_size / 4;
        let field_size = cell_size * self.0.len() as u32 + padding * (self.0.len() as u32 - 1);
        let remaining_window_width = (window_size.0 - field_size) as i32;
        let remaining_window_height = (window_size.0 - field_size) as i32;
        let texture_creator = canvas.texture_creator();

        // Fill field_rects with "empty" rects (not a nice solution but it works
        if game_state.field_rects.is_empty() {
            game_state.field_rects = vec![vec![Rect::new(0, 0, 0, 0); self.0[0].len()]; self.0.len()];
        }

        for (row_idx, row) in self.0.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // Construct Cell
                let cell_x_pos = remaining_window_width / 2 + (cell_size * col_idx as u32 + padding * col_idx as u32) as i32;
                let cell_y_pos = remaining_window_height / 2 + (cell_size * row_idx as u32 + padding * row_idx as u32) as i32;

                let cell_rect = Rect::new(
                    cell_x_pos,
                    cell_y_pos,
                    cell_size,
                    cell_size,
                );
                game_state.field_rects[row_idx][col_idx] = cell_rect;

                // Draw Cell
                canvas.set_draw_color(Color::RGB(200, 0, 255));
                canvas.fill_rect(cell_rect).expect("Possibly graphic driver failure!");
                canvas.set_draw_color(BACKGROUND_COLOR);

                // Draw Sign
                let sign_text = match cell.0 {
                    Some(sign) => sign.into(),
                    None => " ",
                };
                let sign_texture = get_text_texture(sign_text, &game_state.font, &texture_creator).context("Creating texture for player Sign.")?;
                let target = Rect::new(cell_x_pos, cell_y_pos, cell_size, cell_size);
                canvas.copy(&sign_texture, None, Some(target)).expect("Displaying texture for player Sign."); //TODO: Really do not want to use expect here
            }
        }

        Ok(())
    }
}

/// How the game has ended.
enum GameResult {
    /// The Sign is what player has won.
    Win(Sign),
    Tie,
}

struct GameState<'a> {
    font: Font<'a, 'a>,
    field: Field,
    /// The actual [`Rect`]s on screen
    field_rects: Vec<Vec<Rect>>,
    current_player: Sign,
    game_result: Option<GameResult>,
}

const BACKGROUND_COLOR: Color = Color::RGB(69, 69, 69);
const FIELD_SIZE: usize = 3;

fn main() {
    let (mut canvas, mut event_pump, ttf_context) = setup_sdl();

    // Setup GameState
    let font = ttf_context.load_font("assets/ComicSansMS3.ttf", 69).expect("Loading font");
    // Get random starting player
    let current_player = rand::thread_rng().gen_range(0_u32..=1_u32);
    let current_player = match current_player {
        0 => Sign::O,
        1 => Sign::X,
        _ => panic!("The rand crate broke or I am stupid.")
    };
    let mut game_state = GameState {
        font,
        field: Field::empty(FIELD_SIZE),
        field_rects: Vec::new(),
        current_player,
        game_result: None,
    };

    // Game Loop
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::MouseButtonDown { y, x, .. } => on_mouse_clicked(x, y, &mut game_state),
                _ => {}
            }
        }

        update(&mut canvas, &mut game_state).expect("Failed updating the game");
    }
}

/// Updates the game and draws to the window
fn update(canvas: &mut WindowCanvas, game_state: &mut GameState) -> Result<()> {
    canvas.clear();

    let field = game_state.field.clone();

    field.draw(game_state, canvas).context("Drawing game Field")?;

    if check_draw(&game_state.field) {
        game_state.game_result = Some(GameResult::Tie);
    }

    if let Some(game_result) = &game_state.game_result {
        draw_end_text(game_result, &game_state.font, canvas)?;
    }

    canvas.present();
    std::thread::sleep(Duration::new(0, 1_000_000_000_u32 / 60));

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

/// Checks if the game has ended in a draw
fn check_draw(field: &Field) -> bool {
    for row in field.0.iter() {
        for cell in row {
            if cell.0.is_none() {
                return false
            }
        }
    }

    true
}

/// Draws the text at the end of the game, when the game ends in a tie, win or lose
fn draw_end_text(game_result: &GameResult, font: &Font, canvas: &mut WindowCanvas) -> Result<()> {
    let texture_creator = canvas.texture_creator();
    let window_size = canvas.window().size();

    let text = match game_result {
        GameResult::Win(sign) => match sign {
            Sign::X => "Player X wins!",
            Sign::O => "Player O wins!",
        },
        GameResult::Tie => "It is a tie!",
    };
    let texture = get_text_texture(text, font, &texture_creator).context("Creating texture for player Sign.")?;

    let text_width = window_size.0 / 5;
    let text_height = window_size.1 / 8;
    let text_x_pos = window_size.0 / 2 - text_width / 2;
    let text_y_pos = window_size.1 / 42;
    let target = Rect::new(text_x_pos as i32, text_y_pos as i32, text_width, text_height);
    canvas.copy(&texture, None, Some(target)).expect("Displaying texture for ending text."); //TODO: Really do not want to use expect here

    Ok(())
}

/// Handles what happens when the mouse is clicked.
fn on_mouse_clicked(x_pos: i32, y_pos: i32, game_state: &mut GameState) {
    let clicked_point = Point::new(x_pos, y_pos);

    for (row_idx, rows) in game_state.field_rects.iter().enumerate() {
        for (col_idx, rect) in rows.iter().enumerate() {
            // Change Rect Sign and switch current player, if the Rect is clicked and it is empty
            if rect.contains_point(clicked_point) && game_state.field.0[row_idx][col_idx].is_empty() {
                game_state.field.0[row_idx][col_idx] = Cell(Some(game_state.current_player));
                switch_player(&mut game_state.current_player);
            }
        }
    }
}

/// Switches the `current_player`
fn switch_player(current_player: &mut Sign) {
    *current_player = !*current_player;
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
