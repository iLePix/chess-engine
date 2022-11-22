use std::time::Instant;

use sdl2::{render::Canvas, pixels::Color, rect::Rect, video::Window};

use crate::{atlas::{TextureAtlas}, pieces::{Piece, PieceType, Side}};




pub struct Renderer<'a> {
    tex_atlas: &'a TextureAtlas<'a>,
    canvas: &'a mut Canvas<Window>,
    rects: Vec<(Rect, Color, i32)>,
    images: Vec<((PieceType, Side), Rect, i32)>,
    last_frame_time: Instant,
    s_tick: f32,
    s_tick_increment: f32
}


impl<'a> Renderer<'a> {
    pub fn new(tex_atlas: &'a TextureAtlas<'a>, s_tick_increment: f32, canvas: &'a mut Canvas<Window>) -> Self {
        Self {tex_atlas, rects: Vec::new(), images: Vec::new(), last_frame_time: Instant::now(), s_tick: 0.0, s_tick_increment, canvas}
    }
    
    pub fn draw_rect(&mut self, rect: Rect, color: Color, depth: i32) {
        self.rects.push((rect, color, depth));
    }

    pub fn draw_image(&mut self, piece_ty: PieceType, side: Side, dst: Rect, depth: i32) {
        self.images.push(((piece_ty, side), dst, depth));
    }

    pub fn render(&mut self) {
        let current_frame_time = Instant::now();
        let dt = (current_frame_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_frame_time;

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();


        //rendering rects
        for rect in &self.rects {
            self.canvas.set_draw_color(rect.1);
            self.canvas.fill_rect(rect.0).unwrap();
        }
        //rendering images
        for image in &self.images {
            self.canvas.copy(
                self.tex_atlas.pieces_texture, 
                self.tex_atlas.get_texture_by_piece_n_side(image.0.0, image.0.1), 
                image.1
            ).unwrap();
        }

        self.rects.clear();
        self.images.clear();
        self.canvas.present();
    }
}