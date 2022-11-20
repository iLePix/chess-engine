use std::{collections::HashMap, ops::{Range, RangeInclusive}};

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::Vec2i;


use crate::{figures::{Figure, Side, FigureType}, hashmap, count, atlas::TextureAtlas};







pub struct Board<'a> {
    white_rects: Vec<Rect>,
    black_rects: Vec<Rect>,
    tex_atlas: &'a TextureAtlas<'a>,
    pos: [Option<Figure>; 64],
    hovering: Option<u8>,
    pub selected: Option<u8>,
    beaten_figures: Vec<Figure>,
    valid_moves_for_selected_fig: Vec<u8>,
    valid_mvs_tick: f32,
    white_castling_possible: bool,
    black_castling_possible: bool,
    last_move: Option<(u8, u8)>,
    last_move_tick: f32,
}


impl<'a> Board<'a> {
    pub fn new(tex_atlas: &'a TextureAtlas) -> Self {
        let mut white_rects: Vec<Rect> = vec![];
        let mut black_rects: Vec<Rect> = vec![];
        for i in 0..8 {
            for n in 0..8 {
                let rect = Rect::new(50 * i, 50 * n, 50, 50);
                if (i % 2 == 1 && n % 2 == 0) || (i % 2 == 0 && n % 2 == 1) {
                    //color = black
                    black_rects.push(rect);
                } else {
                    //color = white
                    white_rects.push(rect);
                }
            }
        }




        Self {
            white_rects, 
            black_rects, 
            pos: Self::gen_start_pos(),
            tex_atlas,
            hovering: None,
            selected: None,
            beaten_figures: Vec::new(),
            valid_moves_for_selected_fig: Vec::new(),
            valid_mvs_tick: 0.0,
            white_castling_possible: true,
            black_castling_possible: true,
            last_move: None,
            last_move_tick: 0.0
        }
    }

    fn gen_start_pos() -> [Option<Figure>; 64] {
        let mut start_pos: [Option<Figure>; 64] = [None; 64];

        //gen black
        let mut side = Side::Black;
        //pawns
        for i in 8..=15 {
            start_pos[i] = Some(Figure::new(FigureType::Pawn, side, 11));
        }

        //rooks
        start_pos[0] = Some(Figure::new(FigureType::Rook, side, 9));
        start_pos[7] = Some(Figure::new(FigureType::Rook, side, 9));

        //knights
        start_pos[1] = Some(Figure::new(FigureType::Knight, side, 7));
        start_pos[6] = Some(Figure::new(FigureType::Knight, side, 7));

        //bishops
        start_pos[2] = Some(Figure::new(FigureType::Bishop, side, 5));
        start_pos[5] = Some(Figure::new(FigureType::Bishop, side, 5));

        //king & queen
        start_pos[3] = Some(Figure::new(FigureType::Queen, side, 3));
        start_pos[4] = Some(Figure::new(FigureType::King, side, 1));

        //gen white
        side = Side::White;
        //pawns
        for i in 48..=55 {
            start_pos[i] = Some(Figure::new(FigureType::Pawn, side, 10));
        }

        //rooks
        start_pos[56] = Some(Figure::new(FigureType::Rook, side, 8));
        start_pos[63] = Some(Figure::new(FigureType::Rook, side, 8));

        //knights
        start_pos[57] = Some(Figure::new(FigureType::Knight, side, 6));
        start_pos[62] = Some(Figure::new(FigureType::Knight, side, 6));

        //bishops
        start_pos[58] = Some(Figure::new(FigureType::Bishop, side, 4));
        start_pos[61] = Some(Figure::new(FigureType::Bishop, side, 4));

        //king & queen
        start_pos[59] = Some(Figure::new(FigureType::Queen, side, 2));
        start_pos[60] = Some(Figure::new(FigureType::King, side, 0));


        start_pos
    }


    pub fn draw(&mut self, canvas: &mut Canvas<Window>, turn: Side, dt: f32) {
        //draw board
        canvas.set_draw_color(Color::RGB(250,232,168,)); // rgba(250,232,168,255)
        canvas.fill_rects(&self.white_rects).unwrap();

        canvas.set_draw_color(Color::RGB(20,95,75));
        canvas.fill_rects(&self.black_rects).unwrap();


        //draw last move
        if let Some(last_move) = self.last_move {
            self.last_move_tick += dt * 100.0;
            if self.last_move_tick > 50.0 {
                self.last_move_tick = 50.0;
            }

            let mut draw_rect = |i: u8| {
                let indicator_size = self.last_move_tick as u32;
                let size = 50;
                let x = ((i % 8) as u32 * size) as i32;
                let y = ((i / 8) as u32 * size) as i32;
                let r = Rect::from_center(Point::new(x + size as i32 /2, y + size as i32 /2), indicator_size, indicator_size);
                canvas.set_draw_color(Color::RGBA(255,255, 0, 128));
                canvas.fill_rect(r).unwrap()
            };
            draw_rect(last_move.0);
            draw_rect(last_move.1);
        }



        //draw valid moves
        self.valid_mvs_tick += dt * 100.0;
        if self.valid_mvs_tick > 30.0 {
            self.valid_mvs_tick = 30.0;
        }

        self.valid_moves_for_selected_fig.iter()
            .for_each(|i| {
                let size: u32 = 50;
                let x = ((i % 8) as u32 * size) as i32;
                let y = ((i / 8) as u32 * size) as i32;
                let indicator_size = self.valid_mvs_tick as u32;
                let r = Rect::from_center(Point::new(x + size as i32 /2, y + size as i32 /2), indicator_size, indicator_size);
                canvas.set_draw_color(Color::RGBA(3, 138, 255, 128));
                canvas.fill_rect(r).unwrap();
            });

        //draw selection
        if let Some(selected) = self.selected {
            let size: u32 = 50;
            let x = ((selected % 8) as u32 * size) as i32;
            let y = ((selected / 8) as u32 * size) as i32;
            let indicator_size = (self.valid_mvs_tick * 0.75) as u32;
            let s_r = Rect::from_center(Point::new(x + size as i32 /2, y + size as i32 /2), indicator_size, indicator_size);
            canvas.set_draw_color(Color::RGBA(255, 123, 98, 200));
            canvas.fill_rect(s_r).unwrap();
        }


        //draw figures
        self.pos.iter()
            .enumerate()
            .filter(|(_,f)| f.is_some())
            .for_each(|(i, f)| {
                let mut size = 50;
                let mut x = ((i % 8) * size) as i32;
                let mut y = ((i / 8) * size) as i32;
                let f = f.unwrap();
                if f.side == turn {
                    if let Some(selected) = self.selected && selected == i as u8 {
                        return
                    }
                    if let Some(hovering) = self.hovering && hovering == i as u8 {
                        x -= 5;
                        y -= 5;
                        size += 10;
                    }
                }
                let src = self.tex_atlas.figure_atlas_cords.get(&f.tex_id)
                    .unwrap_or_else(|| panic!("Created figure with wrong index {}", f.tex_id));
                let dst = Rect::new(x,y, size as u32,size as u32);
                canvas.copy(self.tex_atlas.pieces_texture, *src, dst).unwrap();
            })
    }

    pub fn select(&mut self, i: u8, side: Side) -> Option<Figure> { 
        if let Some(selected) = self.selected {
            if i == selected { 
                self.unselect();
                return None;
            }
        }
        if i < 64 {
            if let Some(f) = self.pos[i as usize] {
                if f.side == side {
                    self.selected = Some(i);
                    self.valid_moves_for_selected_fig = self.valid_moves(i);
                    return Some(f)
                }
            }
        }
        None
    }

    pub fn move_figure(&mut self, dst: u8) -> bool {
        if dst < 64 {
            if let Some(selected) = self.selected {
                if dst == selected {
                    self.unselect();
                    return false
                }
                //dont allow for any future castling if king or tower of specific side moved

                if self.valid_moves_for_selected_fig.contains(&dst) {
                    //detect castling
                    let selected_figure = self.pos(selected).unwrap();

                    match selected_figure.ty {
                        FigureType::King => {
                            match selected_figure.side {
                                Side::Black => {
                                    if self.black_castling_possible && dst == 2 {
                                        self.pos[3] = self.pos[0];
                                        self.pos[0] = None;
                                    }  
                                    self.black_castling_possible = false;
                                },
                                Side::White => {
                                    if self.white_castling_possible && dst == 58 {
                                        self.pos[59] = self.pos[56];
                                        self.pos[56] = None;
                                    }
                                    self.white_castling_possible = false;
                                },
                            }
                        },
                        FigureType::Rook => {
                            match selected_figure.side {
                                Side::Black => if self.black_castling_possible {
                                    self.black_castling_possible = !(selected == 0);
                                },
                                Side::White => if self.white_castling_possible {
                                    self.white_castling_possible = !(selected == 56);
                                },
                            }
                        },
                        _ => {},
                    }
                    if let Some(dst_fig) = self.pos[dst as usize] {
                        self.beaten_figures.push(dst_fig);
                    }
                    self.pos[dst as usize] = self.pos[selected as usize];
                    self.pos[selected as usize] = None;
                    self.last_move = Some((selected, dst));
                    self.unselect();
                    return true
                }
            }
        }
        false //bc every move is allowed rn 
    }


    pub fn valid_moves(&mut self, i: u8) -> Vec<u8> {
        let mut valid_mvs = Vec::new();
        let f = self.pos[i as usize].unwrap();
        let pos = self.i_to_xy(i).unwrap();
        self.valid_mvs_tick = 0.0;

        let mut mv = |dir: Point| {
            let mut new_pos = pos + dir;
            while (new_pos.x >= 0 && new_pos.x  <= 7) && (new_pos.y >= 0 && new_pos.y  <= 7) {
                let new_pos_i = self.xy_to_i(new_pos).unwrap();
                if let Some(o_f) = self.pos(new_pos_i) {
                    if o_f.side != f.side {
                        valid_mvs.push(new_pos_i);
                        break;
                    } else {
                        break;
                    }
                } else {
                    valid_mvs.push(new_pos_i);
                }
                new_pos += dir;
            }
        };

        match f.ty {
            FigureType::Queen => {
                mv(Point::new(-1,1));
                mv(Point::new(1,1));
                mv(Point::new(-1,-1));
                mv(Point::new(1,-1));

                mv(Point::new(1,0));
                mv(Point::new(-1,0));
                mv(Point::new(0,1));
                mv(Point::new(0,-1));
            },
            FigureType::King => {
                //regular
                for x in -1..=1 {
                    for y in -1..=1 {
                        if let Some(new_pos_i) = self.xy_to_i(pos + Point::new(x, y)) {
                            if let Some(o_f) = self.pos(new_pos_i) {
                                if o_f.side == f.side {
                                    continue;
                                }
                            }
                            valid_mvs.push(new_pos_i);
                        }
                    }
                }
                //edge-cases
                //castling

                let castling_space_is_empty = |space: RangeInclusive<u8>| -> bool {
                    for o_f_i in space {
                        if self.pos(o_f_i).is_some() {
                            return false
                        }
                    }
                    return true
                };

                match f.side {
                    Side::Black => {
                        if self.black_castling_possible && i == 4 && castling_space_is_empty(RangeInclusive::new(1, 3)) {
                            valid_mvs.push(2);
                        }
                    },
                    Side::White => {
                        if self.white_castling_possible && i == 60 && castling_space_is_empty(RangeInclusive::new(57, 59)) {
                            valid_mvs.push(58);
                        }
                    },
                }

            },
            FigureType::Knight => {
                let moves = vec![
                    Point::new(-1, 2),
                    Point::new(1, 2),

                    Point::new(-1, -2),
                    Point::new(1, -2),

                    Point::new(2, 1),
                    Point::new(2, -1),

                    Point::new(-2, 1),
                    Point::new(-2, -1),
                ];
                for mv in moves {
                    if let Some(p) = self.xy_to_i(pos + mv) {
                        match self.pos(p) {
                            Some(o_f) => if o_f.side != f.side {valid_mvs.push(p)},
                            None => valid_mvs.push(p),
                        }
                    }
                }
            },
            FigureType::Bishop => {
                mv(Point::new(-1,1));
                mv(Point::new(1,1));
                mv(Point::new(-1,-1));
                mv(Point::new(1,-1));
            },
            FigureType::Rook => {
                mv(Point::new(1,0));
                mv(Point::new(-1,0));
                mv(Point::new(0,1));
                mv(Point::new(0,-1));
            },
            FigureType::Pawn => {
                /*let pawn_beating = |pos: Option<u8>, o_side: Side| {
                    if let Some(pos) = pos && let Some(fig) = self.pos(pos) && fig.side == o_side {
                        valid_mvs.push(pos);
                    }
                };*/

                match f.side {
                    Side::Black => {
                        if pos.y > 6 {
                            return valid_mvs;
                        }
                        let front_pos = self.xy_to_i(pos + Point::new(0, 1));   
                        let left_pos = self.xy_to_i(pos + Point::new(-1, 1));
                        let right_pos = self.xy_to_i(pos + Point::new(1, 1));
                        let jump_pos = self.xy_to_i(pos + Point::new(0, 2));
                        if let Some(f_p) = front_pos && self.pos(f_p).is_none() {
                            valid_mvs.push(f_p);
                        }
                        if let Some(l_p) = left_pos && let Some(l_f) = self.pos(l_p) && l_f.side == Side::White {
                            valid_mvs.push(l_p);
                        }
                        if let Some(r_p) = right_pos && let Some(r_f) = self.pos(r_p) && r_f.side == Side::White {
                            valid_mvs.push(r_p);
                        }
                        if let Some(j_p) = jump_pos && self.pos(j_p).is_none() && pos.y == 1 && self.pos(j_p - 8).is_none() {
                            valid_mvs.push(j_p);
                        }
                    },
                    Side::White => {
                        if pos.y < 0 {
                            return valid_mvs;
                        }
                        let front_pos = self.xy_to_i(pos + Point::new(0, -1));   
                        let left_pos = self.xy_to_i(pos + Point::new(-1, -1));
                        let right_pos = self.xy_to_i(pos + Point::new(1, -1));
                        let jump_pos = self.xy_to_i(pos + Point::new(0, -2));
                        if let Some(f_p) = front_pos && self.pos(f_p).is_none() {
                            valid_mvs.push(f_p);
                        }
                        if let Some(l_p) = left_pos && let Some(l_f) = self.pos(l_p) && l_f.side == Side::Black {
                            valid_mvs.push(l_p);
                        }
                        if let Some(r_p) = right_pos && let Some(r_f) = self.pos(r_p) && r_f.side == Side::Black {
                            valid_mvs.push(r_p);
                        }
                        if let Some(j_p) = jump_pos && self.pos(j_p).is_none() &&  pos.y == 6 && self.pos(j_p + 8).is_none() {
                            valid_mvs.push(j_p);
                        }
                    },
                }
            },
        }
        valid_mvs
    }

    fn pos(&self, i: u8) -> Option<Figure> {
        if i < 64 {
            return self.pos[i as usize];
        }
        None
    }

    pub fn get_selected_fig(&self) -> Option<Figure> {
        if let Some(selected) = self.selected {
            return self.pos[selected as usize]
        }
        None
    }

    pub fn unselect(&mut self) {
        self.valid_moves_for_selected_fig.clear();
        self.selected = None;
    }

    pub fn hover(&mut self, i: u8) {
        self.hovering = None;
    }

    pub fn i_to_xy(&self, i: u8) -> Option<Point> {
        if !(0..=63).contains(&i) {
            return None;
        }
        let x = (i % 8) as i32;
        let y = (i / 8) as i32;
        Some(Point::new(x,y))
    }

    pub fn xy_to_i(&self, p: Point) -> Option<u8> {
        if !(0..=7).contains(&p.y()) || !(0..=7).contains(&p.x()) {
            return None
        }
        Some((p.x + p.y*8) as u8)
    }
}