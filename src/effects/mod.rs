// src/effects/mod.rs
use crate::errors::GameError;
use crate::game_state::GameState;
use crate::models::{CardType, Rarity};
use rand::prelude::IndexedRandom;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    Damage,
    Heal,
    Draw,
    Boost,
    Buff,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Damage(DamageEffect),
    Heal(HealEffect),
    Draw(DrawEffect),
    Boost(BoostEffect),
    BuffStats(BuffEffect),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectValue {
    pub base: u32,
    pub scaling: Option<ScalingFactor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalingFactor {
    MountainLevel(f32), // Scales with mountain level
    CardsInHand(f32),   // Scales with number of cards in hand
    CardsPlayed(f32),   // Scales with cards played this turn
    ManaSpent(f32),     // Scales with mana spent this turn
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectTarget {
    Self_,                   // The card that played the effect
    Specific(Uuid),          // A specific target by UUID
    Multiple(HashSet<Uuid>), // Multiple specific targets
    AllPlayers(Vec<Uuid>),   // All players in the game
    Random(u32),             // Random number of targets
    Adjacent,                // Cards adjacent to the target
    Area {
        // Area of effect tile targeting
        center: Uuid,
        radius: u32,
    },
    Conditional {
        // Targets that meet certain conditions
        condition: TargetCondition,
        max_targets: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetCondition {
    PowerGreaterThan(u32),
    PowerLessThan(u32),
    HasEffect(EffectType),
    IsRarity(Rarity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageEffect {
    pub value: EffectValue,
    pub target: EffectTarget,
    pub penetrating: bool, // Ignores shields/armor
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealEffect {
    pub value: EffectValue,
    pub target: EffectTarget,
    pub over_heal: bool, // Can heal beyond max health
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawEffect {
    pub cards: u32,
    pub target: EffectTarget,
    pub filter: Option<DrawFilter>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoostEffect {
    pub value: EffectValue,
    pub target: EffectTarget,
    pub stat: BoostType,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuffEffect {
    pub power: i32, // Can be negative for debuffs
    pub health: i32,
    pub target: EffectTarget,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoostType {
    Power,
    Health,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Duration {
    Temporary(u32),          // Lasts for X turns
    UntilMountainLevel(u32), // Lasts until reaching specific mountain level
    Permanent,               // Lasts for the rest of the game
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawFilter {
    Cost(CostFilter),
    Type(CardType),
    Rarity(Rarity),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CostFilter {
    Equal(u32),
    LessThan(u32),
    GreaterThan(u32),
}

impl Effect {
    pub fn apply(&self, game_state: &mut GameState, source: Uuid) -> Result<(), GameError> {
        match self {
            Effect::Damage(damage_effect) => {
                let targets = resolve_targets(&damage_effect.target, game_state, source)?;
                for target in targets {
                    apply_damage(game_state, target, &damage_effect.value)?;
                }
            }
            Effect::Heal(heal_effect) => {
                let targets = resolve_targets(&heal_effect.target, game_state, source)?;
                for target in targets {
                    apply_heal(
                        game_state,
                        target,
                        &heal_effect.value,
                        heal_effect.over_heal,
                    )?;
                }
            }
            Effect::Draw(draw_effect) => {
                let targets = resolve_targets(&draw_effect.target, game_state, source)?;
                for target in targets {
                    apply_draw(game_state, target, draw_effect.cards, &draw_effect.filter)?;
                }
            }
            Effect::Boost(boost_effect) => {
                let targets = resolve_targets(&boost_effect.target, game_state, source)?;
                for target in targets {
                    apply_boost(game_state, target, boost_effect)?;
                }
            }
            Effect::BuffStats(buff_effect) => {
                let targets = resolve_targets(&buff_effect.target, game_state, source)?;
                for target in targets {
                    apply_buff(game_state, target, buff_effect)?;
                }
            }
        }
        Ok(())
    }
}

fn resolve_targets(
    target: &EffectTarget,
    game_state: &GameState,
    source: Uuid,
) -> Result<Vec<Uuid>, GameError> {
    match target {
        EffectTarget::Self_ => Ok(vec![source]),
        EffectTarget::Specific(uuid) => Ok(vec![*uuid]),
        EffectTarget::Multiple(targets) => Ok(targets.iter().copied().collect()),
        EffectTarget::AllPlayers(players) => Ok(players.clone()),
        EffectTarget::Random(n) => {
            let available_targets: Vec<_> = game_state.players.keys().copied().collect();
            if available_targets.len() < *n as usize {
                return Err(GameError::InvalidTarget);
            }
            let mut rng = rand::rng();
            Ok(available_targets
                .choose_multiple(&mut rng, *n as usize)
                .copied()
                .collect())
        }
        EffectTarget::Adjacent => {
            let source_pos = game_state
                .players
                .get(&source)
                .ok_or(GameError::PlayerNotFound)?
                .position;

            let adjacent_positions =
                game_state
                    .mountain
                    .get_neighbors(source_pos.x, source_pos.y, source_pos.z);

            Ok(game_state
                .players
                .iter()
                .filter(|(_, player)| adjacent_positions.contains(&player.position))
                .map(|(id, _)| *id)
                .collect())
        }
        EffectTarget::Area { center, radius } => {
            let center_pos = game_state
                .players
                .get(center)
                .ok_or(GameError::PlayerNotFound)?
                .position;

            Ok(game_state
                .players
                .iter()
                .filter(|(_, player)| {
                    game_state
                        .mountain
                        .calculate_distance(center_pos, player.position)
                        <= *radius
                })
                .map(|(id, _)| *id)
                .collect())
        }
        EffectTarget::Conditional {
            condition,
            max_targets,
        } => {
            let mut valid_targets: Vec<Uuid> = match condition {
                TargetCondition::PowerGreaterThan(threshold) => game_state
                    .players
                    .iter()
                    .filter(|(_, player)| player.get_power() > *threshold)
                    .map(|(id, _)| *id)
                    .collect(),
                TargetCondition::PowerLessThan(threshold) => game_state
                    .players
                    .iter()
                    .filter(|(_, player)| player.get_power() < *threshold)
                    .map(|(id, _)| *id)
                    .collect(),
                TargetCondition::HasEffect(effect_type) => game_state
                    .players
                    .iter()
                    .filter(|(_, player)| player.has_effect(effect_type))
                    .map(|(id, _)| *id)
                    .collect(),
                TargetCondition::IsRarity(rarity) => game_state
                    .players
                    .iter()
                    .filter(|(_, player)| player.hand.iter().any(|card| card.rarity >= *rarity))
                    .map(|(id, _)| *id)
                    .collect(),
            };

            if let Some(max) = max_targets {
                valid_targets.truncate(*max as usize);
            }

            Ok(valid_targets)
        }
    }
}

fn apply_heal(
    game_state: &mut GameState,
    target: Uuid,
    value: &EffectValue,
    over_heal: bool,
) -> Result<(), GameError> {
    let heal = calculate_value(value, game_state, target);

    let target_player = game_state
        .players
        .get_mut(&target)
        .ok_or(GameError::PlayerNotFound)?;

    if heal == 0 {
        return Ok(());
    }

    if over_heal {
        target_player.health += heal;
    } else {
        target_player.health = (target_player.health + heal).min(target_player.max_health());
    }
    Ok(())
}

fn apply_draw(
    game_state: &mut GameState,
    target: Uuid,
    cards: u32,
    filter: &Option<DrawFilter>,
) -> Result<(), GameError> {
    let player = game_state
        .players
        .get_mut(&target)
        .ok_or(GameError::PlayerNotFound)?;

    for _ in 0..cards {
        if let Some(filter) = filter {
            player.draw_filtered(filter)?;
        } else {
            player.draw_card()?;
        }
    }

    Ok(())
}

fn apply_boost(
    game_state: &mut GameState,
    target: Uuid,
    boost_effect: &BoostEffect,
) -> Result<(), GameError> {
    let boost_amount = calculate_value(&boost_effect.value, game_state, target);

    let player = game_state
        .players
        .get_mut(&target)
        .ok_or(GameError::PlayerNotFound)?;

    match boost_effect.stat {
        BoostType::Power => player.add_power_boost(boost_amount, boost_effect.duration.clone()),
        BoostType::Health => player.add_health_boost(boost_amount, boost_effect.duration.clone()),
        BoostType::Both => {
            player.add_power_boost(boost_amount, boost_effect.duration.clone());
            player.add_health_boost(boost_amount, boost_effect.duration.clone());
        }
    }

    Ok(())
}

fn apply_buff(
    game_state: &mut GameState,
    target: Uuid,
    buff_effect: &BuffEffect,
) -> Result<(), GameError> {
    let player = game_state
        .players
        .get_mut(&target)
        .ok_or(GameError::PlayerNotFound)?;

    player.add_buff(
        buff_effect.power,
        buff_effect.health,
        buff_effect.duration.clone(),
    );

    Ok(())
}

fn calculate_value(value: &EffectValue, game_state: &GameState, target: Uuid) -> u32 {
    let base = value.base;

    if let Some(scaling) = &value.scaling {
        let scaling_factor = match scaling {
            ScalingFactor::MountainLevel(factor) => *factor * game_state.mountain.levels as f32,
            ScalingFactor::CardsInHand(factor) => {
                if let Some(player) = game_state.players.get(&target) {
                    *factor * player.hand.len() as f32
                } else {
                    0.0
                }
            }
            ScalingFactor::CardsPlayed(factor) => {
                if let Some(player) = game_state.players.get(&target) {
                    *factor * player.cards_played_this_turn as f32
                } else {
                    0.0
                }
            }
            ScalingFactor::ManaSpent(factor) => {
                if let Some(player) = game_state.players.get(&target) {
                    *factor * player.mana_spent_this_turn as f32
                } else {
                    0.0
                }
            }
        };
        base + scaling_factor as u32
    } else {
        base
    }
}

fn apply_damage(
    game_state: &mut GameState,
    target: Uuid,
    value: &EffectValue,
) -> Result<(), GameError> {
    let target_player = game_state
        .players
        .get_mut(&target)
        .ok_or(GameError::PlayerNotFound)?;
    let damage = value.base;
    if damage == 0 {
        return Ok(());
    }
    // Use saturating_sub to prevent underflow
    target_player.health = target_player.health.saturating_sub(damage);
    Ok(())
}

// TESTS
#[cfg(test)]
mod effect_tests {
    use super::*;
    use crate::models::{Card, Deck, Player, Position, Rarity};

    #[test]
    fn test_apply_damage() {
        let player1 = Player {
            id: Uuid::new_v4(),
            name: "Player 1".to_string(),
            health: 30,
            hand: vec![],
            deck: Deck {
                cards: vec![],
                owner_id: Uuid::new_v4(),
            },
            mana: 0,
            position: Position {
                x: 0,
                y: 0,
                z: 0,
                level: 0,
            },
            max_health: 30,
            active_effects: vec![],
            cards_played_this_turn: 0,
            health_boosts: vec![],
            power_boosts: vec![],
            mana_spent_this_turn: 0,
        };
        let player2 = Player {
            id: Uuid::new_v4(),
            name: "Player 2".to_string(),
            health: 30,
            hand: vec![],
            deck: Deck {
                cards: vec![],
                owner_id: Uuid::new_v4(),
            },
            mana: 0,
            position: Position {
                x: 0,
                y: 0,
                z: 0,
                level: 0,
            },
            max_health: 30,
            active_effects: vec![],
            cards_played_this_turn: 0,
            health_boosts: vec![],
            power_boosts: vec![],
            mana_spent_this_turn: 0,
        };
        let mut game_state = GameState::new(player1, player2);
        let card = Card {
            id: Uuid::new_v4(),
            name: "Test Card".to_string(),
            cost: 1,
            power: 1,
            rarity: Rarity::Common,
            effects: vec![],
            card_type: CardType::Spell,
        };

        let player_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        game_state.players.insert(
            player_id,
            Player {
                id: player_id,
                name: "Test Player".to_string(),
                health: 30,
                hand: vec![card.clone()],
                deck: Deck {
                    cards: vec![],
                    owner_id: player_id,
                },
                mana: 0,
                position: Position {
                    x: 0,
                    y: 0,
                    z: 0,
                    level: 0,
                },
                max_health: 30,
                active_effects: vec![],
                cards_played_this_turn: 0,
                health_boosts: vec![],
                power_boosts: vec![],
                mana_spent_this_turn: 0,
            },
        );

        game_state.players.insert(
            target_id,
            Player {
                id: target_id,
                name: "Target Player".to_string(),
                health: 30,
                hand: vec![],
                deck: Deck {
                    cards: vec![],
                    owner_id: target_id,
                },
                mana: 0,
                position: Position {
                    x: 0,
                    y: 0,
                    z: 0,
                    level: 0,
                },
                max_health: 30,
                active_effects: vec![],
                cards_played_this_turn: 0,
                health_boosts: vec![],
                power_boosts: vec![],
                mana_spent_this_turn: 0,
            },
        );

        let damage_effect = DamageEffect {
            value: EffectValue {
                base: 5,
                scaling: None,
            },
            target: EffectTarget::Specific(target_id),
            penetrating: false,
        };

        let effect = Effect::Damage(damage_effect);

        effect.apply(&mut game_state, player_id).unwrap();

        assert_eq!(game_state.players[&target_id].health, 25);
    }
}
