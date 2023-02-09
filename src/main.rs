use anyhow::Result;
use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

/// [`Sign`] to represent the players.
#[derive(Copy, Clone)]
enum Sign {
    X,
    O,
}

impl ToString for Sign {
    fn to_string(&self) -> String {
        match self {
            Sign::X => "X".to_string(),
            Sign::O => "O".to_string(),
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
    fn draw(&self, canvas: &mut WindowCanvas) {
        let window_size = canvas.window().size();
        let cell_size = window_size.0 / 5;
        let padding = cell_size / 4;

        for (row_idx, row) in self.0.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                //TODO: display Sign
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
    }
}

struct GameState {
    field: Field,
}

const BACKGROUND_COLOR: Color = Color::RGB(69, 69, 69);

fn main() {
    let (mut canvas, mut event_pump) = setup_sdl();
    let mut game_state = GameState {
        field: Field::empty(3),
    };

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

    game_state.field.draw(canvas);

    canvas.present();
    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

    Ok(())
}

/// Setup everything that has to do with SDL2
fn setup_sdl() -> (WindowCanvas, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("tic_tac_toe_rs", 600, 600)
        .position_centered()
        .resizable()
        .build()
        .expect("Window creation went terribly wrong!");

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump().unwrap();

    (canvas, event_pump)
}
