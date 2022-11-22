use std::{collections::{HashMap, HashSet}, ops::{Range, RangeInclusive}, time::Instant};

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, VecInto};


use crate::{pieces::{Piece, Side, PieceType}, hashmap, count, atlas::TextureAtlas, renderer::Renderer};


#[derive(Clone, Copy)]
struct Castle {
    short: bool,
    long: bool
}

impl Castle {
    fn new() -> Self {
        Self {short: true, long: true}
    }
    fn forbid() -> Self {
        Self {short: false, long: false}
    }
}


#[derive(Clone)]
pub struct Board {
    pub board: Vec<Vec<Option<Piece>>>,//[[Option<Piece>; 8]; 8],
    pub size: Vec2u,
    //0 = White, 1, Black
    pawn_start_y: (u32, u32),
    pub valid_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    //0 = long, 1 = short
    white_castle: Castle,
    black_castle: Castle,
    en_passant_possible: Option<Vec2i>,
    pub check: (bool, bool),
    captured_pieces: Vec<Piece>,
    pub last_move: Option<(Vec2i, Vec2i)>
}

impl Board {
    pub fn new(board: Vec<Vec<Option<Piece>>>, size: Vec2u, pawn_start_y: (u32, u32), white_castling_possible: (bool, bool), black_castling_possible: (bool, bool), en_passant_possible: Option<Vec2i>) -> Self {
        Self {board, size, pawn_start_y, valid_moves: HashMap::new(), white_castle: Castle::new(), black_castle: Castle::new(), en_passant_possible, captured_pieces: Vec::new(), check: (false, false), last_move: None}
    }

    fn valid_moves_for_piece(&self, piece_pos: Vec2i, turn: Side) -> HashSet<Vec2i> {
        let mut valid_moves: HashSet<Vec2i> = HashSet::new();
        let piece = self.get_piece_at_pos(&piece_pos).expect("Tried to calculate possible moves for nonexisiting pieces");
        
        let mvs_in_direction = |dir: Vec2i, valid_mvs: &mut HashSet<Vec2i>| {
            let mut new_pos = piece_pos + dir;
            while self.is_on_board(&new_pos)  {
                if let Some(other_piece) = self.get_piece_at_pos(&new_pos) {
                    if other_piece.side != piece.side {
                        valid_mvs.insert(new_pos);
                        break;
                    } else {
                        break;
                    }
                } else {
                    valid_mvs.insert(new_pos);
                }
                new_pos += dir;
            }
        };

        let rook = |valid_moves: &mut HashSet<Vec2i>| {
            mvs_in_direction(Vec2i::new(1,0), valid_moves);
            mvs_in_direction(Vec2i::new(-1,0), valid_moves);
            mvs_in_direction(Vec2i::new(0,1), valid_moves);
            mvs_in_direction(Vec2i::new(0,-1), valid_moves);
        };

        let bishop = |valid_moves: &mut HashSet<Vec2i>| {
            mvs_in_direction(Vec2i::new(1,1), valid_moves);
            mvs_in_direction(Vec2i::new(-1,1), valid_moves);
            mvs_in_direction(Vec2i::new(1,-1), valid_moves);
            mvs_in_direction(Vec2i::new(-1,-1), valid_moves);
        };

        let knight = |valid_moves: &mut HashSet<Vec2i>| {
            let moves = [
                Vec2i::new(-1, 2),
                Vec2i::new(1, 2),

                Vec2i::new(-1, -2),
                Vec2i::new(1, -2),

                Vec2i::new(2, 1),
                Vec2i::new(2, -1),

                Vec2i::new(-2, 1),
                Vec2i::new(-2, -1),
            ];
            for mv in moves {
                let new_pos = piece_pos + mv;
                if self.is_on_board(&new_pos) {
                    match self.get_piece_at_pos(&new_pos) {
                        Some(other_piece) => if other_piece.side != piece.side {valid_moves.insert(new_pos);},
                        None => {valid_moves.insert(new_pos);}
                    }
                }
            }
        };

        let king = |valid_mvs: &mut HashSet<Vec2i>| {
            for x in -1..=1 {
                for y in -1..=1 {
                    let new_pos = piece_pos + Vec2i::new(x, y);
                    if self.is_on_board(&new_pos) {
                        if let Some(other_piece) = self.get_piece_at_pos(&new_pos) {
                            if other_piece.side == piece.side {
                                continue;
                            }
                        }
                        valid_mvs.insert(new_pos);
                    }
                }
            }

            let castle = |y: i32, castle: Castle, valid_mvs: &mut HashSet<Vec2i>| {
                if castle.long && self.space_is_empty(Vec2i::new(1,y), Vec2i::new(3,y)) {
                    valid_mvs.insert(Vec2i::new(2,y));
                }
                if castle.short && self.space_is_empty(Vec2i::new(5,y), Vec2i::new(6,y)) {
                    valid_mvs.insert(Vec2i::new(6,y));
                }
            };


            match piece.side {
                Side::Black => {
                    castle(0, self.black_castle, valid_mvs);
                },
                Side::White => {
                    castle(7, self.white_castle, valid_mvs);
                },
            }
        };

        let pawn = |valid_mvs: &mut HashSet<Vec2i>| {
            let y_dir = match piece.side {
                Side::Black => 1,
                Side::White => -1
            };
            let mut ys = vec![y_dir];
            match piece.side {
                Side::Black => if piece_pos.y == self.pawn_start_y.1 as i32 {ys.push(y_dir*2)},
                Side::White => if piece_pos.y == self.pawn_start_y.0 as i32 {ys.push(y_dir*2)},
            }
            for y in ys {
                let new_pos = piece_pos + Vec2i::new(0, y);
                if self.is_on_board(&new_pos) && self.space_is_empty(piece_pos + Vec2i::new(0,y_dir), new_pos) {
                    valid_mvs.insert(new_pos);
                }
            };
            for x_dir in [-1, 1] {
                let new_pos = piece_pos + Vec2i::new(x_dir, y_dir);
                if self.is_on_board(&new_pos) {
                    if let Some(other_piece) = self.get_piece_at_pos(&new_pos) {
                        if other_piece.side != piece.side {valid_mvs.insert(new_pos);}
                    }
                } 
            };
            //detect en passant
            if let Some(en_passant_pos) = self.en_passant_possible {
                if (en_passant_pos.x - piece_pos.x).abs() == 1 && en_passant_pos.y == piece_pos.y {
                    valid_mvs.insert(en_passant_pos + Vec2i::new(0,y_dir));
                }
            }
        };

        match piece.ty {
            PieceType::Queen => {
                bishop(&mut valid_moves);
                rook(&mut valid_moves);
            },
            PieceType::King => king(&mut valid_moves),
            PieceType::Knight => knight(&mut valid_moves),
            PieceType::Bishop => bishop(&mut valid_moves),
            PieceType::Rook => rook(&mut valid_moves),
            PieceType::Pawn => pawn(&mut valid_moves),
        }
        valid_moves
    }

    pub fn is_check_after(&self, pos: &Vec2i, dst: &Vec2i, side: Side) -> bool {
        let mut board_copy = self.clone();
        board_copy.move_piece(pos, dst, side);
        let king_pos = board_copy.find_king(side).expect("No king found");
        board_copy.threathens(&king_pos)
    }


    fn find_king(&self, side: Side) -> Option<Vec2i> {
        for (x, y_column) in self.board.iter().enumerate() {
            for (y, optional_piece) in y_column.iter().enumerate() {
                if let Some(piece) = optional_piece && piece.ty == PieceType::King && piece.side == side {
                    return Some(Vec2i::new(x as i32,y as i32));
                }
            }
        }
        None
    }

    fn threathens(&self, victim_pos: &Vec2i) -> bool {
        let victim_piece = self.get_piece_at_pos(victim_pos).unwrap();
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let piece_pos = Vec2i::new(x as i32 , y as i32);
                if let Some(piece) = self.get_piece_at_pos(&piece_pos) {
                    if victim_piece.side != piece.side {
                        let moves = self.valid_moves_for_piece(piece_pos, piece.side);
                        if moves.contains(&victim_pos) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }


    fn is_on_board(&self, pos: &Vec2i) -> bool {
        (0..=7).contains(&pos.x) && (0..=7).contains(&pos.y)
    }



    pub fn calculate_valid_moves(&mut self, turn: Side) -> (i32, i32) {
        let mut valid_moves: HashMap<Vec2i, HashSet<Vec2i>>  = HashMap::new();
        let mut total_moves_pre_check = 0;
        let mut total_moves_post_check = 0;
        for (x, y_row) in self.board.iter().enumerate() {
            for (y, optional_piece) in y_row.iter().enumerate() {
                if optional_piece.is_some() && optional_piece.unwrap().side == turn {
                    let pos = Vec2i::new(x as i32, y as i32);
                    let mut mvs = self.valid_moves_for_piece(pos, turn);
                    total_moves_pre_check += mvs.len();
                    mvs.drain_filter(|to_pos| self.is_check_after(&pos, to_pos, turn));
                    if !mvs.is_empty() {
                        total_moves_post_check += mvs.len();
                        valid_moves.insert(pos, mvs);
                    }
                }
            }
        }
        self.valid_moves = valid_moves;
        (total_moves_pre_check as i32, total_moves_post_check as i32)
    }

    //returns true if move was possible
    pub fn make_move(&mut self, piece_pos: &Vec2i, dst: &Vec2i, turn: Side) -> bool {
        if let Some(valid_moves) = self.valid_moves.get(&piece_pos) {
            if valid_moves.contains(&dst) {
                self.move_piece(&piece_pos, &dst, turn);   
                let (total_moves_pre_check, total_moves_post_check) = self.calculate_valid_moves(!turn);
                self.check = self.is_check();
                
                if total_moves_pre_check == 0 {
                    println!("GAME DRAWN")
                } else if total_moves_post_check == 0 {
                    println!("GAME WON BY {:?} ", turn);
                }

                return true
            }
        }
        false
    }

    pub fn is_check(&self) -> (bool, bool) {
        let white_king = self.find_king(Side::White).expect("No white king found");
        let black_king = self.find_king(Side::Black).expect("No black king found");
        (self.threathens(&white_king), self.threathens(&black_king))
    }

    pub fn set_piece(&mut self, piece: Piece, pos: &Vec2i) {
        if self.is_on_board(pos) {
            self.board[pos.x as usize][pos.y as usize] = Some(piece);
        }
    }

    fn move_piece(&mut self, pos: &Vec2i, dst: &Vec2i, turn: Side) {
        if let Some(piece) = self.get_piece_at_pos(pos) {
            //if there is a figure at the dst
            if let Some(dst_piece) = self.get_piece_at_pos(dst) {
                self.captured_pieces.push(dst_piece);
            }
            match piece.ty {
                PieceType::King => {
                    match piece.side {
                        Side::Black => {
                            self.try_castle(0, self.black_castle, dst);
                            self.black_castle = Castle::forbid();
                        },
                        Side::White => {
                            self.try_castle(7, self.white_castle, dst);
                            self.white_castle = Castle::forbid();
                        },
                    }
                },
                PieceType::Rook => {

                    let rook = |y: i32, castle: &mut Castle| {
                        if *pos == Vec2i::new(0,0) {
                            castle.long = false
                        } else if *pos == Vec2i::new(7,7) {
                            castle.short = false
                        }
                    };

                    match piece.side {
                        Side::Black => rook(7, &mut self.black_castle),
                        Side::White => rook(0, &mut self.black_castle)
                    }
                },
                PieceType::Pawn => {
                    let y_dir = match piece.side {
                        Side::Black => -1,
                        Side::White => 1,
                    };
                    if let Some(en_passant_pos) = self.en_passant_possible {
                        if en_passant_pos == *dst + Vec2i::new(0,y_dir) {
                            self.remove_piece(&en_passant_pos);
                        }
                    }
                    if (pos.y - dst.y).abs() > 1 {
                        self.en_passant_possible = Some(*dst);
                    }
                },
                _ => {self.en_passant_possible = None}
            }

            self.set_piece(piece, dst);
            self.remove_piece(pos);
            self.last_move = Some((*pos, *dst));
        }
    }

    fn try_castle(&mut self, y: i32, castle: Castle, dst: &Vec2i) {
        if *dst == Vec2i::new(2, y) && castle.long {
            let rook = self.get_piece_at_pos(&Vec2i::new(0,y)).expect("Couldnt find castling tower");
            self.set_piece(rook, &Vec2i::new(3,y));
            self.remove_piece(&Vec2i::new(0,y));
        } else if *dst == Vec2i::new(2, y) && castle.short {
            let rook = self.get_piece_at_pos(&Vec2i::new(7,y)).expect("Couldnt find castling tower");
            self.set_piece(rook, &Vec2i::new(5,y));
            self.remove_piece(&Vec2i::new(7,y));
        }
    }

    pub fn remove_piece(&mut self, pos: &Vec2i) {
        if self.is_on_board(pos) {
            self.board[pos.x as usize][pos.y as usize] = None;
        } 
    }

    pub fn get_piece_at_pos(&self, pos: &Vec2i) -> Option<Piece> {
        if self.is_on_board(pos) {
            return self.board[pos.x as usize][pos.y as usize]
        }
        None 
    }

    fn space_is_empty(&self, pos1: Vec2i, pos2: Vec2i) -> bool {
        for x in pos1.x..=pos2.x {
            for y in pos1.y..=pos2.y {
                if self.get_piece_at_pos(&Vec2i::new(x,y)).is_some() {
                    return false;
                }
            }
        }
        return true
    }

}

pub fn gen_kings_pos() -> Board {
    let mut board = vec![vec![None; 8]; 8];
    board[4][0] = Some(Piece::new(PieceType::King, Side::Black));
    board[4][7] = Some(Piece::new(PieceType::King, Side::White));
    Board::new(
        board,
        Vec2u::fill(8),
        (1,6),
        (false, false),
        (false, false),
        None,
    )
}

pub fn gen_starting_pos() -> Board {
    let mut board = vec![vec![None; 8]; 8];


    //gen black
    let mut side = Side::Black;
    //pawns
    for x in 0..=7 {
        board[x][1] = Some(Piece::new(PieceType::Pawn, Side::Black));
        board[x][6] = Some(Piece::new(PieceType::Pawn, Side::White));
    }

    let first_rank = {
        use PieceType::*;
        [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook]
    };

    for (x, piece_ty) in first_rank.iter().enumerate() {
        board[x][0] = Some(Piece::new(*piece_ty, Side::Black));
        board[x][7] = Some(Piece::new(*piece_ty, Side::White))
    }

    Board::new(
        board,
        Vec2u::fill(8),
        (6,1),
        (false, false),
        (false, false),
        None,
    )
}


pub struct ColorTheme {
    pub board_primary: Color,
    pub board_secondary: Color,
    pub valid_moves: Color,
    pub selection: Color
}

impl ColorTheme {
    pub fn new(board_primary: Color, board_secondary: Color,   valid_moves: Color, selection: Color) -> Self {
        Self {board_primary, board_secondary, valid_moves, selection}
    }
}
