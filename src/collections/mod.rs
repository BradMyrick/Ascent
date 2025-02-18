// src/collections/mod.rs
use crate::models::Deck;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct Collection {
    pub owner_id: Uuid,
    pub cards: HashSet<Uuid>,
    pub decks: HashMap<String, Deck>,
}

impl Collection {
    pub fn new(owner_id: Uuid) -> Self {
        Self {
            owner_id,
            cards: HashSet::new(),
            decks: HashMap::new(),
        }
    }
}
