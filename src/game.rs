use std::{collections::{HashMap, HashSet}, thread::JoinHandle, net::TcpStream, sync::mpsc::{Receiver, self}};

use vecm::vec::Vec2i;

use crate::{pieces::{Piece, Side}, board::{Board, FenError}};
use crate::Move;



pub struct Game {
    pub captured_pieces: Vec<Piece>,
    pub board: Board,
    pub game_state: GameState,
    pub possible_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    pub turn: Side,
    white: Player,
    black: Player,
    flipped: bool
}

impl Game {
    pub fn new(white: Player, black: Player, flipped: bool) -> Self {
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

    pub fn from_fen(white: Player, black: Player, fen: &str, flipped: bool) -> Result<Self, FenError> {
        let (board, turn) = Board::from_fen(fen)?;
        Ok(Self {
            captured_pieces: Vec::new(),
            board,
            game_state: GameState::Running,
            possible_moves: HashMap::new(),
            turn,
            white,
            black,
            flipped
        })  
    }

    pub fn change_turn(&mut self) {
        self.turn = match self.turn {
            Side::Black => Side::White,
            Side::White => Side::Black,
        };
    }
}


pub enum GameState {
    Running,
    Winner(Side),
    Draw
}

pub enum Player {
    Me,
    Remote(Remote),
    Cpu {
        depth: usize,
        computation: Option<JoinHandle<(Vec2i, Vec2i)>>,
    }
}


 
pub struct Remote {
    pub socket: TcpStream,
    pub rx: Receiver<Move>,
}