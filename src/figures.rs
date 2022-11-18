
#[derive(Clone, Copy)]
pub enum FigureType {
    QUEEN,
    KING,
    KNIGHT,
    BISHOP,
    ROOK,
    PAWN,
}


#[derive(Clone, Copy)]
pub struct Figure {
    ty: FigureType,
    side: Side,
    pub tex_id: i32, 
}

impl Figure  {
    pub fn new(ty: FigureType, side: Side, tex_id: i32) -> Self {
        Self { ty, side, tex_id}
    }
}


//king, rook, bishop, queen, knight, and pawn.

#[derive(Clone, Copy)]
pub enum Side {
    BLACK,
    WHITE
}