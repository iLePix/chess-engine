use std::collections::HashMap;

use sdl2::{rect::Rect, pixels::Color, render::{Canvas, Texture}, video::Window};
use vecm::vec::Vec2i;


use crate::{figures::{Figure, Side, FigureType}, hashmap, count};







pub struct Board<'a> {
    white_rects: Vec<Rect>,
    black_rects: Vec<Rect>,
    figure_atlas_cords: HashMap<i32, Rect>,
    pieces_texture: &'a Texture<'a>,
    pos: [Option<Figure>; 64],
    hovering: u8,
    selected: u8
}


impl<'a> Board<'a> {
    pub fn new(pieces_texture: &'a Texture) -> Self {
        let mut white_rects: Vec<Rect> = vec![];
        let mut black_rects: Vec<Rect> = vec![];
        for i in 0..8 {
            for n in 0..8 {
                let rect = Rect::new(50 * i, 50 * n, 50, 50);
                let mut color = Color::WHITE;
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
            figure_atlas_cords: Self::gen_fig_atlas_cords(90), 
            pieces_texture, pos: Self::gen_start_pos(),
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

    fn gen_fig_atlas_cords(fig_size: u32) -> HashMap<i32, Rect> {
        let figure_atlas_lr_cords: HashMap<i32, (i32, i32)> = hashmap![

            0 => (0, 0), //king_white
            1 => (0, 1), //king_black

            2 => (1, 0), //queen_white
            3 => (1, 1), //queen_black

            4 => (2, 0), //bishop_white
            5 => (2, 1), //bishop_black

            6 => (3, 0), //rook_white
            7 => (3, 1), //rook_black

            8 => (4, 0), //knight_white
            9 => (4, 1), //knight_black

            10 => (5, 0), //pawn_white
            11 => (5, 1) //pawn_black

        ];

        figure_atlas_lr_cords.iter()
        .enumerate()
        .map(|(i, (k, lr_cords))| (*k, Rect::new((lr_cords.0 * fig_size as i32) as i32, (lr_cords.1 * fig_size as i32) as i32,fig_size, fig_size))).collect()
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(250,232,168,)); // rgba(250,232,168,255)
        canvas.fill_rects(&self.white_rects);

        canvas.set_draw_color(Color::RGB(20,95,75));
        canvas.fill_rects(&self.black_rects);

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
                let src = self.figure_atlas_cords.get(&f.tex_id)
                    .unwrap_or_else(|| panic!("Created figure with wrong index {}", f.tex_id));
                let dst = Rect::new(x,y, size as u32,size as u32);
                canvas.copy(self.pieces_texture, *src, dst);
            })

        /*let mut y = -(size as i32);
        let mut x = 0;
        self.figure_atlas_cords.iter()
        .enumerate()
        .for_each(|(i, (k,v))| {
            if i % 7 == 0 {y += 50}
            x = ((i % 7) * size) as i32;
            println!("Rendering {} at ({},{}) from {:?}", k, x, y, v);
            canvas.copy(self.pieces_texture, *v, Rect::new(x,y, 50,50));
        });*/

    }

    pub fn select(&mut self, i: u8) -> Option<Figure> { 
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

    pub fn unselect(&mut self) {
        self.select(255);
    }

    pub fn hover(&mut self, i: u8) {
        self.hovering = i;
    }

    /*pub fn x() -> &[] {

    }*/

}
