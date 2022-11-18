use std::collections::HashMap;

use sdl2::{render::Texture, rect::Rect};

use crate::{hashmap, count};



pub struct TextureAtlas<'a> {
    pub figure_atlas_cords: HashMap<i32, Rect>,
    pub pieces_texture: &'a Texture<'a>,
}

impl<'a> TextureAtlas<'a> {
    pub fn new(pieces_texture: &'a Texture, fig_src_size: u32) -> Self {
        Self { pieces_texture, figure_atlas_cords: Self::gen_fig_atlas_cords(fig_src_size)}
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
}