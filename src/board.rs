use std::{collections::{HashMap, HashSet}, ops::{Range, RangeInclusive}, time::Instant};

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::{Vec2i, Vec2u, VecInto};


use crate::{pieces::{Piece, Side, PieceType}, hashmap, count, atlas::TextureAtlas};




pub struct Board {
    board: Vec<Vec<Option<Piece>>>,//[[Option<Piece>; 8]; 8],
    size: Vec2u,
    //0 = White, 1, Black
    pawn_start_y: (u32, u32), 
    //0 = left, 1 = right
    white_castling_possible: (bool, bool),
    black_castling_possible: (bool, bool),
    en_passant_possible: Option<Vec2i>,
    beaten_figures: Vec<Piece>,
}

impl Board {
    pub fn new(board: Vec<Vec<Option<Piece>>>, size: Vec2u, pawn_start_y: (u32, u32), white_castling_possible: (bool, bool), black_castling_possible: (bool, bool), en_passant_possible: Option<Vec2i>) -> Self {
        Self {board, size, pawn_start_y, white_castling_possible, black_castling_possible, en_passant_possible, beaten_figures: Vec::new()}
    }

    fn valid_moves_for_piece(&self, piece_pos: Vec2i) -> HashSet<Vec2i> {
        let mut valid_moves: HashSet<Vec2i> = HashSet::new();
        //let piece = self.board.get(piece_pos.x).get;
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
                match self.get_piece_at_pos(new_pos.vec_into()) {
                        Some(other_piece) => if other_piece.side != piece.side {valid_moves.insert(new_pos.vec_into());},
                        None => {valid_moves.insert(new_pos.vec_into());}
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
                Side::Black => -1,
                Side::White => 1
            };
            for y in [y_dir, y_dir*2] {
                let new_pos = piece_pos + Vec2i::new(0, y_dir);
                if self.is_on_board(new_pos) && self.space_is_empty(piece_pos, new_pos) {
                    valid_mvs.insert(new_pos.vec_into());
                }
            };
            for x_dir in [-1, 1] {
                let new_pos = piece_pos + Vec2i::new(x_dir, y_dir);
                if self.is_on_board(new_pos) {
                    if let Some(other_piece) = self.get_piece_at_pos(new_pos.vec_into()) {
                        if other_piece.side != piece.side {valid_mvs.insert(new_pos.vec_into());}
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



    fn valid_moves(&self) -> HashMap<Vec2i, HashSet<Vec2i>> {
        let mut valid_moves: HashMap<Vec2i, HashSet<Vec2i>>  = HashMap::new();

        valid_moves
    }


    fn get_piece_at_pos(&self, pos: Vec2i) -> Option<Piece> {
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


pub struct Renderer<'a> {
    tex_atlas: &'a TextureAtlas<'a>,
    canvas: &'a mut Canvas<Window>,
    rects: Vec<(Rect, Color)>,
    last_frame_time: Instant,
    s_tick: f32,
    s_tick_increment: f32
}


impl<'a> Renderer<'a> {
    pub fn new(tex_atlas: &'a TextureAtlas<'a>, s_tick_increment: f32, canvas: &'a mut Canvas<Window>) -> Self {
        Self {tex_atlas, rects: Vec::new(), last_frame_time: Instant::now(), s_tick: 0.0, s_tick_increment, canvas}
    }
    
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.rects.push((rect, color));
    }

    pub fn render(&mut self) {
        let current_frame_time = Instant::now();
        let dt = (current_frame_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_frame_time;

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();



        for rect in &self.rects {
            self.canvas.set_draw_color(rect.1);
            self.canvas.fill_rect(rect.0);
        }
        self.rects.clear();

        self.canvas.present();
    }
}


fn gen_starting_pos() -> Board {
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
        board[x][7] = Some(Piece::new(PieceType::Pawn, side));
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

    Board {
        white_castling_possible: (false, false),
        black_castling_possible: (false, false),
        en_passant_possible: None,
        board,
        size: Vec2u::fill(8),
        pawn_start_y: (1,6),
        beaten_figures: Vec::new(),
    }
}


pub struct BoardRenderer {
    board_ground: Vec<(Rect, Color)>,
    hovering: Option<Vec2i>,
    selected: Option<Vec2i>,
    valid_mvs_tick: f32,
    last_move_tick: f32
}


impl BoardRenderer {
    pub fn new(size: Vec2u) -> Self {
        let mut board_ground: Vec<(Rect, Color)> = Vec::new();
        let mut color = Color::WHITE;
        for i in 0..8 {
            for n in 0..8 {
                let rect = Rect::new(50 * i, 50 * n, 50, 50);
                if (i % 2 == 1 && n % 2 == 0) || (i % 2 == 0 && n % 2 == 1) {
                    //color = black
                    color = Color::BLACK;
                } else {
                    color = Color::WHITE;
                }
                board_ground.push((rect, color));
            }
        }
        Self {board_ground, hovering: None, selected: None, valid_mvs_tick: 0.0, last_move_tick: 0.0 }
    }

    pub fn hover(&mut self, pos: Vec2i) {
        self.hovering = Some(pos);
    }


    pub fn render(&mut self, renderer: &mut Renderer) {
        for rect in &self.board_ground {
            renderer.draw_rect(rect.0, rect.1);
        }
        self.hovering = None
    }
}