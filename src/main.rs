#![feature(let_chains)]
#![feature(hash_drain_filter)]
pub mod pieces;
pub mod macros;
pub mod atlas;
pub mod renderer;
pub mod dtos;
pub mod color_themes;
pub mod game_renderer;
pub mod boardb;
pub mod gameb;
pub mod castle;

use atlas::TextureAtlas;
use binverse::error::BinverseError;
use game_renderer::GameRenderer;
use dtos::{PlayerInfo, Move, GameInfo};
use gameb::PlayerType;
use pieces::Side;
use input::InputHandler;
use renderer::Renderer;
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::{pixels::Color, render::Canvas};
use vecm::vec::Vec2u;

use std::collections::HashMap;
use std::env::Args;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::{self, TryRecvError, Sender};
use std::thread::JoinHandle;
//use world::celo::Celo;
use std::time::Instant;
use rand::Rng;

mod input; 


use crate::boardb::{Pos, PosTrait, BoardB, BitMap};
use crate::gameb::{GameB, Remote, GameState};
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

fn parse_args(args: &mut Args) -> (bool, bool, Option<usize>, Option<usize>, Option<String>, Option<String>){
    args.skip(1);
    let mut versus = true;
    let mut server = false;
    let mut ai = None;
    let mut ip = None;
    let mut fen = None;
    let mut vai = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-s" | "--server" => panic!("Not available at the moment"),//server = true,
            "-a" | "--ai" => ai = Some(
                args.next()
                    .expect("give ai depth as argument")
                    .parse::<usize>()
                    .expect("depth has to be a positive integer")
                ),
            "-v" | "--vai" => vai = Some(
                args.next()
                    .expect("give ai depth as argument")
                    .parse::<usize>()
                    .expect("depth has to be a positive integer")
                ),
            "-f" | "--fen" => fen = Some(args.next().expect("fen expected after -f/--fen")),
            "-c" | "--c" => ip = Some(args.next().expect("connect requires ip")), 
            _ => eprintln!("unrecognized arg {arg}"),
        }
    }
    (versus, server, ai, vai, ip, fen)
} 

#[derive(Debug)]
enum ConnectionError {
    Playername,
    IPParse,
    Send,
    Receive
}

fn connect(ip: String) -> Result<MultiplayerUtils, ConnectionError> {
    println!("Type in your name: ");
    let mut player_name = String::new();
    std::io::stdin().read_line(&mut player_name).or(Err(ConnectionError::Playername))?;
    player_name = player_name.trim().to_owned();
    println!("Waiting for opponent");
    let mut tcp_stream = TcpStream::connect(ip).or(Err(ConnectionError::IPParse))?;
    dtos::send(&mut tcp_stream, PlayerInfo { name: player_name }).or(Err(ConnectionError::Send))?;
    let game_info: GameInfo = dtos::recv(&mut tcp_stream).or(Err(ConnectionError::Receive))?;
    let my_side = if game_info.is_black { Side::Black } else { Side::White };
    println!("Your Enemy has connected: {}", game_info.other_player);
    println!("Your are: {}", my_side);

    
    let tcp_stream_clone = tcp_stream.try_clone().unwrap();
    let (sender, rx) = mpsc::channel();

    std::thread::spawn(|| receive_mvs(tcp_stream_clone, sender));
    Ok(MultiplayerUtils { tcp_stream, moves_rx: rx, my_side})
}

fn try_apply_remote_move(game: &mut GameB) {
    if let PlayerType::Remote(remote) = &game.turn() {
        match remote.rx.try_recv() {
            Ok(new_move) => {
                println!("Receiving move {:?} for {:?}", new_move, game.turn);
                if !game.make_move(Pos::new(new_move.x1, 7 - new_move.y1).to_i(), Pos::new(new_move.x2, 7 - new_move.y2).to_i()) {
                    panic!("Opponent move not accepted");
                }
                game.change_turn();
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Disconnected"),
        }
    }
}


fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let (versus, server, ai, vai, ip, fen) = parse_args(&mut args);
    let mut mp = false;

    let mut gameb = GameB::versus();


    if let Some(ip)  = ip {
        let mp_utils = match connect(ip) {
            Ok(utils) => {mp = true; utils},
            Err(err) => panic!("Error connecting: {:?}", err)
        };
        gameb = GameB::remote(
            Remote::new(mp_utils.tcp_stream, mp_utils.moves_rx), 
            match mp_utils.my_side {
                Side::Black => true,
                Side::White => false,
            }
        );
    }

    if let Some(depth) = ai {
        let mut rng = rand::thread_rng();
        let is_white: bool = rng.gen();
        gameb = GameB::cpu(depth, is_white)
    }

    if let Some(depth) = vai {
        gameb = GameB::vcpu(depth)
    }

    if let Some(fen) = fen {
       match BoardB::from_fen(&fen) {
        Ok((b,t)) => {gameb.board = b; gameb.turn = t},
        Err(err) => println!("Fen error: {:?}", err),
        }
    }



    let font_path = &Path::new("../../res/IBMPlexSerif-Medium.ttf");
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);


    let window = video_subsystem.window("Chess", 720, 720)
        //.resizable()
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas: Canvas<sdl2::video::Window> = window.into_canvas().build()
        .expect("could not make a canvas");
    
    let mut screen_size = Vec2u::new(720, 720);
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut event_pump = sdl_context.event_pump()?;
    let mut inputs = InputHandler::new();

    let chess_pieces = &Path::new("../../res/chess_pieces.png");
    let texture_creator = canvas.texture_creator();
    let pieces_texture = texture_creator.load_texture(chess_pieces)?;
    let tex_atlas = TextureAtlas::new(&pieces_texture, 90);

    let field_size = 90;
    let board_size = Vec2u::fill(8);

    let mut color_lifted = true;
    let mut pieces_lifted = true;

    let mut renderer = Renderer::new(&tex_atlas, &mut canvas);
    let mut game_renderer = GameRenderer::new(field_size, board_size, 100.0);
    let (progress_sender, progress_rx) = mpsc::channel();



    let mut last_frame_time = Instant::now();



    fn spawn_move_computer(board: BoardB, depth: usize, turn: Side, progress_sender: Sender<f32>) -> JoinHandle<(u8, u8)> { 
        std::thread::spawn(move || {
            let mut next_moves_by_piece = HashMap::with_capacity(16);
            board.valid_moves(turn, &mut next_moves_by_piece);
            let mut best_move = ((0,0), i32::MIN);
            let mvs = next_moves_by_piece.iter().flat_map(|(from, tos)| tos.ones().iter().map(|to| (*from, *to)).collect::<Vec<_>>()).collect::<Vec<(u8, u8)>>();
            let total = mvs.len();
            let mut progress = 0;
            for (from, to) in mvs {
                progress += 1;
                progress_sender.send(progress as f32 / total as f32).unwrap();
                let mut b = board;
                b.make_move(from, to);
                let eval = compute_best_move(b, depth - 1, !turn, turn, i32::MIN, i32::MAX);
                if eval > best_move.1 {
                    best_move = ((from, to), eval);
                }
            }
            best_move.0
        })
    }




    fn compute_best_move(board: BoardB, depth: usize, turn: Side, max: Side, mut alpha: i32, mut beta: i32) -> i32 { 
        let mut next_moves_by_piece = HashMap::with_capacity(16);
        board.valid_moves(turn, &mut next_moves_by_piece);
        if next_moves_by_piece.len() == 0 || depth == 0 {
            return board.evaluate(turn);
        }
        let mut t_eval = if turn == max {i32::MIN} else {i32::MAX};
        let mvs = next_moves_by_piece.iter().flat_map(|(from, tos)| tos.ones().iter().map(|to| (*from, *to)).collect::<Vec<_>>()).collect::<Vec<(u8, u8)>>();
        for (from, to) in mvs {
            let mut b = board;
            b.make_move(from, to);
            let eval = compute_best_move(b, depth - 1, !turn, max, alpha, beta);
            if turn == max {
                t_eval = t_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            } else {
                t_eval = t_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
        }  
        return t_eval;
    }

    let mut next_move_option: Option<JoinHandle<(u8, u8)>> = None;

    'running: loop {
        let current_frame_time = Instant::now();
        let dt = (current_frame_time - last_frame_time).as_secs_f32();
        last_frame_time = current_frame_time;

        inputs.handle_events(&mut event_pump);
        if inputs.quit {
            break 'running;
        }

        
        let cursor_field_xy = inputs.mouse_pos / field_size;
        let cursor_field = pos!(cursor_field_xy.x,cursor_field_xy.y) as u8;

        //colortheme
        if inputs.pressed(Control::Color) && color_lifted {
            game_renderer.next_theme();
        }
        if inputs.pressed(Control::Pieces) && pieces_lifted {
            tex_atlas.next_theme();
        }
        if inputs.pressed(Control::Escape) {
            game_renderer.unselect();
        }
        color_lifted = !inputs.pressed(Control::Color);
        pieces_lifted = !inputs.pressed(Control::Pieces);

        if inputs.left_click {
            if let Some(selected) = game_renderer.selected && gameb.turn().is_me() {
                gameb.make_move(selected, cursor_field);
                game_renderer.unselect();
            } else {
                game_renderer.select(cursor_field, gameb.turn, &gameb.board);
            }
        }

        game_renderer.update_mouse_pos(inputs.mouse_pos);
        game_renderer.render(&gameb, &mut renderer, dt);
        renderer.render();

        if gameb.state == GameState::Running {
            match gameb.turn() {
                PlayerType::Remote(_) => try_apply_remote_move(&mut gameb),
                PlayerType::Cpu { depth } => {
                    match progress_rx.try_recv() {
                        Ok(progress) => {
                            match gameb.turn {
                                Side::Black => {game_renderer.ai_progess.1 = Some(progress); game_renderer.ai_progess.0 = None},
                                Side::White => {game_renderer.ai_progess.0 = Some(progress); game_renderer.ai_progess.1 = None},
                            }
                        },
                        Err(TryRecvError::Empty) => {},
                        Err(TryRecvError::Disconnected) => {},
                    }
                    if let Some(next_move) = &next_move_option {
                        if next_move.is_finished() {
                            let mv = next_move_option.take().unwrap().join().expect("Thread couldnt be joined");
                            gameb.make_move(mv.0, mv.1);
                        }
                    } else {
                        next_move_option = Some(spawn_move_computer(gameb.board, *depth, gameb.turn, progress_sender.clone()));
                    }
                },
                _ => {}
            }
        }

        use sdl2::mouse::MouseButton::*;
        inputs.mouse_up(Left);
        inputs.mouse_up(Right);

    }

    Ok(())
}
