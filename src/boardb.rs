use std::{cmp::{min, max}, slice::Iter, collections::HashMap};

use vecm::vec::{Vec2i, PolyVec2};

use crate::{pieces::{Side, PieceType}, pos, castle::{Castle, self}, board::FenError};




#[derive(Clone, Copy)]
pub struct BoardB {
    pub board: [Option<Piece>; 64],
    //.0 = long, .1 = short
    white_castle: Castle,
    black_castle: Castle,
    pub kings: (u8, u8),
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
        let kings = (60, 4);

        Self {
            board,
            white_castle: castle,
            black_castle: castle,
            kings,
            en_passant: 64
        }
    }

    pub fn occupied(&self, i: u8) -> bool {
        self.board.get(i as usize).unwrap().is_some()
    }

    pub fn remove_piece(&mut self, i: u8) {
        self.board[i as usize] = None;
    }
    //may be faster if positions are switched
    pub fn set_piece(&mut self, i: u8, piece: Piece) {
        self.board[i as usize] = Some(piece);
    }

    fn valid_moves_for_piece(&self, i: u8, from_castling_check: bool) -> u64 {
        let mut valid_moves = 0;
        let piece = self.get_piece_at_pos(i).expect("Tried to calculate moves for non-existing piece");
        let piece_pos = Pos::from_i(i);


        let add_pos = |pos: u8, valid_mvs: &mut u64| {
            *valid_mvs ^= 1 << pos;
        };

        let moves_in_dir = |dir: Pos, valid_mvs: &mut u64| {
            let mut new_pos = Pos::from_i(i) + dir;
            while self.xy_on_board(new_pos) {
                if let Some(other_piece) = self.get_piece_at_pos(new_pos.to_i()) {
                    if other_piece.side != piece.side {
                        add_pos(new_pos.to_i(), valid_mvs);
                    }
                    break;
                } else {
                    add_pos(new_pos.to_i(), valid_mvs);
                }
                new_pos += dir
            }
        };

        let rook = |valid_mvs: &mut u64| {
            moves_in_dir(Pos::new(1,0), valid_mvs);
            moves_in_dir(Pos::new(-1,0), valid_mvs);
            moves_in_dir(Pos::new(0,1), valid_mvs);
            moves_in_dir(Pos::new(0,-1), valid_mvs);
        };

        let bishop = |valid_mvs: &mut u64| {
            moves_in_dir(Pos::new(1,1), valid_mvs);
            moves_in_dir(Pos::new(1,-1), valid_mvs);
            moves_in_dir(Pos::new(-1,1), valid_mvs);
            moves_in_dir(Pos::new(-1,-1), valid_mvs);
        };

        let queen = |valid_mvs: &mut u64| {
            bishop(valid_mvs);
            rook(valid_mvs);
        };

        let knight = |valid_mvs: &mut u64| {
            let possible_moves = [
                Pos::new(-1, 2),
                Pos::new(1, 2),

                Pos::new(-1, -2),
                Pos::new(1, -2),

                Pos::new(2, 1),
                Pos::new(2, -1),

                Pos::new(-2, 1),
                Pos::new(-2, -1),
            ];
            for mv in possible_moves {
                let new_pos = Pos::from_i(i) + mv;
                if self.xy_on_board(new_pos) {
                    match self.get_piece_at_pos(new_pos.to_i()) {
                        Some(other_piece) => if other_piece.side == piece.side {continue},
                        None => {}
                    }
                    add_pos(new_pos.to_i(), valid_mvs);
                }
            }
        };

        let king = |valid_mvs: &mut u64| {
            for x in -1..=1 {
                for y in -1..=1 {
                    let new_pos = piece_pos + Pos::new(x, y);
                    if self.xy_on_board(new_pos) {
                        if let Some(other_piece) = self.get_piece_at_pos(new_pos.to_i()) {
                            if other_piece.side == piece.side {
                                continue;
                            }
                        }
                        add_pos(new_pos.to_i(), valid_mvs);
                    }
                }
            }

            let mut castle = |y: i8, castle: Castle| {
                //for long: x = 2,3 for short: x = 5,6 / check for check
                let mvs = self.valid_moves_as_array(!piece.side, true, true);
                if 
                    castle.long && 
                    !self.space_occupied(Pos::new(1,y), Pos::new(3,y)) && 
                    (2..=4).all(|x| !self.threathens(Pos::new(x,y).to_i(), &mvs))
                {
                    add_pos(Pos::new(2,y).to_i(), valid_mvs);
                }
                if  castle.short && 
                    !self.space_occupied(Pos::new(5,y), Pos::new(6,y)) &&
                    (4..=6).all(|x| !self.threathens(Pos::new(x,y).to_i(), &mvs))
                {
                    add_pos(Pos::new(6,y).to_i(), valid_mvs);
                }
            };

            if !from_castling_check {
                match piece.side {
                    Side::Black => {
                        castle(0, self.black_castle);
                    },
                    Side::White => {
                        castle(7, self.white_castle);
                    },
                }
            }
        };

        let pawn = |valid_mvs: &mut u64| {
            let y_dir = match piece.side {
                Side::Black => 1,
                Side::White => -1
            };
            let mut ys = vec![y_dir];
            match piece.side {
                Side::Black => if piece_pos.y == 1 {ys.push(y_dir*2)},
                Side::White => if piece_pos.y == 6 {ys.push(y_dir*2)},
            }
            for y in ys {
                let new_pos = piece_pos + Pos::new(0, y);
                if self.xy_on_board(new_pos) && !self.space_occupied(piece_pos + Pos::new(0,y_dir), new_pos) {
                    add_pos(new_pos.to_i(), valid_mvs)
                }
            };
            for x_dir in [-1, 1] {
                let new_pos = piece_pos + Pos::new(x_dir, y_dir);
                if self.xy_on_board(new_pos) {
                    if let Some(other_piece) = self.get_piece_at_pos(new_pos.to_i()) {
                        if other_piece.side != piece.side {
                            add_pos(new_pos.to_i(), valid_mvs)
                        }
                    }
                } 
            };
            //detect en passant
            if self.en_passant < 64 {
                let en_passant_pos = Pos::from_i(self.en_passant);
                if (en_passant_pos.x - piece_pos.x).abs() == 1 && en_passant_pos.y == piece_pos.y {
                    add_pos((self.en_passant as i8 + 8 * y_dir) as u8, valid_mvs)
                }
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
    pub fn i_to_xy(&self, i: u8) -> Pos {
        let x = i % 8;
        let y = i / 8 ;
        Pos::new(x as i8, y as i8)
    }

    fn pieces_indexes_of_side(&self, side: Side) -> Vec<u8> {
        self.board.iter()
            .enumerate()
            .filter_map(|(i,p)| p.map_or(false, |p| p.side == side).then_some(i as u8)).collect()
    }

    pub fn valid_moves(&self, side: Side, mvs: &mut HashMap<u8, u64>) {
        mvs.clear();
        for i in self.pieces_indexes_of_side(side) {
            let mut mv_map = self.valid_moves_for_piece(i, false);
            for to in mv_map.ones() {
                if self.is_check_after(i, to, side) {
                    mv_map ^= 1 << to;
                }
            }
            mvs.insert(i, mv_map);
        }
    }


    pub fn valid_moves_as_array(&self, side: Side, from_castling_check: bool, from_check_test: bool) -> [u64; 16] {
        let mut mvs = [0; 16];
        let mut n = 0;
        for i in self.pieces_indexes_of_side(side) {
            let mut mv_map = self.valid_moves_for_piece(i, from_castling_check);
            if !from_check_test {
                for to in mv_map.ones() {
                    if self.is_check_after(i, to, side) {
                        mv_map ^= 1 << to;
                    }
                }
            }
            mvs[n] = mv_map; 
            n += 1;
        }
        mvs
    }

    pub fn find_king(&self, side: Side) -> u8 {
        match side {
            Side::Black => self.kings.1,
            Side::White => self.kings.0,
        }
    }

    fn try_castle(&mut self, y: i8, castle: Castle, dst: u8) {
        if let Some(rook) = self.get_piece_at_pos(pos!(0,y as u8)) && dst == pos!(2, y as u8) && castle.long {
            self.set_piece(pos!(3,y as u8), rook);
            self.remove_piece(pos!(0,y as u8));
        } else if let Some(rook) = self.get_piece_at_pos(pos!(7,y as u8)) && dst == pos!(6, y as u8) && castle.short {
            self.set_piece(pos!(5,y as u8), rook);
            self.remove_piece(pos!(7,y as u8));
        }
    }

    //Returns captured piece
    pub fn make_move(&mut self, from: u8, to: u8) -> Option<Piece> {
        let captured_piece = self.get_piece_at_pos(to);
        let mut piece = self.get_piece_at_pos(from).expect("Tried to move non existing piece");
        match piece.ty {
            PieceType::King => {
                match piece.side {
                    Side::Black => {
                        self.try_castle(0, self.black_castle, to);
                        self.black_castle = Castle::forbid();
                        self.kings.1 = to;
                    },
                    Side::White => {
                        self.try_castle(7, self.white_castle, to);
                        self.white_castle = Castle::forbid();
                        self.kings.0 = to;
                    },
                }
            },
            PieceType::Rook => {
                let rook = |y: i8, castle: &mut Castle| {
                    if from == pos!(0,y as u8) {
                        castle.long = false
                    } else if from == pos!(7,y as u8){
                        castle.short = false
                    }
                };
                match piece.side {
                    Side::Black => rook(0, &mut self.black_castle),
                    Side::White => rook(7, &mut self.white_castle)
                }
            },
            PieceType::Pawn => {
                let y_dir: i8 = match piece.side {
                    Side::Black => -1,
                    Side::White => 1,
                };
                let to_xy = self.i_to_xy(to);
                let from_xy = self.i_to_xy(from);

                if self.en_passant < 64 {
                    if self.en_passant as i8 == to as i8 + y_dir * 8 {
                        self.remove_piece(self.en_passant);
                    }
                }
                if (from_xy.y - to_xy.y).abs() > 1 {
                    self.en_passant = to;
                }
                if to_xy.y == 0 || to_xy.y == 7 {
                    piece = Piece::new(PieceType::Queen, piece.side);
                }
            },
            _ => {self.en_passant = 64}
        }
        self.remove_piece(from);
        self.set_piece(to, piece);
        captured_piece
    }

    //inclusive
    fn space_occupied(&self, from: Pos, to: Pos) -> bool {
        for x in min(from.x, to.x)..=max(from.x, to.x) {
            for y in min(from.y, to.y)..=max(from.y, to.y) {
                if self.occupied(Pos::new(x as i8, y as i8).to_i()) {
                    return true
                }
            }
        }
        false
    }


    pub fn is_check(&self, mvs_for_piece: &[u64; 16], side: Side) -> bool {
        self.threathens(self.find_king(side), mvs_for_piece)
    }

    //used for check indicator
    pub fn is_check_from_hm(&self, mvs_for_piece: &HashMap<u8, u64>, side: Side) -> bool {
        let victim = 1_u64 << self.find_king(side);
        for (_, piece_mvs) in mvs_for_piece {
            if victim & piece_mvs > 0 {
                return true
            }
        }
        false
    }

    fn threathens(&self, pos: u8, mvs_for_piece: &[u64; 16]) -> bool{
        let victim = 1_u64 << pos;
        for piece_mvs in mvs_for_piece {
            if victim & piece_mvs > 0 {
                return true
            }
        }
        false
    }

    pub fn is_check_after(&self, from: u8, to: u8, side: Side) -> bool {
        let mut board = *self;
        board.make_move(from, to);
        board.is_check(&board.valid_moves_as_array(!side, false, true), side)
    }

    fn xy_on_board(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < 8 && pos.y >= 0 && pos.y < 8
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

pub type Pos = PolyVec2<i8>;

impl PosTrait for Pos {
    fn from_i(i: u8) -> Self {
        let x = i % 8;
        let y = i / 8 ;
        Self::new(x as i8, y as i8)
    }

    fn to_i(self) -> u8 {
        (self.x + self.y*8) as u8
    }
}

pub trait PosTrait {
    fn from_i(i: u8) -> Self;
    fn to_i(self) -> u8;
}

impl BitMap for u64 {
    fn ones(self) -> Vec<u8> {
        let mut ones = Vec::new();
        for n in 0..63 {
            if self >> n & 1 == 1 {
                ones.push(n);
            }
        }
        ones
    }
}

pub trait BitMap {
    fn ones(self) -> Vec<u8>;
}