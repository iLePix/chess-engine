use sdl2::{rect::{Rect, Point}, pixels::Color};
use vecm::vec::{Vec2i, Vec2u};

use crate::{board::{Board, ColorTheme}, renderer::Renderer, pieces::{Side, Piece, PieceType}};




pub struct BoardRenderer<'a> {
    board_ground: Vec<(Rect, Color)>,
    hovering: Option<Vec2i>,
    field_size: u32,
    color_theme: &'a ColorTheme,
    pub selected: Option<Vec2i>,
    valid_mvs_tick: f32,
    last_move_tick: f32,
    animation_increment: f32,
    last_move: Option<(Vec2i, Vec2i)>
}


impl<'a>  BoardRenderer<'a> {
    pub fn new(field_size: u32, color_theme: &'a ColorTheme, size: Vec2u, animation_increment: f32) -> Self {
        let mut board_ground: Vec<(Rect, Color)> = Vec::new();
        let mut color;
        for x in 0..(size.x as i32) {
            for y in 0..(size.y as i32) {
                let rect = Rect::new(field_size as i32 * x, field_size as i32 * y, field_size, field_size);
                if (x % 2 == 1 && y % 2 == 0) || (x % 2 == 0 && y % 2 == 1) {
                    //color = black
                    color = color_theme.board_secondary;
                } else {
                    color = color_theme.board_primary;
                }
                board_ground.push((rect, color));
            }
        }
        Self {board_ground, hovering: None, selected: None, valid_mvs_tick: 0.0, last_move_tick: 0.0 , field_size, color_theme, animation_increment, last_move: None}
    }

    pub fn hover(&mut self, pos: Vec2i) {
        self.hovering = Some(pos);
    }


    pub fn render(&mut self, turn: &Side, board: &Board, renderer: &mut Renderer, dt: f32) {
        for rect in &self.board_ground {
            renderer.draw_rect(rect.0, rect.1, 0);
        }

        for (x, y_row) in  board.board.iter().enumerate() {
            for (y, optional_piece) in y_row.iter().enumerate() {
                    //draw last move 
                if let Some(last_move) = board.last_move {
                    if self.last_move_tick < self.field_size as f32 {
                        self.last_move_tick += self.animation_increment * dt;
                    } else {
                        self.last_move_tick = self.field_size as f32
                    }
                    let mut draw_move = |mv: Vec2i, color: Color| {
                        if mv == Vec2i::new(x as i32, y as i32) {
                            let r_size = Vec2u::fill(self.last_move_tick as u32);
                            let r_center = Vec2i::new(x as i32,y as i32) * self.field_size as i32 + Vec2i::fill(self.field_size as i32 / 2);
                            let rect = Rect::from_center(Point::new(r_center.x, r_center.y), r_size.x, r_size.y);
                            renderer.draw_rect(rect, color, 3)
                        }
                    };
                    draw_move(last_move.0, self.color_theme.last_move_primary);
                    draw_move(last_move.1, self.color_theme.last_move_secondary);
                }
                if board.last_move != self.last_move {
                    self.last_move_tick = 0.0;
                    self.last_move = board.last_move
                }
                //dont draw selection
                let field_pos = Vec2i::new(x as i32,y as i32);
                if let Some(selected) = self.selected {
                    if self.valid_mvs_tick < self.field_size as f32 * 0.75 {
                        self.valid_mvs_tick += self.animation_increment * dt;
                    } else {
                        self.valid_mvs_tick = self.field_size as f32 * 0.75
                    }

                    if selected == field_pos {
                        //if something selected then draw valid moves
                        let r_size = Vec2u::fill(self.valid_mvs_tick as u32);
                        if let Some(valid_moves) = board.valid_moves.get(&selected) {
                            for mv in valid_moves {
                                let r_center = *mv * self.field_size as i32 + Vec2i::fill(self.field_size as i32 / 2);
                                let rect = Rect::from_center(Point::new(r_center.x, r_center.y), r_size.x, r_size.y);
                                let color = self.color_theme.valid_moves;
                                renderer.draw_rect(rect, color, 0);
                            }
                        }
                        let r_center = field_pos * self.field_size as i32 + Vec2i::fill(self.field_size as i32 / 2);
                        let color = self.color_theme.selection;
                        let rect = Rect::from_center(Point::new(r_center.x, r_center.y), r_size.x, r_size.y);
                        renderer.draw_rect(rect, color, 0);

                        continue;
                    }
                } else {
                    self.valid_mvs_tick = 0.0;
                }




                //possible moves: depth = 1

                if let Some(piece) = optional_piece {
                    let mut window_pos = field_pos * self.field_size as i32;
                    let mut size = self.field_size;

                    
                    //draw check indicator
                    if piece.ty == PieceType::King {
                        let color = self.color_theme.check;
                        let rect = Rect::new(window_pos.x,window_pos.y, size, size);
                        if board.check.0 && piece.side == Side::White {
                            renderer.draw_rect(rect, color, 1);
                        } else if board.check.1 && piece.side == Side::Black {
                            renderer.draw_rect(rect, color, 1);
                        }
                    }



                    //hovering expands piece
                    if let Some(hover_pos) = self.hovering && self.selected.is_none() {
                        if &piece.side == turn && hover_pos.x == x as i32 && hover_pos.y == y as i32{
                            window_pos -= 5;
                            size += 10;
                        }
                    }

                    renderer.draw_image(
                        piece.ty,
                        piece.side,
                        Rect::new(window_pos.x,window_pos.y, size, size),
                        2
                    )           
                }
            }
        }
        self.hovering = None
    }

    pub fn unselect(&mut self) {
        self.selected = None
    }

    pub fn get_selected_piece(&self, board: &Board) -> Option<Piece> {
        if let Some(selected) = self.selected {
            return board.get_piece_at_pos(&selected)
        }
        None
    }

    pub fn update_color_theme(&mut self, color_theme: &'a ColorTheme) {
        let mut board_ground: Vec<(Rect, Color)> = Vec::new();
        let mut color;
        let field_size = self.field_size;
        let size = Vec2u::fill(8);
        for x in 0..(size.x as i32) {
            for y in 0..(size.y as i32) {
                let rect = Rect::new(field_size as i32 * x, field_size as i32 * y, field_size, field_size);
                if (x % 2 == 1 && y % 2 == 0) || (x % 2 == 0 && y % 2 == 1) {
                    //color = black
                    color = color_theme.board_secondary;
                } else {
                    color = color_theme.board_primary;
                }
                board_ground.push((rect, color));
            }
        }
        self.board_ground = board_ground;
        self.color_theme = color_theme;
    }

    pub fn select(&mut self, cursor_field: Vec2i, turn: Side, board: &Board) -> Option<Piece> {
        //previous selection
        if let Some(selection) = board.get_piece_at_pos(&cursor_field) {
            if selection.side == turn {
                self.selected = Some(cursor_field);
                return Some(selection)
            }
        }
        None
    }

}