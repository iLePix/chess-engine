use std::collections::HashMap;

use sdl2::{rect::Rect, pixels::Color, render::{Canvas, Texture}, video::Window};
use vecm::vec::Vec2i;


use crate::{figures::Figure, hashmap};







pub struct Board<'a> {
    white_rects: Vec<Rect>,
    black_rects: Vec<Rect>,
    figure_atlas_cords: HashMap<String, Rect>,
    pieces_texture: &'a Texture<'a>
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



        Self {white_rects, black_rects, figure_atlas_cords: Self::gen_fig_atlas_cords(90), pieces_texture}
    }

    fn gen_fig_atlas_cords(fig_size: u32) -> HashMap<String, Rect> {
        let figure_atlas_lr_cords: HashMap<&str, (i32, i32)> = hashmap![

            "king_white" => (0, 0),
            "king_black" => (0, 1),

            "queen_white" => (1, 0),
            "queen_black" => (1, 1),

            "bishop_white" => (2, 0),
            "bishop_black" => (2, 1),

            "rook_white" => (3, 0),
            "rook_black" => (3, 1),

            "knight_white" => (4, 0),
            "knight_black" => (4, 1),

            "pawn_white" => (5, 0),
            "pawn_black" => (5, 1)

        ];

        figure_atlas_lr_cords.iter()
        .enumerate()
        .map(|(i, (k, lr_cords))| (k.to_string(), Rect::new((lr_cords.0 * fig_size as i32) as i32, (lr_cords.1 * fig_size as i32) as i32,fig_size, fig_size))).collect()
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(250,232,168,)); // rgba(250,232,168,255)
        canvas.fill_rects(&self.white_rects);

        canvas.set_draw_color(Color::RGB(20,95,75));
        canvas.fill_rects(&self.black_rects);

        //draw figures
        let size = 50;
        let mut y = -(size as i32);
        let mut x = 0;
        self.figure_atlas_cords.iter()
        .enumerate()
        .for_each(|(i, (k,v))| {
            if i % 7 == 0 {y += 50}
            x = ((i % 7) * size) as i32;
            println!("Rendering {} at ({},{}) from {:?}", k, x, y, v);
            canvas.copy(self.pieces_texture, *v, Rect::new(x,y, 50,50));
        });

    }
}
