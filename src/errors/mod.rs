// src/errors/mod.rs
#[derive(Debug)]
pub enum GameError {
    InvalidMove,
    PlayerNotFound,
    DeckInvalid,
    GameNotFound,
    EmptyDeck,
    InvalidTarget,
    NoValidCard,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidDeckSize,
    InvalidCardCount,
    InvalidPlayerState,
}
