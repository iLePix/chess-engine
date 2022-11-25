use sdl2::{rect::{Rect, Point}, pixels::Color};
use vecm::vec::{Vec2i, Vec2u};

use crate::{board::Board, renderer::Renderer, pieces::{Side, Piece, PieceType}, color_themes::ColorTheme, game::Game};




pub struct BoardRenderer {
    board_ground: Vec<(Rect, Color)>,
    hovering: Option<Vec2i>,
    field_size: u32,
    themes: [ColorTheme; 3],
    theme_index: usize,
    pub selected: Option<Vec2i>,
    mouse_pos: Vec2u,
    valid_mvs_tick: f32,
    last_move_tick: f32,
    s_tick: f32,
    animation_increment: f32,
    last_move: Option<(Vec2i, Vec2i)>
}


impl  BoardRenderer{
    pub fn new(field_size: u32, size: Vec2u, animation_increment: f32) -> Self {
        let mut board_ground: Vec<(Rect, Color)> = Vec::new();
        let theme_index = 0;
        let mut color;
        let themes = [ColorTheme::blue_theme(), ColorTheme::green_theme(), ColorTheme::red_theme()];
        for x in 0..(size.x as i32) {
            for y in 0..(size.y as i32) {
                let rect = Rect::new(field_size as i32 * x, field_size as i32 * y, field_size, field_size);
                if (x % 2 == 1 && y % 2 == 0) || (x % 2 == 0 && y % 2 == 1) {
                    //color = black
                    color = themes[theme_index].board_secondary;
                } else {
                    color = themes[theme_index].board_primary;
                }
                board_ground.push((rect, color));
            }
        }
        Self {board_ground, 
            hovering: None, 
            selected: None,
            valid_mvs_tick: 0.0,
            last_move_tick: 0.0,
            field_size, 
            themes,
            theme_index,
            animation_increment, 
            last_move: None, 
            mouse_pos: Vec2u::zero(), 
            s_tick: 0.0
        }
    }

    pub fn hover(&mut self, pos: Vec2i) {
        self.hovering = Some(pos);
    }


    pub fn render(&mut self, game: &Game, renderer: &mut Renderer, dt: f32) {
        let turn = game.turn;
        for rect in &self.board_ground {
            renderer.draw_rect(rect.0, rect.1, 0);
        }

        for (x, y_row) in  game.board.board.iter().enumerate() {
            for (y, optional_piece) in y_row.iter().enumerate() {
                    //draw last move 
                if let Some(last_move) = game.board.last_move {
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
                    draw_move(last_move.0, self.color_theme().last_move_primary);
                    draw_move(last_move.1, self.color_theme().last_move_secondary);
                }
                if game.board.last_move != self.last_move {
                    self.last_move_tick = 0.0;
                    self.last_move = game.board.last_move
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
                        if let Some(valid_moves) = game.board.valid_moves.get(&selected) {
                            for mv in valid_moves {
                                let r_center = *mv * self.field_size as i32 + Vec2i::fill(self.field_size as i32 / 2);
                                let rect = Rect::from_center(Point::new(r_center.x, r_center.y), r_size.x, r_size.y);
                                let color = self.color_theme().valid_moves;
                                renderer.draw_rect(rect, color, 0);
                            }
                        }
                        let r_center = field_pos * self.field_size as i32 + Vec2i::fill(self.field_size as i32 / 2);
                        let color = self.color_theme().selection;
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
                        let color = self.color_theme().check;
                        let rect = Rect::new(window_pos.x,window_pos.y, size, size);
                        if game.board.check.0 && piece.side == Side::White {
                            renderer.draw_rect(rect, color, 1);
                        } else if game.board.check.1 && piece.side == Side::Black {
                            renderer.draw_rect(rect, color, 1);
                        }
                    }



                    //hovering expands piece
                    if let Some(hover_pos) = self.hovering && self.selected.is_none() {
                        if piece.side == turn && hover_pos.x == x as i32 && hover_pos.y == y as i32{
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

        if let Some(piece) = self.get_selected_piece(&game.board) {
            fn parabola(x: i32) -> f32 {
                -1.0 * (0.125 * x as f32 - 16.0).powi(2) + 256.0
            }


            if self.s_tick + self.animation_increment * 100.0 *dt >= 255.0 {
                self.s_tick = 0.0;
            } else {
                self.s_tick += dt * self.animation_increment * 100.0 as f32;
            }
            let p = (parabola(self.s_tick as i32) / 20.0);
            let size = self.field_size + p as u32;
            let dst = Rect::from_center(Point::new(self.mouse_pos.x as i32, self.mouse_pos.y as i32), size, size);
            renderer.draw_image(piece.ty, piece.side, dst, 3);
        } else {
            self.s_tick = 0.0;
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

    pub fn color_theme(&self) -> ColorTheme {
        self.themes[self.theme_index]
    }

    pub fn next_theme(&mut self) {
        if self.theme_index < self.themes.len() - 1 {
            self.theme_index += 1;
        } else {
            self.theme_index = 0;
        }
        self.update_theme();
    }

    pub fn update_theme(&mut self) {
        let mut board_ground: Vec<(Rect, Color)> = Vec::new();
        let mut color;
        let theme = self.themes[self.theme_index];
        let field_size = self.field_size;
        let size = Vec2u::fill(8);
        for x in 0..(size.x as i32) {
            for y in 0..(size.y as i32) {
                let rect = Rect::new(field_size as i32 * x, field_size as i32 * y, field_size, field_size);
                if (x % 2 == 1 && y % 2 == 0) || (x % 2 == 0 && y % 2 == 1) {
                    //color = black
                    color = theme.board_secondary;
                } else {
                    color = theme.board_primary;
                }
                board_ground.push((rect, color));
            }
        }
        self.board_ground = board_ground;
    }

    pub fn select(&mut self, cursor_field: Vec2i, turn: Side, board: &Board) -> Option<Piece> {
        //previous selection
        if let Some(selected) = self.selected && cursor_field == selected {
            self.unselect();
            return None
        }


        if let Some(selection) = board.get_piece_at_pos(&cursor_field) {
            if selection.side == turn {
                self.selected = Some(cursor_field);
                return Some(selection)
            }
        }
        None
    }

    pub fn update_mouse_pos(&mut self, mouse_pos: Vec2u) {
        self.mouse_pos = mouse_pos;
    }
}