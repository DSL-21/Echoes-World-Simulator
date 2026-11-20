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
- [x] Expanded world terrain and deterministic generation
- [x] Procedural rivers that follow land from mountains to coastal provinces
- [x] Procedural rivers that follow land from mountains to coastal provinces
- [x] Resource stockpiles and terrain/building yield model
- [x] City founding and settlement population
- [x] Road infrastructure with movement-supply and defensive effects
- [x] Road infrastructure with movement-supply and defensive effects
- [x] Treaty matrix, trade agreements, war and truces
- [x] Army recruitment, movement, supply and province battles
- [x] Player army pathfinding toward selected provinces over legal land routes
- [x] Road- and river-aware army movement costs and crossing penalties
- [x] Technology catalogue, prerequisites and research progress
- [x] Periodic world events with persistent effects
- [x] Camera pan and zoom
- [x] Main menu with new campaign, resume, load and Escape pause/resume
- [x] Main menu with new campaign, resume, load and Escape pause/resume
- [x] Strategic map overlays for terrain, political ownership, population, resource potential and infrastructure
- [x] Versioned JSON save/load
- [x] Rival AI city founding and personality-based province development
- [x] AI relationship drift, trade, alliances, truces and army pathfinding
- [x] Government types and civic policies with economic/stability effects
- [x] Legitimacy, unrest pressure and province revolt resolution
- [x] Player national reform action with resource cost and cooldown
- [x] Adjustable tax rate with bounded values and unrest consequences
- [x] Four victory paths, capital-loss defeat condition and terminal campaign state
- [x] Global commodity market with supply/demand price movement and trade-income effects
- [x] International land trade routes with treaty access, cargo exports, partner payment and road-based capacity
- [x] Player buy/sell orders for food, timber, stone and iron
- [ ] Verify visual playability on a Windows desktop
- [x] Add automated multi-year balance and AI simulation tests

## 0.3 — Grand strategy systems
- [ ] Multi-step production chains, player-sized market orders and price-sensitive trade contracts
- [x] International overland trade routes
- [ ] Ports, maritime trade routes and naval transport
- [ ] Diplomacy AI: proposals, alliances, guarantees and demands
- [ ] War goals, peace terms, occupations and sieges
- [ ] Unit recruitment by province, commanders and army composition
- [ ] Province autonomy and governors (unrest/revolt foundations are implemented)
- [ ] Faction objectives, personalities and difficulty settings

## 0.4 — Civilization and world history
- [ ] Culture groups, languages and religious/civic institutions
- [ ] Migration, assimilation and demographic change
- [ ] Dynamic historical event chains
- [ ] Deeper research tree and era progression
- [ ] Institutions, laws, succession and political crises
- [ ] Victory conditions and sandbox campaign rules

## 0.5 — World presentation and release quality
- [ ] Province names, animated borders, trade-route and army-range overlays
- [ ] City and army models/icons with animated state changes
- [ ] Scenario setup, settings and tutorial (main campaign menu is implemented)
- [ ] Audio, ambience, music and event presentation
- [ ] Performance profiling and large-world optimization
- [ ] Release workflow, versioned Windows packages and Linux build

## Engineering rules

- Simulation rules stay independent from Bevy rendering.
- Randomized world generation and events are seed-driven and reproducible.
- Major player actions validate costs and preconditions before changing state.
- Save files carry an explicit format version and are validated on load.
- Each milestone must pass formatting, tests and a Windows x64 build.
