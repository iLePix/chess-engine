use crate::board::FenError;

#[derive(Clone, Copy)]
pub struct Castle {
    pub short: bool,
    pub long: bool
}

impl Castle {
    pub fn new() -> Self {
        Self {short: true, long: true}
    }
    pub fn forbid() -> Self {
        Self {short: false, long: false}
    }

    //KQkq ->

    pub fn from_fen(fen: &str) -> Result<(Self, Self), FenError> {
        let mut black = Castle::forbid();
        let mut white = Castle::forbid();
        if fen == "-" { return Ok((white, black)) }
        for c in fen.chars() {
            match c {
                'K' => white.short = true,
                'Q' => white.long = true,
                'k' => black.short = true,
                'q' => black.long = true,
                _ => return Err(FenError::Castle)
            }
        }
        Ok((white, black)) 
    }

}
