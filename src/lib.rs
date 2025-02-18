pub mod collections;
pub mod database;
pub mod effects;
pub mod errors;
pub mod game_state;
pub mod models;
pub mod networking;

// Re-export commonly used items
pub use {
    collections::Collection,
    effects::{Effect, EffectTarget},
    errors::GameError,
    game_state::GameState,
    models::{Card, Deck, Player, Rarity},
};
