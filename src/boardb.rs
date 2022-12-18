use std::{cmp::{min, max}, slice::Iter, collections::HashMap};

use vecm::vec::Vec2i;

use crate::{pieces::{Side, PieceType}, pos, castle::{Castle, self}, board::FenError};




#[derive(Clone, Copy)]
pub struct BoardB {
    pub board: [Option<Piece>; 64],
    //.0 = long, .1 = short
    white_castle: Castle,
    black_castle: Castle,
    kings: (u8, u8),
    //pos_index of possible en passant, if > 63, not possible
    en_passant: u8
}

impl BoardB {    

    pub fn with_starting_pos() -> Self {
        let mut board = [None; 64];

        for x in 0..=7 {
            board[pos!(x, 1)] = Some(Piece::new(PieceType::Pawn, Side::Black));
            board[pos!(x, 6)] = Some(Piece::new(PieceType::Pawn, Side::White));
        }
    
        let first_rank = {
            use PieceType::*;
            [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook]
        };
    
        for (x, piece_ty) in first_rank.iter().enumerate() {
            board[pos!(x, 0)]  = Some(Piece::new(*piece_ty, Side::Black));
            board[pos!(x, 7)]  = Some(Piece::new(*piece_ty, Side::White))
        }
    
        let castle = Castle::new();
        let kings = (3, 60);

        Self {
            board,
            white_castle: castle,
            black_castle: castle,
            kings,
            en_passant: 64
        }
    }

    pub fn occupied(&self, i: u8) -> bool {
        self.board.get(i as usize).is_some()
    }

    pub fn remove_piece(&mut self, i: u8) {
        self.board[i as usize] = None;
    }
    //may be faster if positions are switched
    pub fn set_piece(&mut self, i: u8, piece: Piece) {
        self.board[i as usize] = Some(piece);
    }

    fn valid_moves_for_piece(&self, i: u8) -> u64 {
        let mut valid_moves = 0;
        let piece = self.get_piece_at_pos(i).expect("Tried to calculate moves for non-existing piece");

        let add_pos = |pos: u8, valid_mvs: &mut u64| {
            *valid_mvs ^= 1 << pos
        };

        let moves_in_dir = |dir: i8, valid_mvs: &mut u64| {
            let mut new_pos = i as i8 + dir;
            while self.is_on_board(new_pos) {
                if let Some(other_piece) = self.get_piece_at_pos(new_pos as u8) {
                    if other_piece.side != piece.side {
                        add_pos(new_pos as u8, valid_mvs);
                    }
                    break;
                } else {
                    add_pos(new_pos as u8, valid_mvs);
                }
                new_pos += dir
            }
        };

        let rook = |valid_mvs: &mut u64| {
            moves_in_dir(pos![0,1], valid_mvs);
            moves_in_dir(pos![-1,0], valid_mvs);
            moves_in_dir(pos![0,1], valid_mvs);
            moves_in_dir(pos![0,-1], valid_mvs);
        };

        let bishop = |valid_mvs: &mut u64| {
            moves_in_dir(pos![1,1], valid_mvs);
            moves_in_dir(pos![-1,1], valid_mvs);
            moves_in_dir(pos![1,-1], valid_mvs);
            moves_in_dir(pos![1,-1], valid_mvs);
        };

        let queen = |valid_mvs: &mut u64| {
            bishop(valid_mvs);
            rook(valid_mvs);
        };

        let knight = |valid_mvs: &mut u64| {
            let possible_moves = [
                pos!(-1, 2),
                pos!(1, 2),

                pos!(-1, -2),
                pos!(1, -2),

                pos!(2, 1),
                pos!(2, -1),

                pos!(-2, 1),
                pos!(-2, -1),
            ];
            for mv in possible_moves {
                let new_pos = i as i8 + mv;
                if self.is_on_board(new_pos) {
                    match self.get_piece_at_pos(new_pos as u8) {
                        Some(other_piece) => if other_piece.side == piece.side {continue;},
                        None => {}
                    }
                    add_pos(new_pos as u8, valid_mvs);
                }
            }
        };

        let king = |valid_mvs: &mut u64| {
            for x in -1..=1 {
                for y in -1..=1 {
                    let new_pos = i as i8 + pos!(x, y);
                    if self.is_on_board(new_pos) {
                        if let Some(other_piece) = self.get_piece_at_pos(new_pos as u8) {
                            if other_piece.side == piece.side {
                                continue;
                            }
                        }
                        add_pos(new_pos as u8, valid_mvs);
                    }
                }
            }
        };

        let pawn = |valid_mvs: &mut u64| {
            match piece.side {
                Side::Black => add_pos(i + 8, valid_mvs),
                Side::White => add_pos(i - 8, valid_mvs),
            }
        };

        match piece.ty {
            PieceType::Queen => queen(&mut valid_moves),
            PieceType::King => king(&mut valid_moves),
            PieceType::Knight => knight(&mut valid_moves),
            PieceType::Bishop => bishop(&mut valid_moves),
            PieceType::Rook => rook(&mut valid_moves),
            PieceType::Pawn => pawn(&mut valid_moves),
        }
        valid_moves
    }    

    pub fn valid_moves(&self, side: Side, mvs: &mut HashMap<u8, u64>) {
        self.board.iter()
            .enumerate()
            .filter_map(|(i,p)| p.map_or(false, |p| p.side == side).then_some(i as u8))
            .for_each(|i| { mvs.insert(i, self.valid_moves_for_piece(i)); });
    }

    pub fn find_king(&self, side: Side) -> u8 {
        match side {
            Side::Black => self.kings.1,
            Side::White => self.kings.0,
        }
    }

    //Returns captured piece
    pub fn make_move(&mut self, from: u8, to: u8) -> Option<Piece> {
        let captured_piece = self.get_piece_at_pos(to);
        let piece = self.get_piece_at_pos(from).expect("Tried to move non existing piece");
        self.remove_piece(from);
        self.set_piece(to, piece);
        captured_piece
    }


    //inclusive
    fn space_occupied(&self, from: u8, to: u8) -> bool {
        if from > 63 || to > 63 {
            panic!("Space out of bounds: from: {}, to: to {}", from, to);
        }
        for i in min(from, to)..=max(from, to) {
            if self.occupied(i) {
                return true
            }
        }
        false
    }

    fn is_on_board(&self, i: i8) -> bool {
        i >= 0 && i < 64
    }


    pub fn get_piece_at_pos(&self, i: u8) -> Option<Piece> {
        match self.board.get(i as usize) {
            Some(opt_piece) => *opt_piece,
            None => None,
        }
    }

    pub fn from_fen(fen: &str) -> Result<(Self, Side), FenError> {
        let mut cursor = Vec2i::zero();
        let mut board = [None; 64];
        let mut kings = (0,0);
        let mut sections = fen.split(' ');
        let mut piece_type = |c: char, side, pos| -> Option<PieceType> {
            use PieceType::*;
            match c {
                'p' => Some(Pawn),
                'r' => Some(Rook),
                'n' => Some(Knight),
                'b' => Some(Bishop),
                'q' => Some(Queen),
                'k' => {
                    match side {
                        Side::Black => kings.1 = pos,
                        Side::White => kings.0 = pos,
                    }
                    Some(King)
                },
                e => panic!("Undefined character in fen: {}", e)
            }
        };

        fn pos(s: &str) -> Result<u8, FenError> {
            let a = s.chars().next().ok_or(FenError::EnPassant)?;
            let b = s.chars().next().ok_or(FenError::EnPassant)?;
            if s.chars().next().is_some() || !('a'..='h').contains(&a) || !('1'..='8').contains(&a) {
                return Err(FenError::EnPassant);
            }
            Ok(pos!((a as u8 - b'a'), (b as u8 - b'1')))
        }

        let pieces = sections.next().ok_or(FenError::Pieces)?;

        for c in pieces.chars() {
            match c {
                '/' => {cursor.y += 1; cursor.x = 0},
                '0'..='8' => cursor.x += (c as u8 - b'0') as i32,
                'a'..='z' | 'A'..='Z' => {
                    if cursor.y > 7 {
                        return Err(FenError::Cursor);
                    }
                    use Side::*;
                    let side = if c.is_lowercase() {Black} else {White};
                    board[pos!(cursor.x as usize, cursor.y as usize)] = Some(
                        Piece::new(
                            piece_type(c.to_ascii_lowercase(), side, pos!(cursor.x, cursor.y) as u8).ok_or(FenError::Pieces)?,
                            side
                        )
                    );
                    cursor.x += 1;
                },
                _ => return Err(FenError::Pieces)
            }
        }

        let turn = match sections.next().ok_or(FenError::Turn)? {
            "b" => Side::Black,
            "w" => Side::White,
            _ => return Err(FenError::Turn)
        };


        let (white_castle, black_castle) = Castle::from_fen(sections.next().ok_or(FenError::MissingSection(1))?)?;

        let en_passant_possible = match sections.next().ok_or(FenError::EnPassant)?
         {
            "-" => 64,
            s => pos(s)?
        };

        let _halfmoves: &str = sections.next().ok_or(FenError::HalfMoves)?;
        let _fullmoves: &str = sections.next().ok_or(FenError::FullMoves)?;

        if sections.next().is_some() { return Err(FenError::Cursor)}


        Ok((    
            Self {
                board,
                white_castle,
                black_castle,
                kings,
                en_passant: en_passant_possible
            },
            turn
        ))
    }


    pub fn evaluate(&self, side: Side) -> i32 {
        self.board.into_iter()
            .filter_map(|p| p )
            .map(|p| {
                let s = if side == p.side {1} else {-1};
                p.value() * s
            })
            .sum()
    }

    pub fn print_board(&self) {
        for y in 0..8 {
            let mut row_string = String::new();
            for x in 0..8 {
                row_string += " ";
                match self.get_piece_at_pos(pos![x,y]) {
                    Some(piece) => {
                        row_string += match piece.side {
                            Side::Black => "b",
                            Side::White => "w",
                        };
                        row_string += match piece.ty {
                            PieceType::Queen => "q",
                            PieceType::King => "k",
                            PieceType::Knight => "n",
                            PieceType::Bishop => "b",
                            PieceType::Rook => "r",
                            PieceType::Pawn => "p",
                        }
                    },
                    None => row_string += "--",
                };
                row_string += " ";
            }
            println!("{}", row_string)
        }
    }


}



const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;


//piece = u8  | Dont care bits -> XXXX1111
//first relevant bit = side /  white = 0, black = 1
#[derive(Clone, Copy)]
pub struct Piece {
    pub ty: PieceType,
    pub side: Side
}

impl Piece {

    pub fn new(ty: PieceType, side: Side) -> Self {
        Self {ty, side}
    }


    pub fn value(&self) -> i32 {
        match self.ty {
            PieceType::Queen => QUEEN_VALUE,
            PieceType::King => 0,
            PieceType::Knight => KNIGHT_VALUE,
            PieceType::Bishop => BISHOP_VALUE,
            PieceType::Rook => ROOK_VALUE,
            PieceType::Pawn => PAWN_VALUE,
        }
    }
}