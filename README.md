# ECHOES — Living World Simulator

**ECHOES** is a turn-based world-building strategy game written in Rust with the Bevy engine.

You are not controlling a single army. You are guiding a realm inside a living procedural world: claim land, develop its resources, grow your population, and watch rival civilizations expand around you.

> Current status: **early playable prototype (0.1.0-dev)**. Systems and presentation are actively being built; this is not yet a finished game.

## First playable milestone

- Procedurally generated terrain map with water, grassland, forest, desert, plains and mountains.
- Five rival realms, each with a capital, territory, resource stockpiles and an AI expansion turn.
- Select land with the mouse.
- Claim neutral land connected to your territory.
- Improve owned land with farms, logging camps, quarries and markets.
- End turns to produce resources, grow population and advance the world year.
- Random world events and a running chronicle.
- Keyboard controls and an in-game control panel.

## Controls

| Input | Action |
|---|---|
| Left mouse button | Select a tile |
| `Space` | End turn |
| `C` | Claim selected adjacent neutral land |
| `1` | Build a farm |
| `2` | Build a logging camp |
| `3` | Build a quarry |
| `4` | Build a market |

The in-game panel also provides clickable actions.

## Build and run

Requirements: a current stable Rust toolchain and the platform prerequisites listed in the [Bevy setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

```bash
git clone https://github.com/DSL-21/Echoes-World-Simulator.git
cd Echoes-World-Simulator
cargo run --release
```

Run the test suite:

```bash
cargo test
```

## Technology

- **Rust** — simulation, world generation, economy and rival decision-making.
- **Bevy 0.20.1** — rendering, input, windowing and UI.
- **GitHub Actions** — formatting, tests and Windows release builds.

## Project direction

ECHOES is planned as a systemic strategy sandbox. Future milestones can add deeper diplomacy, wars, trade routes, technologies, cultures, cities, historical events, save/load and larger worlds. The prototype focuses first on making the world readable and the turn loop genuinely playable.

See [CHANGELOG.md](CHANGELOG.md) and [ROADMAP.md](docs/ROADMAP.md) for development progress.

## License

This project is released under the MIT License. See [LICENSE](LICENSE).
