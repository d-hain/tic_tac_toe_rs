#![warn(clippy::unwrap_used)]

use std::ops::{Not};
use anyhow::{Result, Ok, Context};
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
#[derive(Clone, PartialEq)]
struct Cell(Option<Sign>);

impl Cell {
    /// # Returns 
    ///
    /// true if the [`Cell`] is [`None`].
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

    /// # Returns
    ///
    /// the row count of the [`Field`].
    fn row_count(&self) -> usize {
        self.0.len()
    }

    /// # Returns
    ///
    /// the column count of the [`Field`].
    fn column_count(&self) -> usize {
        self.0[0].len()
    }

    /// Draw the [`Field`] to the given `canvas`.
    fn draw(&self, game_state: &mut GameState, canvas: &mut WindowCanvas) -> Result<()> {
        let window_size = canvas.window().size();
        let cell_size = window_size.0 / 5;
        let padding = cell_size / 4;
        let field_size = cell_size * self.row_count() as u32 + padding * (self.row_count() as u32 - 1);
        let remaining_window_width = (window_size.0 - field_size) as i32;
        let remaining_window_height = (window_size.0 - field_size) as i32;
        let texture_creator = canvas.texture_creator();

        // Fill field_rects with "empty" rects (not a nice solution but it works
        if game_state.field_rects.is_empty() {
            game_state.field_rects = vec![vec![Rect::new(0, 0, 0, 0); self.column_count()]; self.row_count()];
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

struct GameState<'a> {
    font: Font<'a, 'a>,
    field: Field,
    /// The actual [`Rect`]s on screen
    field_rects: Vec<Vec<Rect>>,
    current_player: Sign,
    has_won: bool,
}

const BACKGROUND_COLOR: Color = Color::RGB(69, 69, 69);
const FIELD_SIZE: usize = 3;

fn main() {
    let (mut canvas, mut event_pump, ttf_context) = setup_sdl();

    // Setup GameState
    let font = ttf_context.load_font("assets/ComicSansMS3.ttf", 69).expect("Loading font");
    let mut game_state = GameState {
        font,
        field: Field::empty(FIELD_SIZE),
        field_rects: vec![],
        current_player: get_random_player(),
        has_won: false,
    };

    // Game Loop
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    game_state = reset_game(game_state);
                }
                Event::MouseButtonDown { y, x, .. } if !game_state.has_won => on_mouse_clicked(x, y, &mut game_state),
                _ => {}
            }
        }

        update(&mut canvas, &mut game_state).expect("Failed updating the game");
    }
}

/// Updates the game and draws to the window.
fn update(canvas: &mut WindowCanvas, game_state: &mut GameState) -> Result<()> {
    canvas.clear();
    let field = game_state.field.clone();
    let texture_creator = canvas.texture_creator();
    let window_size = canvas.window().size();

    field.draw(game_state, canvas).context("Drawing game Field")?;

    // Check for win or draw
    if check_win(&game_state.field, &!game_state.current_player) {
        game_state.has_won = true;

        let mut text = "Player ".to_owned();
        text.push_str((!game_state.current_player).into());
        text.push_str(" has won!");

        draw_end_text(&*text, (window_size.0 / 2, window_size.1 / 8), &game_state.font, &texture_creator, canvas)?;
    } else if check_draw(&game_state.field) {
        draw_end_text("It is a tie!", (window_size.0 / 4, window_size.1 / 8), &game_state.font, &texture_creator, canvas)?;
    }

    canvas.present();

    Ok(())
}

/// Setup everything that has to do with SDL2.
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

/// Resets the [`GameState`].
fn reset_game(game_state: GameState) -> GameState {
    GameState {
        font: game_state.font,
        field: Field::empty(FIELD_SIZE),
        field_rects: vec![],
        current_player: get_random_player(),
        has_won: false,
    }
}

/// Checks if the game has ended in a win for the given `player`.
fn check_win(field: &Field, player: &Sign) -> bool {
    let mut field = field.clone();

    // Rows
    if check_win_rows(&field, player) {
        return true;
    }

    // Diagonals
    if let Some(middle_sign) = field.0[1][1].0 {
        // Left-Top to Right-Bottom
        if &middle_sign == player && field.0[0][0] == field.0[1][1] && field.0[1][1] == field.0[2][2] {
            return true;
        }

        if &middle_sign == player && field.0[0][2] == field.0[1][1] && field.0[1][1] == field.0[2][0] {
            return true;
        }
    }

    // Cols
    field = rotate_field_90deg(&field);
    if check_win_rows(&field, player) {
        return true;
    }

    false
}

/// Checks if the game has ended in a draw.
fn check_draw(field: &Field) -> bool {
    for row in field.0.iter() {
        for cell in row {
            if cell.0.is_none() {
                return false;
            }
        }
    }

    true
}

/// Checks if the `field` contains a row with three of the same [`Sign`]s.
fn check_win_rows(field: &Field, player: &Sign) -> bool {
    if field.0
        .windows(FIELD_SIZE)
        .any(|row| row.contains(&vec![Cell(Some(*player)); FIELD_SIZE])) {
        return true;
    }

    false
}

/// # Return
/// 
/// a random [`Sign`] to use as the player.
fn get_random_player() -> Sign {
    let player = rand::thread_rng().gen_range(0_u32..=1_u32);
    match player {
        0 => Sign::O,
        1 => Sign::X,
        _ => unreachable!()
    }
}

/// Rotates the field by 90 degrees clockwise.
///
/// (at this time I am not so smart that I could do this so I "borrowed" it from:
/// [qiwei9743 on Leetcode](https://leetcode.com/problems/rotate-image/solutions/435653/rust-with-std::mem::swap-in-2D-vector))
fn rotate_field_90deg(field: &Field) -> Field {
    let mut field = field.clone();

    field.0.reverse();
    for i in 1..field.0.len() {
        let (left, right) = field.0.split_at_mut(i);
        for (j, left_item) in left.iter_mut().enumerate().take(i) {
            std::mem::swap(&mut left_item[i], &mut right[0][j]);
        }
    }

    field
}

/// Draws the text at the end of the game, when the game ends in a tie, win or lose.
fn draw_end_text<'a>(
    text: impl Into<&'a str>,
    text_w_h: (u32, u32),
    font: &'a Font,
    texture_creator: &'a TextureCreator<WindowContext>,
    canvas: &mut WindowCanvas,
) -> Result<()> {
    let window_size = canvas.window().size();
    let text_width = text_w_h.0;
    let text_height = text_w_h.1;

    let texture = get_text_texture(text, font, texture_creator).context("Creating texture for player Sign.")?;

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

/// Switches the `current_player`.
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
