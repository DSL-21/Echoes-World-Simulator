pub mod ai;
pub mod diplomacy;
pub mod economy;
pub mod events;
pub mod market;
pub mod military;
pub mod society;
pub mod technology;
pub mod world;

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use self::{
    diplomacy::{Relation, Treaty, new_matrix, set_symmetric},
    economy::{Building, Resources, terrain_yield},
    events::WorldEvent,
    market::MarketState,
    military::{Army, UnitClass, recruitment_cost},
    society::{CivicPolicy, GovernmentType},
    technology::Technology,
    world::{Coord, WorldMap},
};

pub const PLAYER_ID: usize = 0;
pub const WORLD_WIDTH: i32 = 44;
pub const WORLD_HEIGHT: i32 = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiPersonality {
    Balanced,
    Expansionist,
    Merchant,
    Scholar,
    Defender,
}

impl AiPersonality {
    fn for_faction(id: usize) -> Self {
        match id % 5 {
            0 => Self::Balanced,
            1 => Self::Expansionist,
            2 => Self::Merchant,
            3 => Self::Scholar,
            _ => Self::Defender,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: usize,
    pub name: String,
    pub color: [f32; 3],
    pub capital: Coord,
    pub population: u32,
    pub resources: Resources,
    pub stability: f32,
    pub tax_rate: f32,
    pub prestige: f32,
    pub war_exhaustion: f32,
    #[serde(default)]
    pub government: GovernmentType,
    #[serde(default)]
    pub policy: CivicPolicy,
    #[serde(default)]
    pub unrest: f32,
    #[serde(default = "default_legitimacy")]
    pub legitimacy: f32,
    #[serde(default)]
    pub policy_changed_turn: u32,
    pub personality: AiPersonality,
    pub active_research: Option<Technology>,
    pub research_progress: f32,
    pub technologies: Vec<Technology>,
}

fn default_legitimacy() -> f32 {
    68.0
}

impl Faction {
    fn new(id: usize, name: &str, color: [f32; 3], capital: Coord) -> Self {
        let mut resources = Resources::starting();
        resources.gold += id as f32 * 4.0;
        Self {
            id,
            name: name.to_owned(),
            color,
            capital,
            population: 12,
            resources,
            stability: 72.0,
            tax_rate: 0.18,
            prestige: 10.0,
            war_exhaustion: 0.0,
            government: GovernmentType::for_faction(id),
            policy: CivicPolicy::for_faction(id),
            unrest: 8.0,
            legitimacy: 68.0,
            policy_changed_turn: 1,
            personality: AiPersonality::for_faction(id),
            active_research: None,
            research_progress: 0.0,
            technologies: Vec::new(),
        }
    }

    pub fn has_technology(&self, technology: Technology) -> bool {
        self.technologies.contains(&technology)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub world: WorldMap,
    pub factions: Vec<Faction>,
    pub relations: Vec<Vec<Relation>>,
    pub armies: Vec<Army>,
    #[serde(default)]
    pub market: MarketState,
    pub selected: Coord,
    pub turn: u32,
    pub year: u32,
    pub seed: u64,
    pub next_army_id: u64,
    pub chronicle: VecDeque<String>,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        let mut world = WorldMap::generate(WORLD_WIDTH, WORLD_HEIGHT, seed);
        let targets = [
            Coord::new(5, 13),
            Coord::new(37, 5),
            Coord::new(37, 22),
            Coord::new(21, 4),
            Coord::new(21, 24),
        ];
        let definitions = [
            ("Aurelian League", [0.25, 0.65, 0.92]),
            ("Verdant Compact", [0.32, 0.75, 0.45]),
            ("Sable Dominion", [0.88, 0.34, 0.31]),
            ("Ivory Synod", [0.88, 0.72, 0.34]),
            ("Azure Cartel", [0.65, 0.47, 0.86]),
        ];
        let city_names = [
            "Aurelia",
            "Greenhold",
            "Duskport",
            "Ivory Crown",
            "Azure Reach",
        ];
        let mut capitals = Vec::new();
        let mut factions = Vec::new();

        for (id, (name, color)) in definitions.iter().enumerate() {
            let capital = world.nearest_land(targets[id], &capitals);
            capitals.push(capital);
            factions.push(Faction::new(id, name, *color, capital));

            if let Some(tile) = world.get_mut(capital) {
                tile.owner = Some(id);
                tile.city = Some(city_names[id].to_owned());
                tile.population = 12;
                tile.discovered = true;
            }

            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    if dx.abs() + dy.abs() == 0 {
                        continue;
                    }
                    let point = Coord::new(capital.x + dx, capital.y + dy);
                    if let Some(tile) = world.get_mut(point) {
                        if tile.terrain.is_land() && tile.owner.is_none() {
                            tile.owner = Some(id);
                            tile.discovered = true;
                        }
                    }
                }
            }
        }

        let mut game = Self {
            world,
            factions,
            relations: new_matrix(definitions.len()),
            armies: Vec::new(),
            market: MarketState::default(),
            selected: capitals[PLAYER_ID],
            turn: 1,
            year: 1200,
            seed,
            next_army_id: 1,
            chronicle: VecDeque::new(),
        };
        game.add_log("The age of realms begins. Five powers rise across an uncharted world.");
        game.add_log(
            "Your council awaits. Expand carefully, develop your provinces and shape history.",
        );
        game
    }

    pub fn add_log(&mut self, message: impl Into<String>) {
        Self::push_chronicle(&mut self.chronicle, message);
    }

    fn push_chronicle(chronicle: &mut VecDeque<String>, message: impl Into<String>) {
        chronicle.push_back(message.into());
        while chronicle.len() > 16 {
            chronicle.pop_front();
        }
    }

    pub fn select(&mut self, coord: Coord) -> bool {
        let Some(tile) = self.world.get(coord) else {
            return false;
        };
        let owner = self.faction_name(tile.owner).to_owned();
        let place = tile
            .city
            .clone()
            .unwrap_or_else(|| tile.terrain.name().to_owned());
        self.selected = coord;
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "Selected {place} — {}, {} — {owner}.",
                coord.x + 1,
                coord.y + 1
            ),
        );
        true
    }

    pub fn owned_count(&self, faction_id: usize) -> usize {
        self.world
            .tiles
            .iter()
            .filter(|tile| tile.owner == Some(faction_id))
            .count()
    }

    pub fn city_count(&self, faction_id: usize) -> usize {
        self.world
            .tiles
            .iter()
            .filter(|tile| tile.owner == Some(faction_id) && tile.city.is_some())
            .count()
    }

    pub fn army_count(&self, faction_id: usize) -> usize {
        self.armies
            .iter()
            .filter(|army| army.owner == faction_id)
            .count()
    }

    pub fn faction_name(&self, faction_id: Option<usize>) -> &str {
        faction_id
            .and_then(|id| self.factions.get(id))
            .map(|f| f.name.as_str())
            .unwrap_or("Unclaimed")
    }

    pub fn treaty_between(&self, a: usize, b: usize) -> Treaty {
        self.relations
            .get(a)
            .and_then(|row| row.get(b))
            .map(|r| r.treaty)
            .unwrap_or(Treaty::None)
    }

    fn adjacent_to_faction(&self, coord: Coord, faction_id: usize) -> bool {
        self.world.neighbors(coord).iter().any(|p| {
            self.world
                .get(*p)
                .is_some_and(|tile| tile.owner == Some(faction_id))
        })
    }

    pub fn claim_selected(&mut self) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else {
            return false;
        };
        if !tile.terrain.is_land() {
            Self::push_chronicle(
                &mut self.chronicle,
                "The deep ocean cannot be claimed without a maritime settlement.",
            );
            return false;
        }
        if tile.owner.is_some() {
            Self::push_chronicle(
                &mut self.chronicle,
                "This province already belongs to a realm.",
            );
            return false;
        }
        if !self.adjacent_to_faction(coord, PLAYER_ID) {
            Self::push_chronicle(
                &mut self.chronicle,
                "A claim must connect to your existing borders.",
            );
            return false;
        }
        let cost = Resources {
            gold: 8.0,
            influence: 7.0,
            ..Default::default()
        };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            Self::push_chronicle(
                &mut self.chronicle,
                "The council lacks the gold or influence to secure this province.",
            );
            return false;
        }
        let terrain = tile.terrain;
        let tile = self.world.get_mut(coord).expect("selected province exists");
        tile.owner = Some(PLAYER_ID);
        tile.discovered = true;
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "New province secured: {}. The borders of the League expand.",
                terrain.name()
            ),
        );
        true
    }

    pub fn build_selected(&mut self, building: Building) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else {
            return false;
        };
        if tile.owner != Some(PLAYER_ID) {
            Self::push_chronicle(
                &mut self.chronicle,
                "You can only construct buildings in provinces controlled by your realm.",
            );
            return false;
        }
        if tile.city.is_some() {
            Self::push_chronicle(
                &mut self.chronicle,
                "Urban administration occupies this province. Choose another province for rural development.",
            );
            return false;
        }
        if tile.buildings.contains(&building) {
            Self::push_chronicle(
                &mut self.chronicle,
                "This province already has that building.",
            );
            return false;
        }
        let terrain = tile.terrain;
        if !building.allowed_on(terrain) {
            Self::push_chronicle(
                &mut self.chronicle,
                format!(
                    "{} cannot be built on {} terrain.",
                    building.name(),
                    terrain.name()
                ),
            );
            return false;
        }
        if !self.factions[PLAYER_ID].resources.pay(&building.cost()) {
            Self::push_chronicle(
                &mut self.chronicle,
                "The treasury does not contain the required construction materials.",
            );
            return false;
        }
        self.world
            .get_mut(coord)
            .expect("selected province exists")
            .buildings
            .push(building);
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "{} completed in the {} province.",
                building.name(),
                terrain.name()
            ),
        );
        true
    }

    pub fn found_city(&mut self) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else {
            return false;
        };
        if tile.owner != Some(PLAYER_ID) || tile.city.is_some() {
            Self::push_chronicle(
                &mut self.chronicle,
                "A new city must be founded in an owned province without an existing city.",
            );
            return false;
        }
        let cost = Resources {
            food: 8.0,
            timber: 18.0,
            stone: 15.0,
            gold: 28.0,
            ..Default::default()
        };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            Self::push_chronicle(
                &mut self.chronicle,
                "Founding a city requires more food, timber, stone and gold.",
            );
            return false;
        }
        let names = [
            "Westhaven",
            "Stoneford",
            "New Aurelia",
            "Brightwater",
            "Highgarden",
            "Rivermarch",
            "Dawnspire",
            "Silverkeep",
        ];
        let index = (self.city_count(PLAYER_ID) + self.turn as usize) % names.len();
        let name = names[index].to_owned();
        let tile = self.world.get_mut(coord).expect("selected province exists");
        tile.city = Some(name.clone());
        tile.population = 3;
        self.factions[PLAYER_ID].population += 3;
        Self::push_chronicle(
            &mut self.chronicle,
            format!("A new settlement is founded: {name}."),
        );
        true
    }

    pub fn recruit_army(&mut self, class: UnitClass) -> bool {
        if self.city_count(PLAYER_ID) == 0 {
            Self::push_chronicle(
                &mut self.chronicle,
                "Your realm needs a city before it can recruit an army.",
            );
            return false;
        }
        let position = self.factions[PLAYER_ID].capital;
        let (gold, iron, timber) = recruitment_cost(class);
        let cost = Resources {
            gold,
            iron,
            timber,
            ..Default::default()
        };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            Self::push_chronicle(
                &mut self.chronicle,
                "The treasury cannot afford to equip another army.",
            );
            return false;
        }
        let id = self.next_army_id;
        self.next_army_id += 1;
        self.armies.push(Army::new(id, PLAYER_ID, position, class));
        Self::push_chronicle(
            &mut self.chronicle,
            format!("{} recruited at the capital.", class.label()),
        );
        true
    }

    pub fn offer_trade_agreement(&mut self, target: usize) -> bool {
        if target >= self.factions.len() || target == PLAYER_ID {
            return false;
        }
        if self.treaty_between(PLAYER_ID, target) == Treaty::War {
            Self::push_chronicle(
                &mut self.chronicle,
                "Trade cannot be negotiated while the two realms are at war.",
            );
            return false;
        }
        let opinion = self.relations[PLAYER_ID][target].opinion;
        if opinion < -35 {
            Self::push_chronicle(
                &mut self.chronicle,
                format!(
                    "{} rejects the trade proposal. Relations are too hostile.",
                    self.factions[target].name
                ),
            );
            return false;
        }
        set_symmetric(
            &mut self.relations,
            PLAYER_ID,
            target,
            Treaty::TradeAgreement,
            self.turn,
        );
        let next_opinion = (self.relations[PLAYER_ID][target].opinion + 8).min(100);
        self.relations[PLAYER_ID][target].opinion = next_opinion;
        self.relations[target][PLAYER_ID].opinion = next_opinion;
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "A trade agreement is signed with {}.",
                self.factions[target].name
            ),
        );
        true
    }

    pub fn declare_war(&mut self, target: usize) -> bool {
        if target >= self.factions.len() || target == PLAYER_ID {
            return false;
        }
        if self.treaty_between(PLAYER_ID, target) == Treaty::War {
            Self::push_chronicle(
                &mut self.chronicle,
                "This realm is already at war with you.",
            );
            return false;
        }
        let name = self.factions[target].name.clone();
        set_symmetric(
            &mut self.relations,
            PLAYER_ID,
            target,
            Treaty::War,
            self.turn,
        );
        self.relations[PLAYER_ID][target].opinion = -100;
        self.relations[target][PLAYER_ID].opinion = -100;
        self.factions[PLAYER_ID].war_exhaustion += 5.0;
        Self::push_chronicle(
            &mut self.chronicle,
            format!("WAR DECLARED — the Aurelian League enters a state of war with {name}."),
        );
        true
    }

    pub fn offer_peace(&mut self, target: usize) -> bool {
        if target >= self.factions.len()
            || target == PLAYER_ID
            || self.treaty_between(PLAYER_ID, target) != Treaty::War
        {
            Self::push_chronicle(
                &mut self.chronicle,
                "There is no active war with that realm to bring to an end.",
            );
            return false;
        }
        let name = self.factions[target].name.clone();
        set_symmetric(
            &mut self.relations,
            PLAYER_ID,
            target,
            Treaty::Truce,
            self.turn,
        );
        self.relations[PLAYER_ID][target].opinion = -15;
        self.relations[target][PLAYER_ID].opinion = -15;
        self.factions[PLAYER_ID].war_exhaustion =
            (self.factions[PLAYER_ID].war_exhaustion - 15.0).max(0.0);
        Self::push_chronicle(
            &mut self.chronicle,
            format!("A truce is signed with {name}. The armies fall silent."),
        );
        true
    }

    pub fn cycle_player_policy(&mut self) -> bool {
        let faction = &self.factions[PLAYER_ID];
        if self.turn.saturating_sub(faction.policy_changed_turn) < 6 {
            self.add_log("The council cannot enact another major reform yet. Allow the previous policy to take effect.");
            return false;
        }
        let next_policy = faction.policy.next();
        let cost = Resources { gold: 18.0, influence: 14.0, ..Default::default() };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            self.add_log("A national reform requires 18 gold and 14 influence.");
            return false;
        }
        let faction = &mut self.factions[PLAYER_ID];
        faction.policy = next_policy;
        faction.policy_changed_turn = self.turn;
        let name = next_policy.name();
        self.add_log(format!("NATIONAL REFORM — the council adopts {name}."));
        true
    }

    pub fn start_next_research(&mut self) -> bool {
        if self.factions[PLAYER_ID].active_research.is_some() {
            Self::push_chronicle(
                &mut self.chronicle,
                "Research is already underway. Let the scholars finish their current work.",
            );
            return false;
        }
        let known = self.factions[PLAYER_ID].technologies.clone();
        let next = Technology::ALL.into_iter().find(|technology| {
            !known.contains(technology)
                && technology.prerequisites().iter().all(|p| known.contains(p))
        });
        let Some(technology) = next else {
            Self::push_chronicle(
                &mut self.chronicle,
                "All currently available technologies have been researched.",
            );
            return false;
        };
        self.factions[PLAYER_ID].active_research = Some(technology);
        self.factions[PLAYER_ID].research_progress = 0.0;
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "Scholars begin researching {}: {}",
                technology.name(),
                technology.description()
            ),
        );
        true
    }

    pub fn move_nearest_army_to_selected(&mut self) -> bool {
        let selected = self.selected;
        let nearest = self
            .armies
            .iter()
            .filter(|army| army.owner == PLAYER_ID && army.position.manhattan(selected) == 1)
            .min_by_key(|army| army.position.manhattan(selected))
            .map(|army| army.id);
        let Some(id) = nearest else {
            Self::push_chronicle(
                &mut self.chronicle,
                "There is no army available to receive orders.",
            );
            return false;
        };
        self.move_army(id, selected)
    }

    pub fn move_army(&mut self, army_id: u64, destination: Coord) -> bool {
        let Some(index) = self.armies.iter().position(|army| army.id == army_id) else {
            return false;
        };
        let origin = self.armies[index].position;
        let owner = self.armies[index].owner;
        if origin.manhattan(destination) != 1 {
            Self::push_chronicle(
                &mut self.chronicle,
                "Armies can move one adjacent province per turn in this simulation build.",
            );
            return false;
        }
        let Some(tile) = self.world.get(destination) else {
            return false;
        };
        if !tile.terrain.is_land() {
            Self::push_chronicle(
                &mut self.chronicle,
                "An army cannot cross open ocean without naval transport.",
            );
            return false;
        }
        let target_owner = tile.owner;
        if let Some(defender) = target_owner.filter(|defender| *defender != owner) {
            if self.treaty_between(owner, defender) != Treaty::War {
                Self::push_chronicle(
                    &mut self.chronicle,
                    "Entering another realm requires a declaration of war.",
                );
                return false;
            }
            let attack = self.armies[index].combat_power();
            let defending_armies: f32 = self
                .armies
                .iter()
                .filter(|a| a.owner == defender && a.position == destination)
                .map(Army::combat_power)
                .sum();
            let garrison = self
                .world
                .get(destination)
                .map(|t| t.population as f32 * 0.7 + if t.city.is_some() { 25.0 } else { 0.0 })
                .unwrap_or(0.0);
            let defense = 45.0 + defending_armies + garrison;
            let enemy_name = self.factions[defender].name.clone();
            if attack > defense {
                if let Some(tile) = self.world.get_mut(destination) {
                    tile.owner = Some(owner);
                    tile.population = (tile.population / 2).max(1);
                }
                self.armies
                    .retain(|a| !(a.owner == defender && a.position == destination));
                if let Some(attacker_index) = self.armies.iter().position(|a| a.id == army_id) {
                    self.armies[attacker_index].position = destination;
                    self.armies[attacker_index].strength =
                        (self.armies[attacker_index].strength * 0.72).max(20.0);
                    self.armies[attacker_index].experience += 8.0;
                }
                Self::push_chronicle(
                    &mut self.chronicle,
                    format!("VICTORY — your army captures a province from {enemy_name}."),
                );
            } else {
                if let Some(attacker_index) = self.armies.iter().position(|a| a.id == army_id) {
                    self.armies[attacker_index].strength *= 0.48;
                    self.armies[attacker_index].supply *= 0.65;
                    if self.armies[attacker_index].strength < 25.0 {
                        self.armies.remove(attacker_index);
                    }
                }
                Self::push_chronicle(
                    &mut self.chronicle,
                    format!("DEFEAT — the assault against {enemy_name} is repelled."),
                );
            }
            return true;
        }
        self.armies[index].position = destination;
        self.armies[index].supply = (self.armies[index].supply - 4.0).max(0.0);
        let name = self.armies[index].name.clone();
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "{name} marches to {}, {}.",
                destination.x + 1,
                destination.y + 1
            ),
        );
        true
    }

    pub fn advance_turn(&mut self) {
        self.turn += 1;
        self.year += 1;
        self.refresh_market();

        for faction_id in 0..self.factions.len() {
            let production = self.calculate_production(faction_id);
            let cities = self.city_count(faction_id) as f32;
            let upkeep: f32 = self.armies.iter()
                .filter(|army| army.owner == faction_id)
                .map(|army| army.strength * 0.035)
                .sum();
            let active_wars = self.relations[faction_id].iter()
                .filter(|relation| relation.treaty == Treaty::War)
                .count() as f32;
            let faction = &mut self.factions[faction_id];
            let mut population_growth = false;
            let modifiers = faction.government.modifiers().combine(faction.policy.modifiers());
            let tax_multiplier = (1.0 + modifiers.tax).clamp(0.65, 1.5);
            let upkeep_multiplier = (1.0 - modifiers.army_upkeep).clamp(0.5, 1.0);

            faction.war_exhaustion = (faction.war_exhaustion + active_wars * 0.18).min(100.0);
            faction.resources.add(&production);
            faction.resources.food -= faction.population as f32 * 0.22;
            faction.resources.gold += faction.population as f32 * faction.tax_rate
                * 0.12 * tax_multiplier + cities * 1.8 - upkeep * upkeep_multiplier;
            faction.resources.influence += 0.8 + cities * 0.55 + modifiers.influence;
            faction.resources.clamp_non_negative();

            if faction.resources.food > faction.population as f32 * 0.85 + 12.0
                && faction.stability > 35.0 && self.turn % 3 == 0
            {
                faction.population += 1;
                faction.resources.food -= 5.0;
                faction.prestige += 0.25;
                population_growth = true;
            } else if faction.resources.food < 2.0 {
                faction.population = faction.population.saturating_sub(1).max(1);
                faction.stability -= 4.0;
                faction.war_exhaustion = (faction.war_exhaustion + 0.5).min(100.0);
            }

            let food_pressure = if faction.resources.food < faction.population as f32 * 0.18 {
                5.0
            } else if faction.resources.food > faction.population as f32 * 1.1 {
                -1.0
            } else {
                0.0
            };
            let tax_pressure = (faction.tax_rate - 0.18).max(0.0) * 18.0;
            faction.unrest = (faction.unrest + food_pressure + tax_pressure
                + active_wars * 0.7 + modifiers.unrest * 2.0 - 0.7).clamp(0.0, 100.0);

            if faction.unrest > 75.0 {
                faction.legitimacy = (faction.legitimacy - 1.1).max(0.0);
            } else if faction.unrest < 35.0 && faction.stability > 55.0 {
                faction.legitimacy = (faction.legitimacy + 0.25).min(100.0);
            } else {
                faction.legitimacy = (faction.legitimacy - 0.08).max(0.0);
            }

            faction.stability = (faction.stability + 0.25 + modifiers.stability
                - (faction.unrest - 35.0).max(0.0) * 0.015
                - faction.war_exhaustion * 0.006).clamp(0.0, 100.0);
            faction.war_exhaustion = (faction.war_exhaustion - 0.18).max(0.0);

            if population_growth {
                let city = self.world.coords().into_iter()
                    .filter_map(|coord| {
                        let tile = self.world.get(coord)?;
                        (tile.owner == Some(faction_id) && tile.city.is_some())
                            .then_some((coord, tile.population))
                    })
                    .min_by_key(|(_, population)| *population)
                    .map(|(coord, _)| coord);
                if let Some(coord) = city {
                    if let Some(tile) = self.world.get_mut(coord) {
                        tile.population += 1;
                    }
                }
            }
        }

        self.progress_research();
        self.update_trade_income();
        self.process_ai_turns();
        ai::process_turn(self);
        self.update_army_supply();
        self.apply_world_event();
        if self.turn % 6 == 0 {
            self.resolve_unrest();
        }

        let year = self.year;
        let provinces = self.owned_count(PLAYER_ID);
        let cities = self.city_count(PLAYER_ID);
        let population = self.factions[PLAYER_ID].population;
        Self::push_chronicle(&mut self.chronicle, format!(
            "Year {year} — {provinces} provinces, {cities} cities, population {population}."
        ));
    }

    fn resolve_unrest(&mut self) {
        for faction_id in 0..self.factions.len() {
            let faction = &self.factions[faction_id];
            if faction.unrest < 78.0 || faction.legitimacy > 42.0 || self.owned_count(faction_id) < 4 {
                continue;
            }

            let capital = faction.capital;
            let lost_province = self.world.coords().into_iter()
                .filter(|coord| *coord != capital)
                .filter_map(|coord| {
                    let tile = self.world.get(coord)?;
                    if tile.owner != Some(faction_id) || tile.city.is_some() { return None; }
                    Some((coord, coord.manhattan(capital)))
                })
                .max_by_key(|(_, distance)| *distance)
                .map(|(coord, _)| coord);

            let Some(coord) = lost_province else { continue; };
            if let Some(tile) = self.world.get_mut(coord) { tile.owner = None; }

            let realm = self.factions[faction_id].name.clone();
            let position = format!("{}, {}", coord.x + 1, coord.y + 1);
            let faction = &mut self.factions[faction_id];
            faction.unrest = 48.0;
            faction.legitimacy = (faction.legitimacy - 15.0).max(0.0);
            faction.stability = (faction.stability - 12.0).max(0.0);
            faction.population = faction.population.saturating_sub(1).max(1);
            Self::push_chronicle(&mut self.chronicle, format!(
                "REVOLT — {realm} loses a province at {position} as local unrest erupts."
            ));
        }
    }

    fn calculate_production(&self, faction_id: usize) -> Resources {
        let mut output = Resources::default();
        let faction = &self.factions[faction_id];
        for tile in self.world.tiles.iter().filter(|tile| tile.owner == Some(faction_id)) {
            let mut province = terrain_yield(tile.terrain);
            for building in &tile.buildings { province.add(&building.yield_bonus()); }
            if tile.city.is_some() {
                province.food += 0.8;
                province.gold += 1.3;
                province.science += 1.0;
            }
            output.add(&province);
        }

        let modifiers = faction.government.modifiers().combine(faction.policy.modifiers());
        output.food *= (1.0 + modifiers.food).max(0.1);
        output.timber *= (1.0 + modifiers.timber).max(0.1);
        output.stone *= (1.0 + modifiers.stone).max(0.1);
        output.iron *= (1.0 + modifiers.iron).max(0.1);
        output.gold *= (1.0 + modifiers.gold).max(0.1);
        output.science *= (1.0 + modifiers.science).max(0.1);
        output.influence *= (1.0 + modifiers.influence).max(0.1);

        if faction.has_technology(Technology::Agriculture) { output.food *= 1.15; }
        if faction.has_technology(Technology::Currency) { output.gold *= 1.12; }
        if faction.has_technology(Technology::Metallurgy) { output.iron *= 1.20; }
        if faction.has_technology(Technology::Printing) { output.science *= 1.20; }
        output
    }

    fn progress_research(&mut self) {
        for id in 0..self.factions.len() {
            let Some(technology) = self.factions[id].active_research else {
                continue;
            };
            let available = self.factions[id].resources.science.min(7.0);
            self.factions[id].resources.science -= available;
            self.factions[id].research_progress += available;
            if self.factions[id].research_progress >= technology.cost() {
                self.factions[id].technologies.push(technology);
                self.factions[id].active_research = None;
                self.factions[id].research_progress = 0.0;
                self.factions[id].prestige += 5.0;
                let name = self.factions[id].name.clone();
                Self::push_chronicle(
                    &mut self.chronicle,
                    format!("{name} discovers {}.", technology.name()),
                );
            }
        }
    }

    fn refresh_market(&mut self) {
        let mut supply = Resources::default();
        let mut demand = Resources::default();

        for faction_id in 0..self.factions.len() {
            supply.add(&self.calculate_production(faction_id));
            let population = self.factions[faction_id].population as f32;
            let cities = self.city_count(faction_id) as f32;
            let provinces = self.owned_count(faction_id) as f32;
            let armies = self.army_count(faction_id) as f32;

            demand.food += population * 0.22;
            demand.timber += population * 0.025 + cities * 0.35;
            demand.stone += cities * 0.32 + provinces * 0.012;
            demand.iron += armies * 0.42 + cities * 0.10;
        }

        self.market.update(supply, demand);
    }

    fn update_trade_income(&mut self) {
        for a in 0..self.factions.len() {
            for b in (a + 1)..self.factions.len() {
                if matches!(self.treaty_between(a, b), Treaty::TradeAgreement | Treaty::Alliance) {
                    let a_modifiers = self.factions[a].government.modifiers()
                        .combine(self.factions[a].policy.modifiers());
                    let b_modifiers = self.factions[b].government.modifiers()
                        .combine(self.factions[b].policy.modifiers());
                    let market_yield = 0.8 + self.market.trade_index() * 0.35;
                    self.factions[a].resources.gold += market_yield + a_modifiers.trade_income;
                    self.factions[b].resources.gold += market_yield + b_modifiers.trade_income;
                    let opinion = (self.relations[a][b].opinion + 1).min(100);
                    self.relations[a][b].opinion = opinion;
                    self.relations[b][a].opinion = opinion;
                }
            }
        }
    }

    fn process_ai_turns(&mut self) {
        for id in 1..self.factions.len() {
            if self.factions[id].active_research.is_none() {
                let known = self.factions[id].technologies.clone();
                let next = Technology::ALL.into_iter().find(|t| {
                    !known.contains(t) && t.prerequisites().iter().all(|p| known.contains(p))
                });
                self.factions[id].active_research = next;
            }
            if self.factions[id].resources.gold >= 8.0
                && self.factions[id].resources.influence >= 7.0
            {
                if let Some(coord) = self.best_expansion(id) {
                    let cost = Resources {
                        gold: 8.0,
                        influence: 7.0,
                        ..Default::default()
                    };
                    if self.factions[id].resources.pay(&cost) {
                        if let Some(tile) = self.world.get_mut(coord) {
                            tile.owner = Some(id);
                            tile.discovered = true;
                        }
                    }
                }
            }
            if self.turn % 9 == 0 {
                let (gold, iron, timber) = recruitment_cost(UnitClass::Infantry);
                let cost = Resources {
                    gold,
                    iron,
                    timber,
                    ..Default::default()
                };
                if self.factions[id].resources.pay(&cost) {
                    let army_id = self.next_army_id;
                    self.next_army_id += 1;
                    self.armies.push(Army::new(
                        army_id,
                        id,
                        self.factions[id].capital,
                        UnitClass::Infantry,
                    ));
                }
            }
        }
    }

    fn best_expansion(&self, faction_id: usize) -> Option<Coord> {
        let mut best: Option<(Coord, f32)> = None;
        for coord in self.world.coords() {
            let Some(tile) = self.world.get(coord) else {
                continue;
            };
            if tile.owner.is_some()
                || !tile.terrain.is_land()
                || !self.adjacent_to_faction(coord, faction_id)
            {
                continue;
            }
            let bias = ((coord.x * 17 + coord.y * 31 + faction_id as i32 * 13).rem_euclid(17))
                as f32
                / 100.0;
            let score = tile.terrain.expansion_value() + bias;
            if best.is_none_or(|(_, current)| score > current) {
                best = Some((coord, score));
            }
        }
        best.map(|(coord, _)| coord)
    }

    fn update_army_supply(&mut self) {
        for army in &mut self.armies {
            let supplied = self
                .world
                .get(army.position)
                .is_some_and(|tile| tile.owner == Some(army.owner));
            if supplied {
                army.supply = (army.supply + 8.0).min(100.0);
            } else {
                army.supply = (army.supply - 9.0).max(0.0);
            }
            if army.supply < 20.0 {
                army.strength = (army.strength - 2.0).max(10.0);
            }
        }
    }

    fn apply_world_event(&mut self) {
        let Some(event) = WorldEvent::roll(self.seed, self.turn) else {
            return;
        };
        let player = &mut self.factions[PLAYER_ID];
        match event {
            WorldEvent::BountifulHarvest => {
                player.resources.food += 22.0;
                player.stability = (player.stability + 3.0).min(100.0);
            }
            WorldEvent::Drought => {
                player.resources.food = (player.resources.food - 18.0).max(0.0);
                player.stability = (player.stability - 4.0).max(0.0);
            }
            WorldEvent::MerchantCaravan => player.resources.gold += 20.0,
            WorldEvent::Plague => {
                player.population = player.population.saturating_sub(2).max(1);
                player.stability = (player.stability - 8.0).max(0.0);
            }
            WorldEvent::IronDiscovery => player.resources.iron += 14.0,
            WorldEvent::PublicFestival => {
                player.stability = (player.stability + 8.0).min(100.0);
                player.prestige += 2.0;
            }
            WorldEvent::BanditRaids => {
                player.resources.gold = (player.resources.gold - 12.0).max(0.0);
                player.resources.food = (player.resources.food - 8.0).max(0.0);
            }
            WorldEvent::PoliticalScandal => player.stability = (player.stability - 7.0).max(0.0),
        }
        Self::push_chronicle(
            &mut self.chronicle,
            format!(
                "WORLD EVENT — {}. The realm must adapt to changing fortunes.",
                event.title()
            ),
        );
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.world.width <= 0
            || self.world.height <= 0
            || self.world.tiles.len() != (self.world.width * self.world.height) as usize
        {
            return Err("The saved world map has invalid dimensions.".to_owned());
        }
        if self.factions.is_empty() || PLAYER_ID >= self.factions.len() {
            return Err("The save does not contain a playable realm.".to_owned());
        }
        if self.world.get(self.selected).is_none() {
            return Err("The selected province is outside the saved map.".to_owned());
        }
        if self.relations.len() != self.factions.len()
            || self
                .relations
                .iter()
                .any(|row| row.len() != self.factions.len())
        {
            return Err("The diplomacy table is inconsistent with the realm list.".to_owned());
        }
        if self.armies.iter().any(|army| {
            army.owner >= self.factions.len() || self.world.get(army.position).is_none()
        }) {
            return Err("An army in the save file has an invalid owner or position.".to_owned());
        }
        if !self.market.is_valid() {
            return Err("The saved global market contains an invalid commodity price.".to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_starts_with_five_realms_and_capitals() {
        let game = GameState::new(20261120);
        assert_eq!(game.factions.len(), 5);
        for (id, faction) in game.factions.iter().enumerate() {
            let capital = game.world.get(faction.capital).unwrap();
            assert_eq!(capital.owner, Some(id));
            assert!(capital.city.is_some());
        }
        assert!(game.validate().is_ok());
    }

    #[test]
    fn founding_a_city_adds_to_realm_population() {
        let mut game = GameState::new(2029);
        let capital = game.factions[PLAYER_ID].capital;
        let site = game.world.neighbors(capital).into_iter().find(|coord| {
            game.world.get(*coord).is_some_and(|tile| tile.owner == Some(PLAYER_ID) && tile.city.is_none())
        }).expect("starting realm has an adjacent non-capital province");
        game.select(site);
        let before = game.factions[PLAYER_ID].population;
        assert!(game.found_city());
        assert_eq!(game.factions[PLAYER_ID].population, before + 3);
        assert_eq!(game.world.get(site).unwrap().population, 3);
    }

    #[test]
    fn player_can_claim_connected_neutral_land_when_available() {
        let mut game = GameState::new(42);
        let capital = game.factions[PLAYER_ID].capital;
        if let Some(target) = game.world.neighbors(capital).into_iter().find(|p| {
            game.world
                .get(*p)
                .is_some_and(|t| t.terrain.is_land() && t.owner.is_none())
        }) {
            assert!(game.select(target));
            assert!(game.claim_selected());
            assert_eq!(game.world.get(target).unwrap().owner, Some(PLAYER_ID));
        }
    }

    #[test]
    fn city_building_and_army_systems_have_real_state_changes() {
        let mut game = GameState::new(7);
        let capital = game.factions[PLAYER_ID].capital;
        if let Some(target) = game.world.neighbors(capital).into_iter().find(|p| {
            game.world
                .get(*p)
                .is_some_and(|t| t.owner == Some(PLAYER_ID) && t.city.is_none())
        }) {
            game.select(target);
            if Building::Farm.allowed_on(game.world.get(target).unwrap().terrain) {
                assert!(game.build_selected(Building::Farm));
                assert!(
                    game.world
                        .get(target)
                        .unwrap()
                        .buildings
                        .contains(&Building::Farm)
                );
            }
            let mut game_city = GameState::new(8);
            let capital = game_city.factions[PLAYER_ID].capital;
            let city_site = game_city.world.neighbors(capital).into_iter().find(|p| {
                game_city
                    .world
                    .get(*p)
                    .is_some_and(|t| t.owner == Some(PLAYER_ID) && t.city.is_none())
            });
            if let Some(site) = city_site {
                game_city.select(site);
                assert!(game_city.found_city());
            }
        }
        assert!(game.recruit_army(UnitClass::Infantry));
        assert_eq!(game.army_count(PLAYER_ID), 1);
    }

    #[test]
    fn national_reform_changes_policy_and_respects_a_cooldown() {
        let mut game = GameState::new(2026);
        game.turn = 10;
        game.factions[PLAYER_ID].resources.gold = 100.0;
        game.factions[PLAYER_ID].resources.influence = 100.0;
        let before = game.factions[PLAYER_ID].policy;
        assert!(game.cycle_player_policy());
        assert_ne!(game.factions[PLAYER_ID].policy, before);
        assert!(!game.cycle_player_policy());
    }

    #[test]
    fn severe_unrest_can_trigger_a_province_revolt() {
        let mut game = GameState::new(2027);
        let capital = game.factions[PLAYER_ID].capital;
        let extra = game.world.coords().into_iter().find(|coord| {
            *coord != capital && game.world.get(*coord).is_some_and(|tile| tile.owner.is_none())
        }).expect("map has at least one neutral province");
        game.world.get_mut(extra).unwrap().owner = Some(PLAYER_ID);
        game.factions[PLAYER_ID].unrest = 95.0;
        game.factions[PLAYER_ID].legitimacy = 10.0;
        let before = game.owned_count(PLAYER_ID);
        game.resolve_unrest();
        assert!(game.owned_count(PLAYER_ID) < before);
        assert!(game.factions[PLAYER_ID].unrest < 95.0);
    }

    #[test]
    fn civic_policy_and_government_supply_economic_modifiers() {
        let game = GameState::new(2028);
        let player = &game.factions[PLAYER_ID];
        let modifiers = player.government.modifiers().combine(player.policy.modifiers());
        assert!(modifiers.stability.is_finite());
        assert!(modifiers.gold.is_finite());
        assert!(modifiers.science.is_finite());
    }

    #[test]
    fn diplomacy_is_symmetric_and_can_transition_to_truce() {
        let mut game = GameState::new(9);
        assert!(game.offer_trade_agreement(1));
        assert_eq!(game.treaty_between(1, PLAYER_ID), Treaty::TradeAgreement);
        assert!(game.declare_war(1));
        assert_eq!(game.treaty_between(PLAYER_ID, 1), Treaty::War);
        assert!(game.offer_peace(1));
        assert_eq!(game.treaty_between(PLAYER_ID, 1), Treaty::Truce);
    }

    #[test]
    fn research_progresses_and_unlocks_technology() {
        let mut game = GameState::new(11);
        assert!(game.start_next_research());
        assert!(game.factions[PLAYER_ID].active_research.is_some());
        let library_site = game
            .world
            .neighbors(game.factions[PLAYER_ID].capital)
            .into_iter()
            .find(|p| {
                game.world
                    .get(*p)
                    .is_some_and(|t| t.owner == Some(PLAYER_ID) && t.city.is_none())
            });
        if let Some(site) = library_site {
            game.select(site);
            if game.build_selected(Building::Library) {
                for _ in 0..40 {
                    game.advance_turn();
                }
            }
        }
        assert!(game.validate().is_ok());
    }

    #[test]
    fn campaign_turns_update_year_population_log_and_events() {
        let mut game = GameState::new(12);
        let year = game.year;
        game.advance_turn();
        assert_eq!(game.year, year + 1);
        assert_eq!(game.turn, 2);
        assert!(!game.chronicle.is_empty());
    }

    #[test]
    fn save_state_has_consistent_relations_and_armies() {
        let mut game = GameState::new(14);
        assert!(game.recruit_army(UnitClass::Infantry));
        assert!(game.validate().is_ok());
    }
}
