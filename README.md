# ECHOES — Living World Simulator

**ECHOES** is a turn-based grand-strategy and civilization simulation project built with Rust and Bevy.

The goal is a systemic world in which independent realms expand, develop economies, research technologies, negotiate, fight wars and leave a historical record. The current build is a playable foundation, not the final scope of the game.

## Current build — 0.2.0-dev

- Deterministic procedural world generation with eight terrain types and coastal transitions.
- Five rival realms with capitals, starting provinces, populations, economies and AI personalities.
- Province ownership, connected expansion and terrain-dependent construction.
- Economy with food, timber, stone, iron, gold, science and influence.
- Eight province building types with costs and production effects.
- City founding and population tracking.
- Diplomacy with trade agreements, alliances, war declarations and truces.
- Internal politics with six government forms, six civic policies, legitimacy, unrest and province revolts.
- A global commodity market where food, timber, stone and iron prices react to world supply and demand, with live buy/sell orders in the Trade Desk.
- Four victory paths: territorial hegemony, scientific ascendancy, economic dominion and cultural legacy.
- Defeat when the player loses the capital; completed campaigns become terminal and remain saved.
- Armies with recruitment, movement, supply, experience and province battles.
- Rival AI city founding, personality-based province development, diplomacy drift, trade and alliance decisions, and army pathfinding.
- Research tree with prerequisites and technology effects.
- Periodic world events and a persistent chronicle.
- Camera panning and zoom, province selection and contextual realm information.
- JSON campaign save/load using F5 and F9.
- Automated tests plus Windows x64 build and downloadable preview artifact.

## Controls

| Input | Action |
|---|---|
| Left mouse button | Select a province |
| WASD | Pan the camera |
| Plus / minus | Zoom in / out |
| F1 | Terrain map layer |
| F2 | Political ownership map layer |
| F3 | Population density map layer |
| F4 | Resource potential map layer |
| F6 | Infrastructure map layer |
| Space | Advance one year |
| C | Claim selected adjacent neutral province |
| 1 | Build a farm |
| 2 | Build a lumber mill |
| 3 | Build a quarry |
| 4 | Build a market |
| 5 | Build an iron mine |
| 6 | Build a library |
| 7 | Build barracks |
| 8 | Build an aqueduct |
| B | Found a city in the selected province |
| G / V / J | Recruit infantry / cavalry / siege armies |
| R | Start the next available technology |
| M | Move the nearest army to the selected adjacent province |
| T | Propose a trade agreement with the selected realm |
| X | Declare war on the selected realm |
| P | Offer a truce to the selected realm |
| L | Enact the next national civic policy (costs gold and influence; six-year cooldown) |
| [ / ] | Lower / raise the national tax rate by 2 percentage points |
| F5 | Save campaign to echoes_save.json |
| F9 | Load campaign from echoes_save.json |

The five map layers can be changed at any time with F1–F4 and F6; F5 remains reserved for saving. The in-game Realm Council also provides clickable actions. The World Market board shows live commodity prices and lets the player buy or sell ten-unit lots of food, timber, stone and iron at a market spread. The Realm panel shows government, active policy, stability, unrest and legitimacy. National reforms cost resources and have a six-year cooldown. The tax rate can be adjusted between 8% and 35%; higher rates raise revenue but increase unrest. Severe unrest and low legitimacy can cause a province revolt. Armies move one province at a time; select an adjacent destination before issuing the move order.

## Campaign goals

A realm can win in one of four ways:

- **Territorial hegemony:** control at least 40% of all land provinces.
- **Scientific ascendancy:** complete seven technologies.
- **Economic dominion:** hold at least four cities, reach 35 population and accumulate 600 gold.
- **Cultural legacy:** reach 120 prestige with at least 65 stability and 75 legitimacy.

Losing the Aurelian League's capital ends the campaign in defeat. The campaign result is shown in the top bar and is included in the save file.

## Build from source

Requirements: stable Rust and the platform prerequisites in the [Bevy setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

    git clone https://github.com/DSL-21/Echoes-World-Simulator.git
    cd Echoes-World-Simulator
    cargo run --release

Run all tests with cargo test.

## Architecture

- src/sim/world.rs — coordinates, terrain, provinces and deterministic map generation.
- src/sim/economy.rs — resource stockpiles, building costs and production.
- src/sim/market.rs — dynamic global commodity prices from supply and demand.
- src/sim/ai.rs — rival development, diplomacy and army pathfinding.
- src/sim/society.rs — government types, civic policies and social modifiers.
- src/sim/diplomacy.rs — treaty state and symmetric relations.
- src/sim/military.rs — armies, supply, unit classes and combat strength.
- src/sim/technology.rs — research catalogue and technology prerequisites.
- src/sim/events.rs — deterministic world-event selection.
- src/sim/mod.rs — campaign state, turn simulation, AI and player actions.
- src/persistence.rs — versioned JSON save/load.
- src/main.rs — Bevy application, rendering, controls and HUD.

## Development direction

The next layers are deeper economic chains, diplomacy and AI negotiation, province governors, more detailed warfare, maritime systems, culture and institutions, scenario setup, campaign settings, improved art/audio and release packaging. Each system is intended to remain testable independently of the renderer.

See docs/ROADMAP.md for the current plan and CHANGELOG.md for implementation history.

## License

MIT. See LICENSE.
