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
- [x] Resource stockpiles and terrain/building yield model
- [x] City founding and settlement population
- [x] Treaty matrix, trade agreements, war and truces
- [x] Army recruitment, movement, supply and province battles
- [x] Technology catalogue, prerequisites and research progress
- [x] Periodic world events with persistent effects
- [x] Camera pan and zoom
- [x] Versioned JSON save/load
- [x] Rival AI city founding and personality-based province development
- [x] AI relationship drift, trade, alliances, truces and army pathfinding
- [x] Government types and civic policies with economic/stability effects
- [x] Legitimacy, unrest pressure and province revolt resolution
- [x] Player national reform action with resource cost and cooldown
- [x] Adjustable tax rate with bounded values and unrest consequences
- [x] Global commodity market with supply/demand price movement and trade-income effects
- [ ] Verify visual playability on a Windows desktop
- [ ] Add automated multi-year balance and AI simulation tests

## 0.3 — Grand strategy systems
- [ ] Multi-step production chains, resource-specific market selling and price-sensitive trade contracts
- [ ] Trade routes, ports and maritime transport
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
- [ ] World map layers, labels, borders and province overlays
- [ ] City and army models/icons with animated state changes
- [ ] Main menu, scenario setup, settings and tutorial
- [ ] Audio, ambience, music and event presentation
- [ ] Performance profiling and large-world optimization
- [ ] Release workflow, versioned Windows packages and Linux build

## Engineering rules

- Simulation rules stay independent from Bevy rendering.
- Randomized world generation and events are seed-driven and reproducible.
- Major player actions validate costs and preconditions before changing state.
- Save files carry an explicit format version and are validated on load.
- Each milestone must pass formatting, tests and a Windows x64 build.
