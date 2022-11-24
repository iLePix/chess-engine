use std::{ops::Not, fmt::{Display, write}};


#[derive(Clone, Copy, PartialEq, Debug)]
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Black,
    White
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Side::Black => "Black",
            Side::White =>  "White",
        })
    }
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}