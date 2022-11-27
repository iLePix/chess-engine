use vecm::vec::PolyVec2;

use crate::{game::{PlayerType, GameState, Remote}, pieces::Side, boardc::{BoardC, Piece}, dtos::{self, Move}};




pub struct GameC {
    pub state: GameState,
    pub board: BoardC,
    pub possible_moves: [u64; 16],
    pub captured_pieces: Vec<Piece>,
    pub turn: Side,
    pub black: PlayerType,
    pub white: PlayerType,
    flipped: bool,
    pub last_move: Option<(u8, u8)>,
}

impl GameC {
    pub fn new(white: PlayerType, black: PlayerType, flipped: bool) -> Self {
        Self {
            state: GameState::Running,
            board: BoardC::gen_starting_pos(),
            possible_moves: [0; 16],
            captured_pieces: Vec::new(),
            turn: Side::White,
            white,
            black,
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

    /*pub fn from_fen(white: PlayerType, black: PlayerType, fen: &str, flipped: bool) -> Result<Self, FenError> {
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
    }*/

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
        //let piece_index = self.board.get_piece_index(from);
        //let moves_for_pieces = self.possible_moves[from as usize];
        if true { //moves_for_pieces & (1 << to) != 0 { //checks if move is in possible moves
            if let Some(captured_piece) = self.board.make_move(from, to) {
                self.captured_pieces.push(captured_piece);
            }
            self.last_move = Some((from, to));
            if let PlayerType::Remote(remote) = &mut self.turn_mut() {
                let f = i_to_xy(from);
                let t = i_to_xy(to);
                dtos::send(&mut remote.socket, Move {x1: f.x as i8, y1: f.y as i8, x2: t.x as i8, y2: t.y as i8})
                    .expect("Failed to send move")
            };
            self.change_turn();
            self.possible_moves = self.board.valid_moves(self.turn);
        }
    }

}

type Pos = PolyVec2<i8>;

pub fn i_to_xy(i: u8) -> Pos {
    let x = i % 8;
    let y = i / 8;
    Pos::new(x as i8, y as i8)
}