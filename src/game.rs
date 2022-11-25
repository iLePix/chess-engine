use std::{collections::{HashMap, HashSet}, thread::JoinHandle, net::TcpStream, sync::mpsc::{Receiver, self}};

use vecm::vec::Vec2i;

use crate::{pieces::{Piece, Side}, board::{Board, FenError}};
use crate::Move;



pub struct Game {
    pub captured_pieces: Vec<Piece>,
    pub board: Board,
    pub game_state: GameState,
    pub possible_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    pub ty: GameType,
    pub turn: Side,
    flipped: bool
}

impl Game {
    pub fn new(ty: GameType, flipped: bool) -> Self {
        Self {
            captured_pieces: Vec::new(),
            board: Board::gen_starting_pos(),
            game_state: GameState::Running,
            possible_moves: HashMap::new(),
            turn: Side::White,
            ty,
            flipped
        }
    }

    pub fn versus() -> Self {
        Self::new(GameType::Versus, false)
    }

    pub fn remote(remote: Remote, flipped: bool) -> Self {
        Self::new(GameType::Remote(remote), flipped)
    }

    pub fn cpu(depth: usize, flipped: bool) -> Self {
        Self::new(GameType::Cpu {depth}, flipped)
    }

    pub fn from_fen(ty: GameType, fen: &str, flipped: bool) -> Result<Self, FenError> {
        let (board, turn) = Board::from_fen(fen)?;
        Ok(Self {
            captured_pieces: Vec::new(),
            board,
            game_state: GameState::Running,
            possible_moves: HashMap::new(),
            ty,
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

    pub fn is_my_turn(&self) -> bool {
        if self.ty.is_versus() {
            return true;
        }
        match self.turn {
            Side::Black => self.flipped,
            Side::White => !self.flipped,
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

pub enum GameType {
    Versus,
    Remote(Remote),
    Cpu {
        depth: usize,
        //computation: Option<JoinHandle<(Vec2i, Vec2i)>>,
    }
}


impl GameType {
    pub fn is_versus(&self) -> bool{
        match self {
            GameType::Versus => true,
            GameType::Remote(_) => false,
            GameType::Cpu { depth } => false,
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
}