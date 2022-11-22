use std::collections::HashMap;

use sdl2::{render::Texture, rect::Rect};
use vecm::vec::Vec2i;

use crate::{hashmap, count, pieces::{PieceType, Side}};

#[derive(Hash, PartialEq, Eq)]
enum PieceTexture {
    KingWhite,
    KingBlack,

    QueenWhite,
    QueenBlack,

    BishopWhite,
    BishopBlack,

    RookWhite,
    RookBlack,

    KnightWhite,
    KnightBlack,

    PawnWhite,
    PawnBlack
}

pub struct TextureAtlas<'a> {
    pub fig_src_size: u32,
    pub pieces_texture: &'a Texture<'a>,
}

impl<'a> TextureAtlas<'a> {
    pub fn new(pieces_texture: &'a Texture, fig_src_size: u32) -> Self {
        Self { pieces_texture, fig_src_size}
    }

    pub fn get_texture_by_piece_n_side(&self, piece: PieceType, side: Side) -> Rect {
        let lr_res_cords = match (piece, side) {
            (PieceType::Queen, Side::Black) => Vec2i::new(1,1),
            (PieceType::Queen, Side::White) => Vec2i::new(1,0),
            (PieceType::King, Side::Black) => Vec2i::new(0,1),
            (PieceType::King, Side::White) => Vec2i::new(0,0),
            (PieceType::Knight, Side::Black) => Vec2i::new(4,1),
            (PieceType::Knight, Side::White) => Vec2i::new(4,0),
            (PieceType::Bishop, Side::Black) => Vec2i::new(2,1),
            (PieceType::Bishop, Side::White) => Vec2i::new(2,0),
            (PieceType::Rook, Side::Black) => Vec2i::new(3,1),
            (PieceType::Rook, Side::White) => Vec2i::new(3,0),
            (PieceType::Pawn, Side::Black) => Vec2i::new(5,1),
            (PieceType::Pawn, Side::White) => Vec2i::new(5,0)
        };
        return Rect::new(lr_res_cords.x * self.fig_src_size as i32, lr_res_cords.y * self.fig_src_size as i32,self.fig_src_size, self.fig_src_size);
    }
}