pub mod figures;
pub mod board;
pub mod macros;

use board::Board;
use figures::Figure;
use input::InputHandler;
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::sys::Window;
use vecm::vec::{Vec2i, Vec2u, Vec2};

use std::path::Path;
//use world::celo::Celo;
use std::time::Duration;

mod input; 


use crate::input::Control;


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem.window("Chess", 400, 400)
        //.resizable()
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas: Canvas<sdl2::video::Window> = window.into_canvas().build()
        .expect("could not make a canvas");
    
    let mut screen_size = Vec2u::new(640, 640);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut event_pump = sdl_context.event_pump()?;
    let mut inputs = InputHandler::new();

    let chess_pieces = &Path::new("../../res/chess_pieces.png");
    let texture_creator = canvas.texture_creator();
    let pieces_texture = texture_creator.load_texture(chess_pieces)?;
    let mut board = Board::new(&pieces_texture);
    let mut selected_fig: Option<Figure> = None;
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::MouseMotion {x, y, ..} => {
                    inputs.mouse_pos.x = x as u32;
                    inputs.mouse_pos.y = y as u32;
                },
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::SizeChanged(w, h) => {screen_size.x = w as u32; screen_size.y = h as u32},
                    sdl2::event::WindowEvent::Resized(w, h) => {screen_size.x = w as u32; screen_size.y = h as u32},
                    _ => (),
                },
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keycode {
                        inputs.set_key(key, true);
                    }
                },
                Event::KeyUp{ keycode, .. } => {
                    if let Some(key) = keycode {
                        inputs.set_key(key, false);
                    }
                },
                Event::MouseButtonDown{ mouse_btn, ..} => {
                    inputs.mouse_down(mouse_btn)
                },
                Event::MouseButtonUp{ mouse_btn, ..} => {
                    inputs.mouse_up(mouse_btn)
                },
                _ => {}
            }
        }

        let sx = inputs.mouse_pos.x / 50;
        let sy = inputs.mouse_pos.y / 50;
        let i = (sx + sy*8) as u8;
        board.hover(i);

        if inputs.pressed(Control::Escape) {
            board.unselect();
            selected_fig = None;
        }


        if inputs.left_click {
            selected_fig = board.select(i);
        }
        


        board.draw(&mut canvas);

        if selected_fig.is_some() {
            canvas.set_draw_color(Color::RGB(255, 0, 0));
            canvas.fill_rect(Rect::from_center(Point::new(inputs.mouse_pos.x as i32 , inputs.mouse_pos.y as i32), 30, 30));
        }

        
        canvas.present();


        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}


