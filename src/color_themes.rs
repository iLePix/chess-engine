use sdl2::pixels::Color;



#[derive(Clone, Copy)]
pub struct ColorTheme {
    pub board_primary: Color,
    pub board_secondary: Color,
    pub valid_moves: Color,
    pub selection: Color,
    pub check: Color,
    pub last_move_primary: Color,
    pub last_move_secondary: Color,
    pub progress: Color,
}

impl ColorTheme {
    pub fn new(board_primary: Color, board_secondary: Color, valid_moves: Color, selection: Color, check: Color, last_move: Color, last_move_primary: Color,  last_move_secondary: Color, progress: Color) -> Self {
        Self {board_primary, board_secondary, valid_moves, selection, check, last_move_primary,  last_move_secondary, progress}
    }

    pub fn blue_theme() -> Self {
        Self {
            board_primary: Color::WHITE,
            board_secondary: Color::RGB(13,56,166),
            valid_moves: Color::RGBA(3, 138, 255, 128),
            selection: Color::RGBA(255, 123, 98, 200),
            last_move_primary: Color::RGB(169,202,142),
            last_move_secondary: Color::RGB(124,172,112),
            check: Color::RGB(230,55,96),
            progress: Color::RGB(46, 204, 113)
        }
    }

    pub fn green_theme() -> Self {
        Self {
            board_primary: Color::RGB(238,238,210),
            board_secondary: Color::RGB(118,150,86),
            valid_moves: Color::RGBA(3, 138, 255, 128),
            selection: Color::RGBA(255, 123, 98, 200),
            last_move_primary: Color::RGB(226,242,108),
            last_move_secondary: Color::RGB(186,202,68),
            check: Color::RGB(230,55,96),
            progress: Color::RGB(46, 204, 113)
        }
    }

    pub fn red_theme() -> Self {
        Self {
            board_primary: Color::WHITE,
            board_secondary: Color::RGB(230,55,96),
            valid_moves: Color::RGBA(3, 138, 255, 128),
            selection: Color::RGBA(255, 123, 98, 200),
            last_move_primary: Color::RGB(226,242,108),
            last_move_secondary: Color::RGB(186,202,68),
            check: Color::RGB(13,56,166),
            progress: Color::RGB(46, 204, 113)
        }
    }


}
