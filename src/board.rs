use std::collections::HashMap;

use sdl2::{rect::{Rect, Point}, pixels::Color, render::{Canvas, Texture}, video::Window, sys::PropModePrepend};
use vecm::vec::Vec2i;


use crate::{figures::{Figure, Side, FigureType}, hashmap, count, atlas::TextureAtlas};







pub struct Board<'a> {
    white_rects: Vec<Rect>,
    black_rects: Vec<Rect>,
    tex_atlas: &'a TextureAtlas<'a>,
    pos: [Option<Figure>; 64],
    hovering: Option<u8>,
    pub selected: Option<u8>
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
            selected: None
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
        //knights
        start_pos[0] = Some(Figure::new(FigureType::Knight, side, 9));
        start_pos[7] = Some(Figure::new(FigureType::Knight, side, 9));

        //rooks
        start_pos[1] = Some(Figure::new(FigureType::Rook, side, 7));
        start_pos[6] = Some(Figure::new(FigureType::Rook, side, 7));

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
        //knights
        start_pos[56] = Some(Figure::new(FigureType::Knight, side, 8));
        start_pos[63] = Some(Figure::new(FigureType::Knight, side, 8));

        //rooks
        start_pos[57] = Some(Figure::new(FigureType::Rook, side, 6));
        start_pos[62] = Some(Figure::new(FigureType::Rook, side, 6));

        //bishops
        start_pos[58] = Some(Figure::new(FigureType::Bishop, side, 4));
        start_pos[61] = Some(Figure::new(FigureType::Bishop, side, 4));

        //king & queen
        start_pos[59] = Some(Figure::new(FigureType::Queen, side, 2));
        start_pos[60] = Some(Figure::new(FigureType::King, side, 0));


        start_pos
    }


    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(250,232,168,)); // rgba(250,232,168,255)
        canvas.fill_rects(&self.white_rects).unwrap();

        canvas.set_draw_color(Color::RGB(20,95,75));
        canvas.fill_rects(&self.black_rects).unwrap();

        //draw figures

        self.pos.iter()
            .enumerate()
            .filter(|(_,f)| f.is_some())
            .for_each(|(i, f)| {
                let mut size = 50;
                let mut x = ((i % 8) * size) as i32;
                let mut y = ((i / 8) * size) as i32;
                let f = f.unwrap();
                if f.side == Side::White {
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

    pub fn select(&mut self, i: u8) -> Option<Figure> { 
        if let Some(selected) = self.selected {
            if i == selected { 
                self.selected = None;
                return None;
            }
        }
        if i < 64 {
            if let Some(f) = self.pos[i as usize] {
                if f.side == Side::White {
                    self.selected = Some(i);
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
                if self.valid_moves(selected).contains(&dst) {
                    self.pos[dst as usize] = self.pos[selected as usize];
                    self.pos[selected as usize] = None;
                    self.unselect();
                    return true
                }
            }
        }
        false //bc every move is allowed rn 
    }


    pub fn valid_moves(&self, i: u8) -> Vec<u8> {
        let mut valid_mvs = Vec::new();
        let f = self.pos[i as usize].unwrap();
        match f.ty {
            FigureType::Queen => todo!(),
            FigureType::King => todo!(),
            FigureType::Knight => todo!(),
            FigureType::Bishop => todo!(),
            FigureType::Rook => todo!(),
            FigureType::Pawn => {
                let pos = self.i_to_xy(i).unwrap();
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
                    },
                    Side::White => {
                        if pos.y > 6 {
                            return valid_mvs;
                        }
                        let front_pos = self.xy_to_i(pos + Point::new(0, -1));   
                        let left_pos = self.xy_to_i(pos + Point::new(-1, -1));
                        let right_pos = self.xy_to_i(pos + Point::new(1, -1));
                        let jump_pos = self.xy_to_i(pos + Point::new(0, -2));
                        if let Some(f_p) = front_pos && self.pos(f_p).is_none() {
                            valid_mvs.push(f_p);
                        }
                        if let Some(l_p) = left_pos && let Some(l_f) = self.pos(l_p) && l_f.side == Side::White {
                            valid_mvs.push(l_p);
                        }
                        if let Some(r_p) = right_pos && let Some(r_f) = self.pos(r_p) && r_f.side == Side::White {
                            valid_mvs.push(r_p);
                        }
                        if let Some(j_p) = jump_pos && self.pos(j_p).is_none() {
                            valid_mvs.push(j_p);
                        }
                    },
                }
                valid_mvs
            },
        }
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
