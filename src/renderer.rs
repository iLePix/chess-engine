use std::time::Instant;

use sdl2::{render::Canvas, pixels::Color, rect::Rect, video::Window};

use crate::atlas::TextureAtlas;




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