# ECHOES development roadmap

The project is being built as a connected simulation rather than a collection of disconnected menus. Milestones describe systems that are implemented and tested, not merely planned.

## 0.1 — Living map prototype
- [x] Procedural terrain generation
- [x] Five starting realms and capitals
- [x] Province selection and ownership
- [x] Connected land claims
- [x] Resource production and population growth
- [x] Province improvements
- [x] Rival expansion AI
- [x] Turn progression, events and chronicle
- [x] Windows x64 CI build

## 0.2 — Simulation foundation
- [x] Separate simulation modules from Bevy presentation
- [x] Expanded deterministic world generation with eight terrain classes
- [x] Rivers following land from highlands to coastal provinces
- [x] Dynamic province borders that respond to claims and conquest
- [x] River and border overlays refresh for every newly seeded campaign
- [x] Strategic map layers: terrain, ownership, population, resource potential and infrastructure
- [x] Resource stockpiles and terrain/building yield model
- [x] Nine province building types, including roads and river-crossing effects
- [x] City founding, settlement population and AI city founding
- [x] Diplomacy: trade agreements, alliances, war and truces
- [x] AI relationship drift, trade, alliances, truces and army decisions
- [x] Armies with infantry, cavalry and siege classes, supply, experience and province battles
- [x] Army pathfinding that accounts for roads, rivers, terrain and hostile borders
- [x] Technology catalogue, prerequisites and research progress
- [x] Periodic world events with persistent effects
- [x] Government types and civic policies with economic/stability effects
- [x] Legitimacy, unrest pressure and province revolt resolution
- [x] National reform action with resource cost and cooldown
- [x] Adjustable tax rate with bounded values and unrest consequences
- [x] Four victory paths, capital-loss defeat condition and terminal campaign state
- [x] Global commodity market with supply/demand prices and player buy/sell orders
- [x] International land trade routes with treaty access, cargo exports and partner payments
- [x] Camera pan/zoom, main menu, resume, load and Escape-to-pause
- [x] Versioned JSON save/load
- [x] Event-driven map and HUD refresh on actual campaign changes
- [ ] Verify visual playability on a Windows desktop
- [ ] Add automated multi-year balance and AI simulation tests

## 0.3 — Grand strategy systems
- [ ] Multi-step production chains, player-sized market orders and price-sensitive trade contracts
- [ ] Ports, maritime trade routes and naval transport
- [ ] Deeper diplomacy: proposals, guarantees, demands and diplomatic leverage
- [ ] War goals, peace terms, occupations, sieges and capitulation
- [ ] Unit recruitment by province, commanders and army composition
- [ ] Province autonomy, governors and regional administration
- [ ] Faction objectives, personalities, difficulty settings and scenario rules

## 0.4 — Civilization and world history
- [ ] Culture groups, languages and civic institutions
- [ ] Migration, assimilation and demographic change
- [ ] Dynamic historical event chains
- [ ] Deeper research tree and era progression
- [ ] Institutions, laws, succession and political crises
- [ ] Additional sandbox and campaign victory conditions

## 0.5 — World presentation and release quality
- [ ] Province names, city labels and map tooltips
- [ ] Animated border transitions, trade-route lines and army-range overlays
- [ ] City and army models/icons with animated state changes
- [ ] Scenario setup, settings, tutorial and in-game encyclopedia
- [ ] Audio, ambience, music and event presentation
- [ ] Performance profiling and large-world optimization
- [ ] Versioned Windows packages and Linux/macOS builds

## Engineering rules

- Simulation rules stay independent from Bevy rendering.
- Randomized world generation and events are seed-driven and reproducible.
- Major player actions validate costs and preconditions before changing state.
- Save files carry an explicit format version and are validated on load.
- Idle input polling must not invalidate the entire world presentation.
- Each milestone must pass formatting, tests and a Windows x64 build.
