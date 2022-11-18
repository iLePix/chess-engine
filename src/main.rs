pub mod figures;
pub mod board;
pub mod macros;

use board::Board;
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
                /*Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: true, .. } => {
                    camera.offset.x += offset_increment;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: true, .. } => {
                    camera.offset.x -= offset_increment;
                },
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: true, .. } => {
                    camera.offset.y += offset_increment;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: true,.. } => {
                    println!("go");
                    camera.offset.y -= offset_increment;
                },
                Event::KeyDown { keycode: Some(Keycode::Plus), repeat: true, .. } => {
                    camera.zoom_in();
                },
                Event::KeyDown { keycode: Some(Keycode::Minus), repeat: false, .. } => {
                    camera.zoom_out();
                },
                Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    println!("stop");
                },*/
                _ => {}
            }
        }

        let x = inputs.mouse_pos.x / 50;
        let y = inputs.mouse_pos.y / 50;
        board.select((x + y*8) as u8);
    
    

        board.draw(&mut canvas);

        
        canvas.present();


        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}


