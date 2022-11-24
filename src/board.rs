use std::{collections::{HashMap, HashSet}, ops::{Range, RangeInclusive}, time::Instant, error::Error};

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, VecInto};


use crate::{pieces::{Piece, Side, PieceType}, hashmap, count, atlas::TextureAtlas, renderer::Renderer, game::GameState};


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

    //KQkq ->

    fn from_fen(fen: &str) -> Result<(Self, Self), FenError> {
        let mut black = Castle::forbid();
        let mut white = Castle::forbid();
        if fen == "-" { return Ok((white, black)) }
        for c in fen.chars() {
            match c {
                'K' => white.short = true,
                'Q' => white.long = true,
                'k' => black.short = true,
                'q' => black.long = true,
                _ => return Err(FenError::Castle)
            }
        }
        Ok((white, black)) 
    }

}


#[derive(Clone)]
pub struct Board {
    pub board: [[Option<Piece>; 8]; 8],
    pub size: Vec2u,
    //0 = White, 1, Black
    pawn_start_y: (u32, u32),
    pub valid_moves: HashMap<Vec2i, HashSet<Vec2i>>,
    //0 = long, 1 = short
    white_castle: Castle,
    black_castle: Castle,
    en_passant_possible: Option<Vec2i>,
    pub check: (bool, bool),
    //captured_pieces: Vec<Piece>,
    pub last_move: Option<(Vec2i, Vec2i)>
}

impl Board {
    pub fn new(board: [[Option<Piece>; 8]; 8], size: Vec2u, pawn_start_y: (u32, u32), white_castling_possible: (bool, bool), black_castling_possible: (bool, bool), en_passant_possible: Option<Vec2i>) -> Self {
        Self {board, size, pawn_start_y, valid_moves: HashMap::new(), white_castle: Castle::new(), black_castle: Castle::new(), en_passant_possible, check: (false, false), last_move: None}
    }

    fn valid_moves_for_piece(&self, piece_pos: Vec2i, turn: Side, from_castling_check: bool) -> HashSet<Vec2i> {
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
                //for long: x = 2,3 for short: x = 5,6 / check for check
                if 
                    castle.long && 
                    self.space_is_empty(Vec2i::new(1,y), Vec2i::new(3,y)) && 
                    (2..=4).all(|x| !self.threathens(&Vec2i::new(x,y), turn, true))
                {
                    valid_mvs.insert(Vec2i::new(2,y));
                }
                if  castle.short && 
                    self.space_is_empty(Vec2i::new(5,y), Vec2i::new(6,y)) &&
                    (4..=6).all(|x| !self.threathens(&Vec2i::new(x,y), turn, true))
                {
                    valid_mvs.insert(Vec2i::new(6,y));
                }
            };

            if !from_castling_check {
                match piece.side {
                    Side::Black => {
                        castle(0, self.black_castle, valid_mvs);
                    },
                    Side::White => {
                        castle(7, self.white_castle, valid_mvs);
                    },
                }
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
        board_copy.threathens(&king_pos, side, false)
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

    fn threathens(&self, victim_pos: &Vec2i, victim_side: Side, from_castling_check: bool) -> bool {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let piece_pos = Vec2i::new(x as i32 , y as i32);
                if let Some(piece) = self.get_piece_at_pos(&piece_pos) {
                    if victim_side != piece.side {
                        let moves = self.valid_moves_for_piece(piece_pos, piece.side, from_castling_check);
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
                    let mut mvs = self.valid_moves_for_piece(pos, turn, false);
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
    pub fn make_move(&mut self, piece_pos: &Vec2i, dst: &Vec2i, turn: Side) -> Result<GameState, ()> {
        if let Some(valid_moves) = self.valid_moves.get(&piece_pos) {
            if valid_moves.contains(&dst) {
                self.move_piece(&piece_pos, &dst, turn);   
                let (total_moves_pre_check, total_moves_post_check) = self.calculate_valid_moves(!turn);
                self.check = self.is_check();
                
                if total_moves_pre_check == 0 {
                    return Ok(GameState::Draw)
                } else if total_moves_post_check == 0 {
                    return Ok(GameState::Winner((turn)))
                }

                return Ok(GameState::Running)
            }
        }
        return Err(())
    }

    pub fn is_check(&self) -> (bool, bool) {
        let white_king = self.find_king(Side::White).expect("No white king found");
        let black_king = self.find_king(Side::Black).expect("No black king found");
        (self.threathens(&white_king, Side::White, false), self.threathens(&black_king, Side::Black, false))
    }

    pub fn set_piece(&mut self, piece: Piece, pos: &Vec2i) {
        if self.is_on_board(pos) {
            self.board[pos.x as usize][pos.y as usize] = Some(piece);
        }
    }

    fn move_piece(&mut self, pos: &Vec2i, dst: &Vec2i, turn: Side) {
        if let Some(mut piece) = self.get_piece_at_pos(pos) {
            //if there is a figure at the dst
            if let Some(dst_piece) = self.get_piece_at_pos(dst) {
                //self.captured_pieces.push(dst_piece);
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
                        if *pos == Vec2i::new(0,y) {
                            castle.long = false
                        } else if *pos == Vec2i::new(7,y) {
                            castle.short = false
                        }
                    };

                    match piece.side {
                        Side::Black => rook(0, &mut self.black_castle),
                        Side::White => rook(7, &mut self.white_castle)
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
                    if dst.y == 0 || dst.y == 7 {
                        piece = Piece::new(PieceType::Queen, turn);
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
        if let Some(rook) = self.get_piece_at_pos(&Vec2i::new(0,y)) && *dst == Vec2i::new(2, y) && castle.long {
            self.set_piece(rook, &Vec2i::new(3,y));
            self.remove_piece(&Vec2i::new(0,y));
        } else if let Some(rook) = self.get_piece_at_pos(&Vec2i::new(7,y)) && *dst == Vec2i::new(6, y) && castle.short {
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
        for x in pos1.x.min(pos2.x)..=pos2.x.max(pos1.x)  {
            for y in pos1.y.min(pos2.y)..=pos2.y.max(pos1.y) {
                if self.get_piece_at_pos(&Vec2i::new(x,y)).is_some() {
                    return false;
                }
            }
        }
        return true
    }


    pub fn make_best_move(&mut self, turn: Side) -> (Vec2i, Vec2i) {
        let mut best_move = ((Vec2i::zero(), Vec2i::zero()), -std::i32::MAX);
        for (piece_pos, mvs) in &self.valid_moves {
            for dst in mvs {
                let mut board_clone = self.clone();
                board_clone.make_move(&piece_pos, &dst, turn);
                let eval = self.evaluate(turn);
                if eval > best_move.1 {
                    best_move = ((*piece_pos, *dst), eval);
                }
            }
        }
        self.make_move(&best_move.0.0, &best_move.0.1, turn);
        best_move.0
    }



    pub fn evaluate(&self, side: Side) -> i32 {
        self.count_material(side) - self.count_material(!side)
    }

    fn count_material(&self, turn: Side) -> i32 {
        const PAWN_VALUE: i32 = 100;
        const KNIGHT_VALUE: i32 = 300;
        const BISHOP_VALUE: i32 = 300;
        const ROOK_VALUE: i32 = 500;
        const QUEEN_VALUE: i32 = 900;

        let mut material = 0;
        for optional_piece in self.board.iter().flat_map(|column| column.iter()) {
            if let Some(piece) = optional_piece && piece.side == turn {
                material += match piece.ty {
                    PieceType::Queen => QUEEN_VALUE,
                    PieceType::King => 0,
                    PieceType::Knight => KNIGHT_VALUE,
                    PieceType::Bishop => BISHOP_VALUE,
                    PieceType::Rook => ROOK_VALUE,
                    PieceType::Pawn => PAWN_VALUE,
                }
            }
        }
        material
    }

    pub fn gen_starting_pos() -> Self {
        let mut board = [[None; 8]; 8];
    
    
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
    
        Self::new(
            board,
            Vec2u::fill(8),
            (6,1),
            (false, false),
            (false, false),
            None,
        )
    }

    pub fn gen_kings_pos() -> Board {
        let mut board = [[None; 8]; 8];
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

    // "p" stands for pawn, "r" for rook, "n" for knight, "b" for bishop, "q" for queen, and "k" for king.

    pub fn from_fen(fen: &str) -> Result<(Self, Side), FenError> {
        let mut cursor = Vec2i::zero();
        let mut board = [[None; 8]; 8];
        let mut sections = fen.split(' ');
        fn piece_type(c: char) -> Option<PieceType> {
            use PieceType::*;
            match c {
                'p' => Some(Pawn),
                'r' => Some(Rook),
                'n' => Some(Knight),
                'b' => Some(Bishop),
                'q' => Some(Queen),
                'k' => Some(King),
                e => {panic!("Undefined character in fen: {}", e); None}
            }
        }

        fn pos(s: &str) -> Result<Vec2i, FenError> {
            let a = s.chars().next().ok_or(FenError::EnPassant)?;
            let b = s.chars().next().ok_or(FenError::EnPassant)?;
            if s.chars().next().is_some() || !('a'..='h').contains(&a) || !('1'..='8').contains(&a) {
                return Err(FenError::EnPassant);
            }
            Ok(Vec2i::new((a as u8 - b'a') as i32, (b as u8 - b'1') as i32))
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
                    board[cursor.x as usize][cursor.y as usize] = Some(
                        Piece::new(
                            piece_type(c.to_ascii_lowercase()).ok_or(FenError::Pieces)?,
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
            "-" => None,
            s => Some(pos(s)?)
        };

        let _halfmoves: &str = sections.next().ok_or(FenError::HalfMoves)?;
        let _fullmoves: &str = sections.next().ok_or(FenError::FullMoves)?;

        if sections.next().is_some() { return Err(FenError::Cursor)}


        Ok((    
            Self {
                board,
                white_castle,
                black_castle,
                en_passant_possible,
                valid_moves: HashMap::new(),
                check: (false, false),
                last_move: None,
                pawn_start_y: (1,6),
                size: Vec2u::fill(8)
            },
            turn
        ))
    }

}

#[derive(Debug)]
pub enum FenError {
    Turn,
    Cursor,
    Castle,
    EnPassant,
    Pieces,
    HalfMoves,
    FullMoves,
    MissingSection(u32)
}






