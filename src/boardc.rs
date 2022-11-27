use std::{cmp::{min, max}, slice::Iter};

use crate::{pieces::{Side, PieceType}, pos};




#[derive(Clone, Copy)]
pub struct BoardC {
    occupied: u64,
    pub pieces: [u8; 16],
    //.0 = long, .1 = short
    white_castle: (bool, bool),
    black_castle: (bool, bool),
    //pos_index of possible en passant, if > 63, not possible
    en_passant: u8
}

impl BoardC {    

    pub fn gen_starting_pos() -> Self {
        let occupied = 0b1111111111111111000000000000000000000000000000001111111111111111u64;
        let mut pieces = [255; 16];

        let b_pawn = Piece::from_ty_n_side(PieceType::Pawn, Side::Black);
        let b_queen = Piece::from_ty_n_side(PieceType::Queen, Side::Black);
        let b_king = Piece::from_ty_n_side(PieceType::King, Side::Black);
        let b_bishop = Piece::from_ty_n_side(PieceType::Bishop, Side::Black);
        let b_knight = Piece::from_ty_n_side(PieceType::Knight, Side::Black);
        let b_rook = Piece::from_ty_n_side(PieceType::Rook, Side::Black);

        fn combine_pieces(first: Piece, second: Piece) -> u8 {
            (first << 4) | second 
        }

        let b_second_rank = [b_rook, b_knight, b_bishop, b_king, b_queen, b_bishop, b_knight, b_rook];
        let w_second_rank = b_second_rank.map(|p| p.to_white());

        //black 
        for (i, pieces_slice) in b_second_rank.chunks(2).enumerate() {
            pieces[i] = combine_pieces(pieces_slice[0], pieces_slice[1])
        }
        for i in 4..=7 {
            pieces[i] = combine_pieces(b_pawn, b_pawn)
        }

        let w_pawn = b_pawn.to_white();
        for i in 8..=11 {
            pieces[i] = combine_pieces(w_pawn, w_pawn)
        }

        for (i, pieces_slice) in w_second_rank.chunks(2).enumerate() {
            pieces[i + 12] = combine_pieces(pieces_slice[0], pieces_slice[1])
        }

        let white_castle = (true, true);
        let black_castle = (true, true);
        let en_passant = 0b11111111u8;

        Self {
            occupied,
            pieces,
            white_castle,
            black_castle,
            en_passant
        }
    }

    pub fn remove_piece(&mut self, i: u8) {
        if self.occupied(i) {
            self.occupied ^= (1 as u64) << i;
            let pieces_left = self.occupied.count_ones();
            let piece_index = self.get_piece_index(i);
            let n = 1 - (i % 2);
            let no_piece = 0b00001111u8 << (4 * n);

            let num_of_pieces = 32;
            let a_shift = (num_of_pieces - piece_index) * 4;
            let b_shift = (piece_index+1) * 4;


            self.pieces = if piece_index == 0 {
                (u128::from_be_bytes(self.pieces) << 4).to_be_bytes()
            } else {

                let mut a = u128::from_be_bytes(self.pieces) >> a_shift;
                let mut b = u128::from_be_bytes(self.pieces) << b_shift;
                a <<= a_shift;
                b >>= piece_index * 4;
                (a | b).to_be_bytes()
            }
        } 
    }
    //may be faster if positions are switched
    pub fn set_piece(&mut self, i: u8, piece: Piece) {
        if self.is_on_board(i as i8) && self.occupied.count_ones() < 32 {
            let piece_index = self.get_piece_index(i);
            self.occupied ^= (1 as u64) << i;
            let prev = self.pieces[(piece_index / 2) as usize] << (piece_index % 2);
            let n = 1 - (piece_index % 2);
            self.pieces[(piece_index / 2) as usize] = (piece << (4 * n)) & prev;
        }
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
                    if other_piece.side() != piece.side() {
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
                        Some(other_piece) => if other_piece.side() == piece.side() {continue;},
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
                            if other_piece.side() == piece.side() {
                                continue;
                            }
                        }
                        add_pos(new_pos as u8, valid_mvs);
                    }
                }
            }
        };

        let pawn = |valid_mvs: &mut u64| {
            bishop(valid_mvs);
            rook(valid_mvs);
        };


        match piece.ty().expect("Tried to calculate moves for none piece") {
            PieceType::Queen => queen(&mut valid_moves),
            PieceType::King => king(&mut valid_moves),
            PieceType::Knight => knight(&mut valid_moves),
            PieceType::Bishop => bishop(&mut valid_moves),
            PieceType::Rook => rook(&mut valid_moves),
            PieceType::Pawn => pawn(&mut valid_moves),
        }
        valid_moves
    }    


    pub fn valid_moves(&self, side: Side) -> [u64; 16] {
        let mut mvs = [0; 16];
        let mut i = 0;
        for n in 0..=63 {
            if let Some(piece) = self.get_piece_at_pos(n) && piece.side() == side {
                mvs[i] = self.valid_moves_for_piece(n);
                i += 1;
            }
        }
        mvs
    }

    pub fn find_king(&self, side: Side) -> Option<u8> {
        for (i, piece) in self.pieces_uncrompressed().iter().enumerate() {
            if let Some(ty) = piece.ty() && ty == PieceType::King && piece.side() == side {
                return self.get_piece_at_pos(i as u8);
            }
        }
        None
    }

    fn pieces_uncrompressed(&self) ->  [Piece; 32] {
        let mut pieces = [0 as Piece; 32]; 
        for (i, two_pieces) in self.pieces.iter().enumerate() {
            let first = two_pieces >> 4;
            let second = 0b00001111u8 & two_pieces;
            pieces[i*2] = first;
            pieces[(i*2) + 1] = second;
        }
        pieces
    } 

    //Return captured piece
    pub fn make_move(&mut self, from: u8, to: u8) -> Option<Piece> {
        let captured_piece = self.get_piece_at_pos(to);
        let piece = self.get_piece_at_pos(from).expect("Tried to move non existing piece");
        self.set_piece(piece, to);
        self.remove_piece(from);
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


    fn occupied(&self, i: u8) -> bool {
        i < 64 && (self.occupied & (1 << i)) > 0
    }


    //counts ones in self.occupied before index
    fn get_piece_index(&self, i: u8) -> u8 {
        let mut count = 0;
        for n in 0..i {
            count += self.occupied >> n & 1
        }
        count as u8
    }

    //returns pos in occupied of the i's number
    //i = piece_index, find the (i+1) 1 
    fn get_piece_pos(&self, i: u8) -> u8 {
        let mut count = 0;
        for n in 0..=63 {
            count += self.occupied >> n & 1;
            if count == (i+1) as u64 {
                return n
            }
        }
        //this should never happen
        panic!("A thing that should never have happend, happend");
        count as u8
    }

    pub fn get_piece_at_pos(&self, i: u8) -> Option<Piece> {
        if self.occupied(i) { 
             let piece_index = self.get_piece_index(i);
             let n = 1 - (piece_index % 2);
             let piece: Piece = self.pieces[(piece_index / 2) as usize] >> (4 * n);
             //println!("{}: {:?} {} - {:#08b}", i, piece.ty().unwrap(), piece.side(), piece);
             if piece.ty().is_some() {
                return Some(piece)
             } else {
                return None
             }
        }
        return None //0b00001111u8 PieceType == None
    }


    pub fn evaluate(&self, side: Side) -> i32 {

        fn v(piece: Piece, side: Side) -> i32 {
            let value = piece.value();
            if piece.side() == side {
                return value
            }
            return -(value)
        }
        let mut material = 0;
        let left_pieces = self.occupied.count_ones();
        let last = (left_pieces - 1) / 2;
        for (i, two_pieces) in self.pieces[0..=last as usize].iter().enumerate() {
            let first_piece = two_pieces >> 4;
            let second_piece = 0b00001111u8 & two_pieces;
            material += v(first_piece, side);
            if i != last as usize || (left_pieces % 2) == 0 {
                material += v(second_piece, side);
            }   
        }
        material
    }

    pub fn print_board(&self) {
        for y in 0..8 {
            let mut row_string = String::new();
            for x in 0..8 {
                row_string += " ";
                match self.get_piece_at_pos(pos![x,y]) {
                    Some(piece) => {
                        row_string += match piece.side() {
                            Side::Black => "b",
                            Side::White => "w",
                        };
                        row_string += match piece.ty() {
                            Some(ty) => {
                                match ty {
                                    PieceType::Queen => "q",
                                    PieceType::King => "k",
                                    PieceType::Knight => "n",
                                    PieceType::Bishop => "b",
                                    PieceType::Rook => "r",
                                    PieceType::Pawn => "p",
                                }
                            },
                            None => "ERROR",
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
pub type Piece = u8;

impl PieceTrait for Piece {
    fn side(&self) -> Side {
        if ((self >> 3) & 1) == 1 {
            Side::Black
        } else {
            Side::White
        }
    }

    fn ty(&self) -> Option<PieceType> {
        let n = 0b00000111;
        match n & self {
            0 => Some(PieceType::Queen),
            1 => Some(PieceType::King),
            2 => Some(PieceType::Knight),
            3 => Some(PieceType::Bishop),
            4 => Some(PieceType::Rook),
            5 => Some(PieceType::Pawn),
            _ => None
        }
    }

    fn from_ty_n_side(ty: PieceType, side: Side) -> Piece {
        let src = match side {
            Side::Black => 0b00001000u8,
            Side::White => 0b00000000u8,
        };
        src | (ty as u8)
    }

    fn to_white(&self) -> Piece {
        self ^ 0b00001000u8
    }

    fn to_black(&self) -> Piece {
       self | 0b00001000u8
    }

    fn value(&self) -> i32 {
        let n = 0b00000111u8;
        match n & self {
            0 => QUEEN_VALUE,
            2 => KNIGHT_VALUE,
            3 => BISHOP_VALUE,
            4 => ROOK_VALUE,
            5 => PAWN_VALUE,
            _ => 0
        }
    }
}

pub trait PieceTrait {
    fn side(&self) -> Side;
    fn ty(&self) -> Option<PieceType>;
    fn from_ty_n_side(ty: PieceType, side: Side) -> Piece;
    fn to_black(&self) -> Piece;
    fn to_white(&self) -> Piece;
    fn value(&self) -> i32;
}