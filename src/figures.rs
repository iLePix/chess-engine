
pub enum FigureType {
    QUEEN,
    KING,
    KNIGHT,
    BISHOP,
    ROOK,
    PAWN,
}

pub struct Figure {
    ty: FigureType,
    side: Side
}


//king, rook, bishop, queen, knight, and pawn.

pub enum Side {
    BLACK,
    WHITE
}