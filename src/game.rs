use std::{collections::{HashMap, HashSet}, thread::JoinHandle, net::TcpStream, sync::mpsc::{Receiver, self, TryRecvError}};

use vecm::vec::Vec2i;

use crate::{pieces::{Piece, Side}, board::{Board, FenError}, dtos};
use crate::Move;



pub struct Game {
    pub captured_pieces: Vec<Piece>,
    pub board: Board,
    pub game_state: GameState,
    pub possible_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    pub turn: Side,
    pub black: PlayerType,
    pub white: PlayerType,
    flipped: bool
}

impl Game {
    pub fn new(white: PlayerType, black: PlayerType, flipped: bool) -> Self {
        Self {
            captured_pieces: Vec::new(),
            board: Board::gen_starting_pos(),
            game_state: GameState::Running,
            possible_moves: HashMap::new(),
            turn: Side::White,
            white,
            black,
            flipped
        }
    }

    pub fn versus() -> Self {
        Self::new(PlayerType::Me, PlayerType::Me, false)
    }

    pub fn remote(remote: Remote, flipped: bool) -> Self {
        let me = PlayerType::Me;
        let rm = PlayerType::Remote(remote);
        let (white, black) = if flipped {(rm, me)} else {(me, rm)};
        Self::new(white, black, flipped)
    }

    pub fn cpu(depth: usize, is_white: bool) -> Self {
        let cpu = PlayerType::Cpu {depth};
        let me = PlayerType::Me;
        let (white, black) = if is_white {(cpu, me)} else {(me,cpu)};
        Self::new(white, black, is_white)
    }

    pub fn from_fen(white: PlayerType, black: PlayerType, fen: &str, flipped: bool) -> Result<Self, FenError> {
        let (board, turn) = Board::from_fen(fen)?;
        Ok(Self {
            captured_pieces: Vec::new(),
            board,
            game_state: GameState::Running,
            possible_moves: HashMap::new(),
            white,
            black,
            turn,
            flipped
        })  
    }

    pub fn change_turn(&mut self) {
        self.turn = match self.turn {
            Side::Black => Side::White,
            Side::White => Side::Black,
        };
    }

    pub fn turn(&self) -> &PlayerType {
        match self.turn {
            Side::Black => &self.black,
            Side::White => &self.white,
        }
    }

    pub fn make_move(&mut self, from: Vec2i, to: Vec2i) -> bool {
        match self.board.make_move(&from, &to, self.turn) {
            Ok(game_state) => {
                self.game_state = game_state; 
                self.change_turn();
                true
            },
            Err(_) => false,
        }
    }

}

#[derive(Clone, Copy)]
pub enum GameState {
    Running,
    Winner(Side),
    Draw
}

pub enum PlayerType {
    Me,
    Remote(Remote),
    Cpu {
        depth: usize,
        //computation: Option<JoinHandle<(Vec2i, Vec2i)>>,
    }
}


impl PlayerType {
    pub fn is_me(&self) -> bool{
        match self {
            PlayerType::Me => true,
            PlayerType::Remote(_) => false,
            PlayerType::Cpu { depth } => false,
        }
    }

    pub fn is_remote(&self) -> bool{
        match self {
            PlayerType::Me => false,
            PlayerType::Remote(_) => true,
            PlayerType::Cpu { depth } => false,
        }
    }

    pub fn is_ai(&self) -> bool{
        match self {
            PlayerType::Me => false,
            PlayerType::Remote(_) => false,
            PlayerType::Cpu { depth } => true,
        }
    }
}


pub struct Remote {
    pub socket: TcpStream,
    pub rx: Receiver<Move>,
}

impl Remote {
    pub fn new(socket: TcpStream, rx: Receiver<Move>) -> Self {
        Self {socket, rx }
    }

    pub fn send_move(&mut self, from: Vec2i, to: Vec2i) {
        dtos::send(
            &mut self.socket, 
            Move {
                x1: from.x as i8,
                y1: 7 - from.y as i8,
                x2:  to.x as i8,
                y2: 7 - to.y as i8
            }
        ).expect("Could not send move");
    }
}