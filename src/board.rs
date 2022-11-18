use std::collections::HashMap;

use sdl2::{rect::Rect, pixels::Color, render::{Canvas, Texture}, video::Window};
use vecm::vec::Vec2i;


use crate::{figures::{Figure, Side, FigureType}, hashmap, count, atlas::TextureAtlas};







pub struct Board<'a> {
    white_rects: Vec<Rect>,
    black_rects: Vec<Rect>,
    tex_atlas: &'a TextureAtlas<'a>,
    pos: [Option<Figure>; 64],
    hovering: u8,
    selected: u8
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
            hovering: 64,
            selected: 64
        }
    }

    fn gen_start_pos() -> [Option<Figure>; 64] {
        let mut start_pos: [Option<Figure>; 64] = [None; 64];

        //gen black
        let mut side = Side::BLACK;
        //pawns
        for i in 8..=15 {
            start_pos[i] = Some(Figure::new(FigureType::PAWN, side, 11));
        }
        //knights
        start_pos[0] = Some(Figure::new(FigureType::KNIGHT, side, 9));
        start_pos[7] = Some(Figure::new(FigureType::KNIGHT, side, 9));

        //rooks
        start_pos[1] = Some(Figure::new(FigureType::ROOK, side, 7));
        start_pos[6] = Some(Figure::new(FigureType::ROOK, side, 7));

        //bishops
        start_pos[2] = Some(Figure::new(FigureType::BISHOP, side, 5));
        start_pos[5] = Some(Figure::new(FigureType::BISHOP, side, 5));

        //king & queen
        start_pos[3] = Some(Figure::new(FigureType::QUEEN, side, 3));
        start_pos[4] = Some(Figure::new(FigureType::KING, side, 1));

        //gen white
        side = Side::WHITE;
        //pawns
        for i in 48..=55 {
            start_pos[i] = Some(Figure::new(FigureType::PAWN, side, 10));
        }
        //knights
        start_pos[56] = Some(Figure::new(FigureType::KNIGHT, side, 8));
        start_pos[63] = Some(Figure::new(FigureType::KNIGHT, side, 8));

        //rooks
        start_pos[57] = Some(Figure::new(FigureType::ROOK, side, 6));
        start_pos[62] = Some(Figure::new(FigureType::ROOK, side, 6));

        //bishops
        start_pos[58] = Some(Figure::new(FigureType::BISHOP, side, 4));
        start_pos[61] = Some(Figure::new(FigureType::BISHOP, side, 4));

        //king & queen
        start_pos[59] = Some(Figure::new(FigureType::QUEEN, side, 2));
        start_pos[60] = Some(Figure::new(FigureType::KING, side, 0));


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
                if f.side == Side::WHITE {
                    if self.selected == i as u8 {
                        return
                    }
                    if self.hovering == i as u8 {
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
        if i == self.selected { // FOR PUTTING BACK - BUT we only works when mouse has only been pressed once
            self.selected = 255;
            return None;
        }
        self.selected = i;
        if i < 64 {
            if let Some(f) = self.pos[i as usize] {
                if f.side == Side::WHITE {
                    return Some(f)
                }
            }
        }
        None
    }

    pub fn move_figure(&mut self, dst: u8) -> bool {
        if dst < 64 {
            if dst == self.selected {
                return false
            }
            self.pos[dst as usize] = self.pos[self.selected as usize];
            self.pos[self.selected as usize] = None;
            self.unselect();
        }
        true //bc every move is allowed rn
    }

    pub fn unselect(&mut self) {
        self.selected = 255;
    }

    pub fn hover(&mut self, i: u8) {
        self.hovering = i;
    }

    /*pub fn x() -> &[] {

    }*/

}
