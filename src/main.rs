use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::render::WindowCanvas;

fn main() {
    let (mut canvas, mut event_pump) = setup_sdl();
    
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
        
        update(&mut canvas);
    }
}

fn update(canvas: &mut WindowCanvas) {
    canvas.clear();

    

    canvas.present();
    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
}

fn setup_sdl() -> (WindowCanvas, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("tic_tac_toe_rs", 600, 600)
        .position_centered()
        .build()
        .expect("Window creation went terribly wrong!");

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(69, 69, 69));
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump().unwrap();

    (canvas, event_pump)
}
