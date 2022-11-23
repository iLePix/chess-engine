use std::{error::Error, io::{Read, Write}};

use binverse::{streams::{Serializer, Deserializer}, serialize::{Serialize, Deserialize}, error::BinverseError};
use binverse_derive::serializable;

#[serializable]
pub struct PlayerInfo {
    pub name: String,
}

#[serializable]
#[derive(Debug)]
pub struct Move {
    pub x1: i8,
    pub y1: i8,
    pub x2: i8,
    pub y2: i8,
}

#[serializable]
pub struct GameInfo {
    pub other_player: String,
    pub is_black: bool,
}

pub fn send<T: Serialize<W>, W: Write>(p: W, t: T) -> Result<(), BinverseError> {
    let mut s = Serializer::new_no_revision(p);
    t.serialize(&mut s)
}
pub fn recv<T: Deserialize<R>, R: Read>(p: R) -> Result<T, BinverseError> {
    Deserializer::new_no_revision(p, 0).deserialize()
}