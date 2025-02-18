# Ascent

A strategic trading card game (TCG) where players battle while climbing a mystical mountain, combining deck building with positional tactics.

## Game Overview

Ascent is a unique TCG that combines traditional card mechanics with spatial positioning. Players must strategically navigate the mountain's levels while managing their deck and resources.

## Core Mechanics

### Mountain System
- Hexagonal grid-based movement
- Multiple levels of increasing difficulty
- Position-based card interactions and effects

### Card System
- Deck building with various card rarities
  - Common
  - Uncommon
  - Rare
  - Legendary

- Effect Types
  - Damage
  - Heal
  - Draw
  - Boost
  - Buff/Debuff

### Combat
- Position-based targeting
- Area of effect abilities
- Strategic movement options
- Resource management

## Technical Details

### Built With
- Rust
- Custom game engine

### Project Structure
```
src/
├── effects/     # Card effect system
├── errors/      # Error handling
├── game_state/  # Game state management
└── models/      # Core game models
```

## Development

### Prerequisites
```
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building
```
cargo build
```

### Testing
```
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
