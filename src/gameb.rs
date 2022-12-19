use std::collections::HashMap;

use vecm::vec::PolyVec2;

use crate::{game::{PlayerType, GameState, Remote}, pieces::Side, boardb::{BoardB, Piece, Pos, PosTrait}, dtos::{self, Move}, board::FenError};




pub struct GameB {
    pub state: GameState,
    pub board: BoardB,
    pub possible_moves: HashMap<u8, u64>,
    pub captured_pieces: Vec<Piece>,
    pub turn: Side,
    pub black: PlayerType,
    pub white: PlayerType,
    pub check: (bool, bool),
    flipped: bool,
    pub last_move: Option<(u8, u8)>,
}

impl GameB {
    pub fn new(white: PlayerType, black: PlayerType, flipped: bool) -> Self {
        let board = BoardB::with_starting_pos();
        let mut mvs = HashMap::with_capacity(16);
        board.valid_moves(Side::White, &mut mvs);
        let check = (false, false);
        Self {
            state: GameState::Running,
            board,
            possible_moves: mvs,
            captured_pieces: Vec::new(),
            turn: Side::White,
            white,
            black,
            check,
            last_move: None,
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

    pub fn vcpu(depth: usize) -> Self {
        let cpu1 = PlayerType::Cpu {depth};
        let cpu2 = PlayerType::Cpu {depth};
        Self::new(cpu1, cpu2, false)
    }

    pub fn from_fen(white: PlayerType, black: PlayerType, fen: &str, flipped: bool) -> Result<Self, FenError> {
        let (board, turn) = BoardB::from_fen(fen)?;
        let mut mvs = HashMap::with_capacity(16);
        board.valid_moves(turn, &mut mvs);
        let white_check = board.is_check_from_hm(&mvs, Side::White);
        let black_check =board.is_check_from_hm(&mvs, Side::Black);
        Ok(Self {
            captured_pieces: Vec::new(),
            board,
            state: GameState::Running,
            possible_moves: mvs,
            white,
            black,
            check: (white_check, black_check),
            turn,
            flipped,
            last_move: None,
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
    pub fn turn_mut(&mut self) -> &mut PlayerType {
        match self.turn {
            Side::Black => &mut self.black,
            Side::White => &mut self.white,
        }
    }

    pub fn make_move(&mut self, from: u8, to: u8) {
        let moves_for_pieces = self.possible_moves.get(&from).unwrap();
        if moves_for_pieces & (1 << to) != 0 {
            if let Some(captured_piece) = self.board.make_move(from, to) {
                self.captured_pieces.push(captured_piece);
            }
            self.last_move = Some((from, to));
            if let PlayerType::Remote(remote) = &mut self.turn_mut() {
                let f = Pos::from_i(from);
                let t = Pos::from_i(to);
                dtos::send(&mut remote.socket, Move {x1: f.x as i8, y1: f.y as i8, x2: t.x as i8, y2: t.y as i8})
                    .expect("Failed to send move")
            };
            self.change_turn();
            self.board.valid_moves(self.turn, &mut self.possible_moves);

            let check = match self.turn {Side::White => self.check.0, Side::Black => self.check.1};
            if self.possible_moves.iter().filter(|(_, v)| **v > 0).count() == 0 {
                if check {
                    self.state = GameState::Winner(!self.turn);
                } else {
                    self.state = GameState::Draw;
                }
            } // & check then its a draw 


            match self.turn {
                Side::Black => self.check.1 = self.board.is_check(&self.board.valid_moves_as_array(Side::White, false, false), Side::Black),
                Side::White => self.check.0 =  self.board.is_check(&self.board.valid_moves_as_array(Side::Black, false, false), Side::White),
            }
        }
    }
}
