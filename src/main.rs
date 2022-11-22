pub mod pieces;
pub mod board;
pub mod macros;
pub mod atlas;
pub mod renderer;

use atlas::TextureAtlas;
use board::BoardRenderer;
use pieces::{Piece, Side};
use input::InputHandler;
use renderer::Renderer;
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, Vec2, VecInto};

use std::path::Path;
//use world::celo::Celo;
use std::time::{Duration, Instant};

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
    
    let mut screen_size = Vec2u::new(400, 400);
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut event_pump = sdl_context.event_pump()?;
    let mut inputs = InputHandler::new();

    let chess_pieces = &Path::new("../../res/chess_pieces.png");
    let texture_creator = canvas.texture_creator();
    let pieces_texture = texture_creator.load_texture(chess_pieces)?;
    let tex_atlas = TextureAtlas::new(&pieces_texture, 90);
    //let mut board = Board2::new(&tex_atlas);
    let field_size = 50;
    let mut board_renderer = BoardRenderer::new(Vec2u::fill(8));
    let mut renderer = Renderer::new(&tex_atlas, 200.0, &mut canvas);


    let mut turn = Side::White;

    'running: loop {
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

        let cursor_field = (inputs.mouse_pos / field_size).vec_into();
        board_renderer.hover(cursor_field);
        board_renderer.render(&mut renderer);
        renderer.render();
        /*board.hover(i);

        if inputs.pressed(Control::Escape) {
            board.unselect();
        }


        if inputs.left_click {
            if board.selected.is_none() {
                board.select(i, turn);
            } else if board.move_figure(i) {
                turn = match turn {
                    Side::Black => Side::White,
                    Side::White => Side::Black,
                }
            }
        }
        



        board.draw(&mut canvas, turn, dt);

        //promoting(screen_size, Side::White, &mut canvas, &tex_atlas);

        if let Some(f) = board.get_selected_fig() {
            if s_tick + s_tick_increment*dt >= 255.0 {
                s_tick = 0.0;
            } else {
                s_tick += dt * s_tick_increment as f32;
            }

            let p = (parabola(s_tick as i32) / 20.0);
            let size = 50 + p as u32;
            let src = tex_atlas.figure_atlas_cords.get(&f.tex_id)
                    .unwrap_or_else(|| panic!("Created figure with wrong tex-index {}", f.tex_id));
            let dst = Rect::from_center(Point::new(inputs.mouse_pos.x as i32 , inputs.mouse_pos.y as i32), size, size);
            canvas.copy(tex_atlas.pieces_texture, *src, dst).unwrap();
          } else {
            s_tick = 0.0;
        }*/

    
        
        use sdl2::mouse::MouseButton::*;
        inputs.mouse_up(Left);
        inputs.mouse_up(Right);

    }

    Ok(())
}



fn parabola(x: i32) -> f32 {
    -1.0 * (0.125 * x as f32 - 16.0).powi(2) + 256.0
}
/*

fn promoting(window_size: Vec2u, side: Side, canvas: &mut Canvas<Window>, tex_atlas: &TextureAtlas) {
    let center = Point::new((window_size.x / 2) as i32 , (window_size.y / 2) as i32);
    let r = Rect::from_center(center, 120, 120);
    if side == Side::White {
        canvas.set_draw_color(Color::RGB(0,0,0));
    } else {
        canvas.set_draw_color(Color::RGB(255,255,255));
    }
    canvas.fill_rect(r).unwrap();

    match side {
        Side::White => {
            for i in 1..=4 {
                let mut x  = i * 60 + (window_size.x as i32 / 2) - 120;
                let mut y = (window_size.y  as i32/ 2) - 60;
                if i > 2 {
                    y += 60;
                    x -= 120;
                }

                let size = 60;
                let src = tex_atlas.figure_atlas_cords.get(&(i * 2)).unwrap_or_else(|| panic!("Couldnt find tex for promoting tex_id {}",  i));
                let dst = Rect::new(x, y, size, size);
                canvas.copy(tex_atlas.pieces_texture, *src, dst).unwrap();

            }
        },
        Side::Black => {
            for i in 1..=4 {
                let mut x  = i * 60 + (window_size.x as i32 / 2) - 120;
                let mut y = (window_size.y  as i32/ 2) - 60;
                if i > 2 {
                    y += 60;
                    x -= 120;
                }

                let size = 60;
                let src = tex_atlas.figure_atlas_cords.get(&((i * 2) + 1)).unwrap_or_else(|| panic!("Couldnt find tex for promoting tex_id {}",  i));
                let dst = Rect::new(x, y, size, size);
                canvas.copy(tex_atlas.pieces_texture, *src, dst).unwrap();

            }
        }
    }
}*/