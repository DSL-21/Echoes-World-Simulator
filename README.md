# ECHOES — Living World Simulator

**ECHOES** is a turn-based grand-strategy and civilization simulation project built with Rust and Bevy.

Independent realms expand, develop economies, research technologies, negotiate, fight wars and leave a historical record. The current build is a playable simulation foundation, not the final scope of the game.

## Current build — 0.2.0-dev

- Deterministic procedural world generation with eight terrain types, coastal transitions and winding rivers from highlands to the sea.
- Five rival realms with capitals, starting provinces, populations, economies and distinct AI personalities.
- Province ownership, connected expansion and terrain-dependent construction.
- Economy with food, timber, stone, iron, gold, science and influence.
- Nine province building types, including roads that improve army supply and defensive positions.
- City founding and population tracking.
- Diplomacy with trade agreements, alliances, war declarations and truces.
- Internal politics with six government forms, six civic policies, legitimacy, unrest and province revolts.
- A global commodity market where food, timber, stone and iron prices react to world supply and demand.
- International land trade routes that ship food, timber, stone or iron to treaty partners, with partner payments and road-based cargo capacity.
- Four victory paths: territorial hegemony, scientific ascendancy, economic dominion and cultural legacy.
- Defeat when the player loses the capital; completed campaigns become terminal and remain saved.
- Armies with recruitment, movement, supply, experience and province battles. Army pathfinding prefers roads and avoids unsafe borders; unbridged rivers increase supply costs and weaken attacks.
- Rival AI city founding, personality-based province development, diplomacy drift, trade and alliance decisions, and army pathfinding.
- Research tree with prerequisites and technology effects.
- Periodic world events and a persistent chronicle.
- Camera panning and zoom, province selection, live ownership borders, five strategic map layers and river overlays.
- Map presentation refreshes from the active campaign seed when a new world is started.
- Main menu with new campaign, resume, load and Escape-to-pause flow.
- JSON campaign save/load using F5 and F9.
- Event-driven visual refresh so idle keyboard polling does not invalidate the map and HUD every frame.
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
| 1–9 | Build the selected province improvement |
| B | Found a city in the selected province |
| G / V / J | Recruit infantry / cavalry / siege armies |
| M | Order the nearest army one legal land step toward the selected province |
| R | Start the next available technology |
| T | Propose a trade agreement with the selected realm |
| Y / H / N / U | Charter a food / timber / stone / iron trade route to the selected foreign city |
| X | Declare war on the selected realm |
| P | Offer a truce to the selected realm |
| L | Enact the next national civic policy |
| [ / ] | Lower / raise the national tax rate by 2 percentage points |
| F5 | Save campaign to `echoes_save.json` |
| F9 | Load campaign from `echoes_save.json` |
| Escape | Open or close the campaign menu / pause overlay |

The five map layers are changed with F1–F4 and F6; F5 remains reserved for saving. Roads reduce army supply loss on movement and strengthen a province's defensive position. Unbridged rivers increase marching supply costs and weaken attacks across crossings. Sign a trade agreement with a realm using T, select one of its cities, then use Y, H, N or U to charter a land convoy. Routes preserve domestic stock reserves, transfer cargo to the partner and pay the exporter at the dynamic market price. Roads increase route capacity. Routes are suspended while the treaty is broken or the safe land corridor is closed.

The Realm panel shows government, active policy, stability, unrest and legitimacy. National reforms cost resources and have a six-year cooldown. The tax rate can be adjusted between 8% and 35%; higher rates raise revenue but increase unrest. Severe unrest and low legitimacy can cause a province revolt. Armies move one province per order. Select a distant destination and use M repeatedly; the nearest reachable army follows a legal land route, cannot cross open ocean, and needs a war declaration to enter hostile territory.

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

- `src/sim/world.rs` — coordinates, terrain, provinces, deterministic map generation and river paths.
- `src/sim/economy.rs` — resource stockpiles, building costs and production.
- `src/sim/market.rs` — dynamic global commodity prices from supply and demand.
- `src/sim/trade_routes.rs` — legal land corridors, treaty access, cargo capacity and trade shipment rules.
- `src/sim/ai.rs` — rival development, diplomacy and road-/river-aware army pathfinding.
- `src/sim/society.rs` — government types, civic policies and social modifiers.
- `src/sim/diplomacy.rs` — treaty state and symmetric relations.
- `src/sim/military.rs` — armies, supply, unit classes and combat strength.
- `src/sim/technology.rs` — research catalogue and technology prerequisites.
- `src/sim/events.rs` — deterministic world-event selection.
- `src/sim/victory.rs` — campaign victory paths and defeat conditions.
- `src/sim/mod.rs` — campaign state, turn simulation, AI and player actions.
- `src/persistence.rs` — versioned JSON save/load.
- `src/main.rs` — Bevy application, rendering, controls and HUD.

## Development direction

The next layers are maritime trade and naval transport, deeper diplomacy and war goals, commanders and sieges, province governors, migration and cultural institutions, additional historical events, scenario setup, campaign settings, art/audio and release packaging. Each system is intended to remain testable independently of the renderer.

See `docs/ROADMAP.md` for the current plan and `CHANGELOG.md` for implementation history.

## License

MIT. See `LICENSE`.
