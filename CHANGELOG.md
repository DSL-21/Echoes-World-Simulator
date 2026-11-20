# Changelog

## 0.2.0-dev — Simulation foundation

- Replaced the single-file prototype model with separate simulation modules.
- Expanded the world to a larger deterministic map with eight terrain classes.
- Added seven resource types and terrain/building production.
- Added eight province buildings with construction restrictions and costs.
- Added cities, settlement population and city founding.
- Added symmetric diplomacy relations, trade agreements, war and truces.
- Added recruitable armies, movement, supply, experience and simple province battles.
- Added a technology tree with prerequisites, ongoing research and economic effects.
- Added deterministic periodic events with lasting campaign effects.
- Added camera pan/zoom and an expanded Realm Council.
- Added versioned JSON save/load through F5 and F9.
- Added rival AI city founding and personality-based province development.
- Added AI relationship drift, trade, alliances, timed truces and BFS army pathfinding.
- Added war exhaustion pressure and expanded unit/building controls.
- Added six government identities and six civic policies with economic and stability modifiers.
- Added national policy reforms, legitimacy, unrest, and revolt-driven province loss.
- Added global supply/demand commodity pricing for food, timber, stone and iron.
- Linked international trade-agreement income to the global market index and displayed current prices in the Realm Council.
- Added tests for the simulation domains, rival turns and campaign persistence.

## 0.1.0-dev — Initial playable prototype

- Created the Rust/Bevy game project.
- Added deterministic procedural terrain generation.
- Added five realms with capital settlements and starting provinces.
- Added resource production, population growth and yearly turns.
- Added province claiming and development improvements.
- Added rival expansion logic and world events.
- Added an in-game HUD, chronicle and keyboard actions.
- Added Windows CI and tag-based release workflows.
