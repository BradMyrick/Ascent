// src/models/mod.rs
use crate::effects::{CostFilter, DrawFilter, Duration, Effect, EffectType};
use crate::errors::GameError;

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    Climber,
    Spell,
    Weapon,
    Trap,
    Gear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub name: String,
    pub cost: u32,
    pub power: u32,
    pub rarity: Rarity,
    pub effects: Vec<Effect>,
    pub card_type: CardType,
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub health: u32,
    pub hand: Vec<Card>,
    pub deck: Deck,
    pub mana: u32,
    pub position: Position,
    pub max_health: u32,
    pub power_boosts: Vec<(u32, Duration)>,
    pub health_boosts: Vec<(u32, Duration)>,
    pub active_effects: Vec<(EffectType, Duration)>,
    pub cards_played_this_turn: u32,
    pub mana_spent_this_turn: u32,
}

impl Player {
    pub fn new(name: String, deck: Deck) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            health: 30,
            max_health: 30,
            hand: Vec::new(),
            deck,
            mana: 0,
            position: Position::default(),
            power_boosts: Vec::new(),
            health_boosts: Vec::new(),
            active_effects: Vec::new(),
            cards_played_this_turn: 0,
            mana_spent_this_turn: 0,
        }
    }

    pub fn get_power(&self) -> u32 {
        let base_power: u32 = self.hand.iter().map(|card| card.power).sum();
        let boost_power: u32 = self.power_boosts.iter().map(|(amount, _)| amount).sum();
        base_power + boost_power
    }

    pub fn has_effect(&self, effect_type: &EffectType) -> bool {
        self.active_effects
            .iter()
            .any(|(effect, _)| effect == effect_type)
    }

    pub fn max_health(&self) -> u32 {
        let boost_health: u32 = self.health_boosts.iter().map(|(amount, _)| amount).sum();
        self.max_health + boost_health
    }

    pub fn draw_filtered(&mut self, filter: &DrawFilter) -> Result<(), GameError> {
        let card_position = self.deck.cards.iter().position(|card| match filter {
            DrawFilter::Cost(cost_filter) => match cost_filter {
                CostFilter::Equal(cost) => card.cost == *cost,
                CostFilter::LessThan(cost) => card.cost < *cost,
                CostFilter::GreaterThan(cost) => card.cost > *cost,
            },
            DrawFilter::Type(card_type) => card.card_type == *card_type,
            DrawFilter::Rarity(rarity) => card.rarity == *rarity,
        });

        match card_position {
            Some(pos) => {
                let card = self.deck.cards.remove(pos);
                self.hand.push(card);
                Ok(())
            }
            None => Err(GameError::NoValidCard),
        }
    }

    pub fn draw_card(&mut self) -> Result<(), GameError> {
        if self.deck.cards.is_empty() {
            return Err(GameError::EmptyDeck);
        }

        let card = self.deck.cards.remove(0);
        self.hand.push(card);
        Ok(())
    }

    pub fn add_power_boost(&mut self, amount: u32, duration: Duration) {
        self.power_boosts.push((amount, duration));
    }

    pub fn add_health_boost(&mut self, amount: u32, duration: Duration) {
        self.health_boosts.push((amount, duration));

        let new_max = self.max_health();
        self.health = self.health.min(new_max);
    }

    pub fn add_buff(&mut self, power: i32, health: i32, duration: Duration) {
        if power > 0 {
            self.add_power_boost(power as u32, duration.clone());
        }
        if health > 0 {
            self.add_health_boost(health as u32, duration);
        }
    }

    pub fn update_turn(&mut self) {
        self.cards_played_this_turn = 0;
        self.mana_spent_this_turn = 0;

        // Update durations and remove expired effects
        self.update_durations();
    }

    fn update_durations(&mut self) {
        // Update power boosts
        self.power_boosts.retain(|(_, duration)| match duration {
            Duration::Temporary(turns) => *turns > 0,
            Duration::UntilMountainLevel(_) => true,
            Duration::Permanent => true,
        });

        // Update health boosts
        self.health_boosts.retain(|(_, duration)| match duration {
            Duration::Temporary(turns) => *turns > 0,
            Duration::UntilMountainLevel(_) => true,
            Duration::Permanent => true,
        });

        // Update active effects
        self.active_effects.retain(|(_, duration)| match duration {
            Duration::Temporary(turns) => *turns > 0,
            Duration::UntilMountainLevel(_) => true,
            Duration::Permanent => true,
        });

        // Decrease temporary durations separately for each type
        for (_, duration) in self.power_boosts.iter_mut() {
            if let Duration::Temporary(turns) = duration {
                *turns = turns.saturating_sub(1);
            }
        }

        for (_, duration) in self.health_boosts.iter_mut() {
            if let Duration::Temporary(turns) = duration {
                *turns = turns.saturating_sub(1);
            }
        }

        for (_, duration) in self.active_effects.iter_mut() {
            if let Duration::Temporary(turns) = duration {
                *turns = turns.saturating_sub(1);
            }
        }
    }
}

// Mountain is our gameboard where the game is played
// it is made up of hexagonal tiles in elevated stages
#[derive(Debug, PartialEq)]
pub struct Mountain {
    pub tiles: Vec<Tile>,
    pub levels: u32,
}

#[derive(Debug, PartialEq)]
pub enum TileContent {
    Empty,
    Card(Card),
    Trap(Card),
    Player(Uuid),
}

#[derive(Debug, PartialEq)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub level: u32,
    pub content: TileContent,
}

impl Mountain {
    pub fn new(levels: u32) -> Self {
        if levels == 0 {
            panic!("Mountain must have at least one level");
        }
        if levels > 50 {
            panic!("Mountain's have a maximum height of 50")
        }
        let mut tiles = Vec::new();

        for level in 0..levels {
            for x in 0..=level {
                for y in 0..=level {
                    let z = (level as i32) - (x as i32) - (y as i32);
                    if z >= 0 {
                        let tile = Tile {
                            x,
                            y,
                            z: z as u32,
                            level,
                            content: TileContent::Empty,
                        };
                        tiles.push(tile);
                    }
                }
            }
        }

        Self { tiles, levels }
    }

    pub fn get_tile(&self, x: u32, y: u32, z: u32) -> Option<&Tile> {
        self.tiles
            .iter()
            .find(|tile| tile.x == x && tile.y == y && tile.z == z)
    }

    pub fn get_tile_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut Tile> {
        self.tiles
            .iter_mut()
            .find(|tile| tile.x == x && tile.y == y && tile.z == z)
    }

    pub fn get_neighbors(&self, x: u32, y: u32, z: u32) -> Vec<Position> {
        let mut neighbors = Vec::new();
        let directions = [
            (1, 0, -1),
            (1, -1, 0),
            (0, -1, 1),
            (-1, 0, 1),
            (-1, 1, 0),
            (0, 1, -1),
        ];

        for (dx, dy, dz) in directions {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;

            if nx >= 0 && ny >= 0 && nz >= 0 {
                let level = ((nx + ny + nz) / 3) as u32;
                if level <= self.levels {
                    neighbors.push(Position {
                        x: nx as u32,
                        y: ny as u32,
                        z: nz as u32,
                        level,
                    });
                }
            }
        }
        neighbors
    }

    pub fn calculate_distance(&self, pos1: Position, pos2: Position) -> u32 {
        let dx = (pos1.x as i32 - pos2.x as i32).abs();
        let dy = (pos1.y as i32 - pos2.y as i32).abs();
        let dz = (pos1.z as i32 - pos2.z as i32).abs();
        dx.max(dy).max(dz) as u32
    }

    pub fn is_valid_move(&self, current: Position, new: Position) -> bool {
        if self.get_tile(new.x, new.y, new.z).is_none()
            || self.get_tile(current.x, current.y, current.z).is_none()
        {
            return false;
        }

        let distance = self.calculate_distance(current, new);
        distance == 1 && new.level <= self.levels
    }

    pub fn get_tiles_in_range(&self, center: Position, range: u32) -> Vec<&Tile> {
        self.tiles
            .iter()
            .filter(|tile| {
                let pos = Position {
                    x: tile.x,
                    y: tile.y,
                    z: tile.z,
                    level: tile.level,
                };
                self.calculate_distance(center, pos) <= range
            })
            .collect()
    }

    pub fn get_level(&self, level: u32) -> Vec<&Tile> {
        self.tiles
            .iter()
            .filter(|tile| tile.level == level)
            .collect()
    }

    pub fn get_level_mut(&mut self, level: u32) -> Vec<&mut Tile> {
        self.tiles
            .iter_mut()
            .filter(|tile| tile.level == level)
            .collect()
    }
}

// TESTS
#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn test_draw_card() {
        let card = Card {
            id: Uuid::new_v4(),
            name: "Test Card".to_string(),
            cost: 1,
            power: 1,
            rarity: Rarity::Common,
            effects: vec![],
            card_type: CardType::Climber,
        };

        let deck = Deck {
            cards: vec![card.clone()],
            owner_id: Uuid::new_v4(),
        };

        let mut player = Player::new("Test Player".to_string(), deck);

        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.deck.cards.len(), 1);

        player.draw_card().unwrap();

        assert_eq!(player.hand.len(), 1);
        assert_eq!(player.deck.cards.len(), 0);
    }

    #[test]
    fn test_draw_card_empty_deck() {
        let deck = Deck {
            cards: vec![],
            owner_id: Uuid::new_v4(),
        };

        let mut player = Player::new("Test Player".to_string(), deck);

        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.deck.cards.len(), 0);

        let result = player.draw_card();

        assert!(result.is_err());
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.deck.cards.len(), 0);
    }

    #[test]
    fn test_player_new() {
        let deck = Deck {
            cards: vec![],
            owner_id: Uuid::new_v4(),
        };

        let player = Player::new("Test Player".to_string(), deck);

        assert_eq!(player.health, 30);
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.mana, 0);
    }

    #[test]
    fn test_player_new_with_cards() {
        let card = Card {
            id: Uuid::new_v4(),
            name: "Test Card".to_string(),
            cost: 1,
            power: 1,
            rarity: Rarity::Common,
            effects: vec![],
            card_type: CardType::Climber,
        };

        let deck = Deck {
            cards: vec![card.clone()],
            owner_id: Uuid::new_v4(),
        };

        let player = Player::new("Test Player".to_string(), deck);

        assert_eq!(player.health, 30);
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.mana, 0);
    }

    #[test]
    fn test_mountain_movement() {
        let mountain = Mountain::new(3);

        let start = Position {
            x: 0,
            y: 0,
            z: 0,
            level: 0,
        };
        let valid_move = Position {
            x: 1,
            y: 0,
            z: 0,
            level: 0,
        };
        let invalid_move = Position {
            x: 2,
            y: 2,
            z: 2,
            level: 0,
        };

        assert!(mountain.is_valid_move(start, valid_move));
        assert!(!mountain.is_valid_move(start, invalid_move));
    }

    #[test]
    fn test_range_calculation() {
        let mountain = Mountain::new(3);

        let pos1 = Position {
            x: 0,
            y: 0,
            z: 0,
            level: 0,
        };
        let pos2 = Position {
            x: 1,
            y: 1,
            z: 0,
            level: 0,
        };

        assert_eq!(mountain.calculate_distance(pos1, pos2), 1);
    }

    #[test]
    fn test_tiles_in_range() {
        let mountain = Mountain::new(3);
        let center = Position {
            x: 1,
            y: 0,
            z: 1,
            level: 2,
        };

        let tiles_range_1 = mountain.get_tiles_in_range(center, 1);
        let tiles_range_2 = mountain.get_tiles_in_range(center, 2);

        assert!(tiles_range_1.len() < tiles_range_2.len());
    }
}
