// src/game_state/mod.rs
use crate::errors::GameError;
use crate::models::{Mountain, Player, Position};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: HashMap<Uuid, Player>,
    pub active_player: Uuid,
    pub turn_number: u32,
    pub mountain: Mountain,
}

impl GameState {
    pub fn new(player1: Player, player2: Player) -> Self {
        let mut players = HashMap::new();
        let p1_id = player1.id;
        players.insert(player1.id, player1);
        players.insert(player2.id, player2);

        Self {
            game_id: Uuid::new_v4(),
            players,
            active_player: p1_id,
            turn_number: 1,
            mountain: Mountain::new(7),
        }
    }

    pub fn move_player(
        &mut self,
        player_id: Uuid,
        new_position: Position,
    ) -> Result<(), GameError> {
        let current_position = self
            .players
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?
            .position;

        if !self.mountain.is_valid_move(current_position, new_position) {
            return Err(GameError::InvalidMove);
        }

        if let Some(player) = self.players.get_mut(&player_id) {
            player.position = new_position;
            Ok(())
        } else {
            Err(GameError::PlayerNotFound)
        }
    }
}

//TESTS

#[cfg(test)]
mod game_state_tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let player1 = Player::new(
            "Player 1".to_string(),
            crate::Deck {
                cards: vec![],
                owner_id: Uuid::new_v4(),
            },
        );
        let player2 = Player::new(
            "Player 2".to_string(),
            crate::Deck {
                cards: vec![],
                owner_id: Uuid::new_v4(),
            },
        );
        let game_state = GameState::new(player1.clone(), player2);

        assert_eq!(game_state.players.len(), 2);
        assert_eq!(game_state.active_player, player1.id);
        assert_eq!(game_state.turn_number, 1);
    }
}
