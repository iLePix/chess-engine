
#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    Queen,
    King,
    Knight,
    Bishop,
    Rook,
    Pawn,
}


#[derive(Clone, Copy)]
pub struct Piece {
    pub ty: PieceType,
    pub side: Side,
}

impl Piece  {
    pub fn new(ty: PieceType, side: Side) -> Self {
        Self { ty, side}
    }
}


//king, rook, bishop, queen, knight, and pawn.

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Black,
    White
}