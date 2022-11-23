#![feature(let_chains)]
#![feature(hash_drain_filter)]
pub mod pieces;
pub mod board;
pub mod macros;
pub mod atlas;
pub mod renderer;
pub mod board_renderer;
pub mod dtos;

use atlas::TextureAtlas;
use binverse::error::BinverseError;
use board::{Board, ColorTheme, gen_starting_pos, gen_kings_pos};
use board_renderer::BoardRenderer;
use dtos::{PlayerInfo, Move, GameInfo};
use pieces::{Piece, Side};
use input::InputHandler;
use renderer::Renderer;
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, Vec2, VecInto};

use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::{self, TryRecvError};
//use world::celo::Celo;
use std::time::{Duration, Instant};

mod input; 


use crate::input::Control;

fn receive_mvs(mut tcp_stream: TcpStream, moves: mpsc::Sender<Move>) -> Result<(), BinverseError> {
    loop {
        moves.send(dtos::recv(&mut tcp_stream)?).unwrap();
    }
}

struct MultiplayerUtils {
    tcp_stream: TcpStream,
    moves_rx: mpsc::Receiver<Move>,
    my_side: Side,
}


fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut player_name = String::new();
    
    let mut mp_utils = args.next().map(|ip|  {
        println!("Type in your name: ");
        std::io::stdin().read_line(&mut player_name).expect("Failed to read playername");
        player_name = player_name.trim().to_owned();
        println!("Waiting for opponent");
        let mut tcp_stream = TcpStream::connect(ip).expect("Couldnt connect");
        dtos::send(&mut tcp_stream, PlayerInfo { name: player_name }).expect("Couldnt sent player_info");
        let game_info: GameInfo = dtos::recv(&mut tcp_stream).expect("Didnt get player_info");
        let my_side = if game_info.is_black { Side::Black } else { Side::White };
        println!("Your Enemy has connected: {}", game_info.other_player);

        
        let tcp_stream_clone = tcp_stream.try_clone().unwrap();
        let (sender, rx) = mpsc::channel();
    
        std::thread::spawn(|| receive_mvs(tcp_stream_clone, sender));
        MultiplayerUtils { tcp_stream, moves_rx: rx, my_side}
    });

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
    let board_size = Vec2u::fill(8);
    let color_theme = ColorTheme {
        board_primary: Color::WHITE,
        board_secondary: Color::RGB(13,56,166),
        valid_moves: Color::RGBA(3, 138, 255, 128),
        selection: Color::RGBA(255, 123, 98, 200),
        last_move: Color::RGB(199,232,172),
        check: Color::RGB(230,55,96)
    };
    let mut renderer = Renderer::new(&tex_atlas, 200.0, &mut canvas);
    let mut turn = Side::White;
    let mut board = gen_starting_pos();
    board.calculate_valid_moves(turn);
    let mut board_renderer = BoardRenderer::new(field_size, color_theme, board_size, 2.0);



    let mut last_frame_time = Instant::now();
    let mut s_tick = 0.0;
    let s_tick_increment = 200.0;


    'running: loop {
        let current_frame_time = Instant::now();
        let dt = (current_frame_time - last_frame_time).as_secs_f32();
        last_frame_time = current_frame_time;


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
  

        if inputs.pressed(Control::Escape) {
            board_renderer.unselect();
        }

        let mut change_turn = |turn: &mut Side| {
            *turn = match turn {
                Side::Black => Side::White,
                Side::White => Side::Black,
            };
        };


        /*if inputs.left_click {
            if board_renderer.selected.is_none() {
                board_renderer.select(cursor_field, turn, &board);
            } else if let Some(selected) = board_renderer.selected && board.make_move(&selected, &cursor_field, turn) {
                board_renderer.unselect();
                change_turn(&mut turn);
            } else if cursor_field == board_renderer.selected.unwrap() {
                board_renderer.unselect();
            }
        }*/



        

        
        if let Some(mp_utils) = &mut mp_utils {
            if inputs.left_click {
                if board_renderer.selected.is_none() {
                    board_renderer.select(cursor_field, mp_utils.my_side, &board);
                } else if let Some(selected) = board_renderer.selected && mp_utils.my_side == turn && board.make_move(&selected, &cursor_field, turn) {
                    //broadcast move
                    dtos::send(
                        &mut mp_utils.tcp_stream, 
                        Move {
                            x1: selected.x as i8,
                            y1: 7 - selected.y as i8,
                            x2:  cursor_field.x as i8,
                            y2: 7 - cursor_field.y as i8
                        }
                    ).expect("Could not send move");
                    board_renderer.unselect();
                    change_turn(&mut turn);
                } else if cursor_field == board_renderer.selected.unwrap() {
                    board_renderer.unselect();
                }
            }
            if mp_utils.my_side != turn {
                match mp_utils.moves_rx.try_recv() {
                    Ok(new_move) => {
                        println!("Receiving move {:?} for {:?}", new_move, turn);
                        if !board.make_move(&Vec2i::new(new_move.x1 as i32, 7 - new_move.y1 as i32), &Vec2i::new(new_move.x2 as i32, 7 - new_move.y2 as i32), turn) {
                            panic!("Opponent move not accepted");
                        }
                        change_turn(&mut turn);
                    },
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => panic!("Disconnected"),
                }
            }
            board_renderer.hover(cursor_field);
            board_renderer.render(&mp_utils.my_side, &board, &mut renderer, dt);

        } else {
            if inputs.left_click {
                if board_renderer.selected.is_none() {
                    board_renderer.select(cursor_field, turn, &board);
                } else if board.make_move(&board_renderer.selected.unwrap(), &cursor_field, turn) {
                    board_renderer.unselect();
                    change_turn(&mut turn);
                } else if cursor_field == board_renderer.selected.unwrap() {
                    board_renderer.unselect();
                }
            }
            board_renderer.hover(cursor_field);
            board_renderer.render(&turn, &board, &mut renderer, dt);
        }


        if let Some(piece) = board_renderer.get_selected_piece(&board) {
            if s_tick + s_tick_increment*dt >= 255.0 {
                s_tick = 0.0;
            } else {
                s_tick += dt * s_tick_increment as f32;
            }
            let p = (parabola(s_tick as i32) / 20.0);
            let size = field_size + p as u32;
            let dst = Rect::from_center(Point::new(inputs.mouse_pos.x as i32 , inputs.mouse_pos.y as i32), size, size);
            renderer.draw_image(piece.ty, piece.side, dst, 3);
        } else {
            s_tick = 0.0;
        }



        renderer.render();


        use sdl2::mouse::MouseButton::*;
        inputs.mouse_up(Left);
        inputs.mouse_up(Right);

    }

    Ok(())
}

enum GameState {
    Running,
    Winner(Side),
    Draw
}


fn parabola(x: i32) -> f32 {
    -1.0 * (0.125 * x as f32 - 16.0).powi(2) + 256.0
}
