use std::{collections::{HashMap, HashSet}, ops::{Range, RangeInclusive}, time::Instant};

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, VecInto};


use crate::{pieces::{Piece, Side, PieceType}, hashmap, count, atlas::TextureAtlas, renderer::Renderer};




pub struct Board {
    pub board: Vec<Vec<Option<Piece>>>,//[[Option<Piece>; 8]; 8],
    pub size: Vec2u,
    //0 = White, 1, Black
    pawn_start_y: (u32, u32),
    pub valid_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    //0 = left, 1 = right
    white_castling_possible: (bool, bool),
    black_castling_possible: (bool, bool),
    en_passant_possible: Option<Vec2i>,
    captured_pieces: Vec<Piece>,
}

impl Board {
    pub fn new(board: Vec<Vec<Option<Piece>>>, size: Vec2u, pawn_start_y: (u32, u32), white_castling_possible: (bool, bool), black_castling_possible: (bool, bool), en_passant_possible: Option<Vec2i>) -> Self {
        Self {board, size, pawn_start_y, valid_moves: HashMap::new(), white_castling_possible, black_castling_possible, en_passant_possible, captured_pieces: Vec::new()}
    }

    fn valid_moves_for_piece(&self, piece_pos: Vec2i) -> HashSet<Vec2i> {
        let mut valid_moves: HashSet<Vec2i> = HashSet::new();
        let piece = self.get_piece_at_pos(piece_pos).expect("Tried to calculate possible moves for nonexisiting pieces");
        
        let mvs_in_direction = |dir: Vec2i, valid_mvs: &mut HashSet<Vec2i>| {
            let mut new_pos = piece_pos + dir;
            while self.is_on_board(new_pos)  {
                if let Some(other_piece) = self.get_piece_at_pos(new_pos) {
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
                if self.is_on_board(new_pos) {
                    match self.get_piece_at_pos(new_pos) {
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
                    if self.is_on_board(new_pos) {
                        if let Some(other_piece) = self.get_piece_at_pos(new_pos) {
                            if other_piece.side == piece.side {
                                continue;
                            }
                        }
                        valid_mvs.insert(new_pos);
                    }
                }
            }
        };

        let pawn = |valid_mvs: &mut HashSet<Vec2i>| {
            let y_dir = match piece.side {
                Side::Black => 1,
                Side::White => -1
            };
            for y in [y_dir, y_dir*2] {
                let new_pos = piece_pos + Vec2i::new(0, y);
                if self.is_on_board(new_pos) && self.space_is_empty(piece_pos + Vec2i::new(0,y_dir), new_pos) {
                    valid_mvs.insert(new_pos);
                }
            };
            for x_dir in [-1, 1] {
                let new_pos = piece_pos + Vec2i::new(x_dir, y_dir);
                if self.is_on_board(new_pos) {
                    if let Some(other_piece) = self.get_piece_at_pos(new_pos) {
                        if other_piece.side != piece.side {valid_mvs.insert(new_pos);}
                    }
                } 
            };
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


    fn is_on_board(&self, pos: Vec2i) -> bool {
        (0..=7).contains(&pos.x) && (0..=7).contains(&pos.y)
    }



    pub fn valid_moves(&mut self) -> HashMap<Vec2i, HashSet<Vec2i>> {
        let mut valid_moves: HashMap<Vec2i, HashSet<Vec2i>>  = HashMap::new();
        for (x, y_row) in self.board.iter().enumerate() {
            for (y, optional_piece) in y_row.iter().enumerate() {
                if optional_piece.is_some() {
                    let pos = Vec2i::new(x as i32, y as i32);
                    let mvs = self.valid_moves_for_piece(pos);
                    if !mvs.is_empty() {
                        valid_moves.insert(pos, mvs);
                    }
                }
            }
        }
        self.valid_moves = valid_moves.clone();
        valid_moves
    }

    //returns if move was possible
    pub fn make_move(&mut self, piece_pos: Vec2i, dst: Vec2i) -> bool {
        let valid_moves = self.valid_moves.get(&piece_pos).expect("generated no move for piece");
        if valid_moves.contains(&dst) {
            self.move_piece(piece_pos, dst);           
            return true
        }
        false
    }

    pub fn set_piece(&mut self, piece: Piece, pos: Vec2i) {
        if self.is_on_board(pos) {
            self.board[pos.x as usize][pos.y as usize] = Some(piece);
        }
    }

    pub fn move_piece(&mut self, pos: Vec2i, dst: Vec2i) {
        if let Some(piece) = self.get_piece_at_pos(pos) {
            //if there is a figure at the dst
            if let Some(dst_piece) = self.get_piece_at_pos(dst) {
                self.captured_pieces.push(dst_piece);
            }
            self.set_piece(piece, dst);
            self.remove_piece(pos);
            self.valid_moves = self.valid_moves();
        }
    }

    pub fn remove_piece(&mut self, pos: Vec2i) {
        if self.is_on_board(pos) {
            self.board[pos.x as usize][pos.y as usize] = None;
        } 
    }

    pub fn get_piece_at_pos(&self, pos: Vec2i) -> Option<Piece> {
        if self.is_on_board(pos.vec_into()) {
            return self.board[pos.x as usize][pos.y as usize]
        }
        None 
    }

    fn space_is_empty(&self, pos1: Vec2i, pos2: Vec2i) -> bool {
        for x in pos1.x..=pos2.x {
            for y in pos1.y..=pos2.y {
                if self.get_piece_at_pos(Vec2i::new(x,y)).is_some() {
                    return false;
                }
            }
        }
        return true
    }

}


pub fn gen_starting_pos() -> Board {
    let mut board = vec![vec![None; 8]; 8];


    //gen black
    let mut side = Side::Black;
    //pawns
    for x in 0..=7 {
        board[x][1] = Some(Piece::new(PieceType::Pawn, side));
    }

    //rooks
    board[0][0] = Some(Piece::new(PieceType::Rook, side));
    board[7][0] = Some(Piece::new(PieceType::Rook, side));

    //knights
    board[1][0] = Some(Piece::new(PieceType::Knight, side));
    board[6][0] = Some(Piece::new(PieceType::Knight, side));

    //bishops
    board[2][0] = Some(Piece::new(PieceType::Bishop, side));
    board[5][0] = Some(Piece::new(PieceType::Bishop, side));

    //king & queen
    board[3][0] = Some(Piece::new(PieceType::Queen, side));
    board[4][0] = Some(Piece::new(PieceType::King, side));

    //gen white
    side = Side::White;
    //pawns
    for x in 0..=7 {
        board[x][6] = Some(Piece::new(PieceType::Pawn, side));
    }

    //rooks
    board[0][7] = Some(Piece::new(PieceType::Rook, side));
    board[7][7] = Some(Piece::new(PieceType::Rook, side));

    //knights
    board[1][7] = Some(Piece::new(PieceType::Knight, side));
    board[6][7] = Some(Piece::new(PieceType::Knight, side));

    //bishops
    board[2][7] = Some(Piece::new(PieceType::Bishop, side));
    board[5][7] = Some(Piece::new(PieceType::Bishop, side));

    //king & queen
    board[3][7] = Some(Piece::new(PieceType::Queen, side));
    board[4][7] = Some(Piece::new(PieceType::King, side));

    Board::new(
        board,
        Vec2u::fill(8),
        (1,6),
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
