
#[derive(Clone, Copy, PartialEq)]
pub enum FigureType {
    Queen,
    King,
    Knight,
    Bishop,
    Rook,
    Pawn,
}


#[derive(Clone, Copy)]
pub struct Figure {
    pub ty: FigureType,
    pub side: Side,
    pub tex_id: i32, 
}

impl Figure  {
    pub fn new(ty: FigureType, side: Side, tex_id: i32) -> Self {
        Self { ty, side, tex_id}
    }
}


//king, rook, bishop, queen, knight, and pawn.

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Black,
    White
}