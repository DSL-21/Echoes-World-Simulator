# Changelog

## 0.2.0-dev — Living strategy foundation

- Replaced the single-file prototype with separate world, economy, diplomacy, military, society, market, AI, technology, events, trade-route and victory modules.
- Expanded the world to a larger deterministic map with eight terrain classes, coastal transitions and winding rivers.
- Added dynamic province borders, five strategic map layers and campaign-seed-aware river/border overlays.
- Added seven resources and terrain/building production, with nine province building types including roads.
- Added road and river effects to army pathfinding, supply costs, defensive positions and combat crossings.
- Added city founding, settlement population, rival city development and population growth.
- Added five rival realms with distinct AI personalities, expansion, diplomacy, trade and military decisions.
- Added six government identities, six civic policies, national reforms, tax adjustment, legitimacy, unrest and province revolts.
- Added diplomacy with symmetric relations, trade agreements, alliances, war declarations and truces.
- Added a dynamic global market for food, timber, stone and iron, including player buy/sell orders.
- Added international land trade routes with treaty checks, cargo exports, partner payments, reserve stock and road-enhanced capacity.
- Added infantry, cavalry and siege recruitment, army movement, supply, experience and province battles.
- Added research technologies with prerequisites, ongoing progress and economic effects.
- Added deterministic periodic events with persistent effects and a campaign chronicle.
- Added four victory paths, capital-loss defeat and terminal campaign state.
- Added camera pan/zoom, a main campaign menu, pause/resume and versioned JSON save/load.
- Added tests for the simulation domains, route transactions, road/river effects, AI turns and persistence.
- Added Windows x64 CI builds with downloadable preview packages.

## 0.1.0-dev — Initial playable prototype

- Created the Rust/Bevy game project.
- Added deterministic procedural terrain generation.
- Added five realms with capital settlements and starting provinces.
- Added resource production, population growth and yearly turns.
- Added province claiming and development improvements.
- Added rival expansion logic and world events.
- Added an in-game HUD, chronicle and keyboard actions.
- Added Windows CI and tag-based release workflows.
