pub mod diplomacy;
pub mod economy;
pub mod events;
pub mod military;
pub mod technology;
pub mod world;

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use self::{
    diplomacy::{new_matrix, set_symmetric, Relation, Treaty},
    economy::{terrain_yield, Building, Resources},
    events::WorldEvent,
    military::{recruitment_cost, Army, UnitClass},
    technology::Technology,
    world::{Coord, WorldMap},
};

pub const PLAYER_ID: usize = 0;
pub const WORLD_WIDTH: i32 = 44;
pub const WORLD_HEIGHT: i32 = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiPersonality {
    Expansionist,
    Merchant,
    Scholar,
    Defender,
    Balanced,
}

impl AiPersonality {
    fn from_id(id: usize) -> Self {
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
    pub personality: AiPersonality,
    pub active_research: Option<Technology>,
    pub research_progress: f32,
    pub technologies: Vec<Technology>,
}

impl Faction {
    fn new(id: usize, name: &str, color: [f32; 3], capital: Coord) -> Self {
        let mut resources = Resources::starting();
        if id > 0 {
            resources.gold += id as f32 * 4.0;
        }
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
            personality: AiPersonality::from_id(id),
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
        let start_targets = [
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

        let mut capitals = Vec::new();
        let mut factions = Vec::new();
        for (id, (name, color)) in definitions.iter().enumerate() {
            let capital = world.nearest_land(start_targets[id], &capitals);
            capitals.push(capital);
            factions.push(Faction::new(id, name, *color, capital));

            let city_names = ["Aurelia", "Greenhold", "Duskport", "Ivory Crown", "Azure Reach"];
            if let Some(tile) = world.get_mut(capital) {
                tile.owner = Some(id);
                tile.city = Some(city_names[id].to_owned());
                tile.population = 12;
                tile.discovered = true;
            }

            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    if dx.abs() + dy.abs() == 0 { continue; }
                    let point = Coord::new(capital.x + dx, capital.y + dy);
                    let Some(tile) = world.get_mut(point) else { continue; };
                    if tile.terrain.is_land() && tile.owner.is_none() {
                        tile.owner = Some(id);
                        tile.discovered = true;
                    }
                }
            }
        }

        let mut game = Self {
            world,
            factions,
            relations: new_matrix(definitions.len()),
            armies: Vec::new(),
            selected: capitals[PLAYER_ID],
            turn: 1,
            year: 1200,
            seed,
            next_army_id: 1,
            chronicle: VecDeque::new(),
        };
        game.add_log("The age of realms begins. Five powers rise across an uncharted world.");
        game.add_log("Your council awaits. Expand carefully, develop your provinces and shape history.");
        game
    }

    pub fn add_log(&mut self, message: impl Into<String>) {
        self.chronicle.push_back(message.into());
        while self.chronicle.len() > 16 { self.chronicle.pop_front(); }
    }

    pub fn select(&mut self, coord: Coord) -> bool {
        let Some(tile) = self.world.get(coord) else { return false; };
        let owner = self.faction_name(tile.owner).to_owned();
        let label = tile.city.clone().unwrap_or_else(|| tile.terrain.name().to_owned());
        self.selected = coord;
        self.add_log(format!("Selected {label} — {}, {} — {owner}.", coord.x + 1, coord.y + 1));
        true
    }

    pub fn owned_count(&self, faction_id: usize) -> usize {
        self.world.tiles.iter().filter(|tile| tile.owner == Some(faction_id)).count()
    }

    pub fn city_count(&self, faction_id: usize) -> usize {
        self.world.tiles.iter().filter(|tile| tile.owner == Some(faction_id) && tile.city.is_some()).count()
    }

    pub fn army_count(&self, faction_id: usize) -> usize {
        self.armies.iter().filter(|army| army.owner == faction_id).count()
    }

    pub fn faction_name(&self, faction_id: Option<usize>) -> &str {
        faction_id.and_then(|id| self.factions.get(id)).map(|f| f.name.as_str()).unwrap_or("Unclaimed")
    }

    pub fn treaty_between(&self, a: usize, b: usize) -> Treaty {
        self.relations.get(a).and_then(|row| row.get(b)).map(|r| r.treaty).unwrap_or(Treaty::None)
    }

    fn adjacent_to_faction(&self, coord: Coord, faction_id: usize) -> bool {
        self.world.neighbors(coord).iter().any(|neighbor| self.world.get(*neighbor).is_some_and(|tile| tile.owner == Some(faction_id)))
    }

    pub fn claim_selected(&mut self) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else { return false; };
        if !tile.terrain.is_land() {
            self.add_log("The deep ocean cannot be claimed without a maritime settlement.");
            return false;
        }
        if tile.owner.is_some() {
            self.add_log("This province already belongs to a realm.");
            return false;
        }
        if !self.adjacent_to_faction(coord, PLAYER_ID) {
            self.add_log("A claim must connect to your existing borders.");
            return false;
        }
        let cost = Resources { gold: 8.0, influence: 7.0, ..Default::default() };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            self.add_log("The council lacks the gold or influence to secure this province.");
            return false;
        }
        let terrain = tile.terrain;
        if let Some(tile) = self.world.get_mut(coord) {
            tile.owner = Some(PLAYER_ID);
            tile.discovered = true;
        }
        self.add_log(format!("New province secured: {}. The borders of the League expand.", terrain.name()));
        true
    }

    pub fn build_selected(&mut self, building: Building) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else { return false; };
        if tile.owner != Some(PLAYER_ID) {
            self.add_log("You can only construct buildings in provinces controlled by your realm.");
            return false;
        }
        if tile.city.is_some() {
            self.add_log("Urban administration occupies this province. Choose another province for rural development.");
            return false;
        }
        if tile.buildings.contains(&building) {
            self.add_log("This province already has that building.");
            return false;
        }
        let terrain = tile.terrain;
        if !building.allowed_on(terrain) {
            self.add_log(format!("{} cannot be built on {} terrain.", building.name(), terrain.name()));
            return false;
        }
        let cost = building.cost();
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            self.add_log("The treasury does not contain the required construction materials.");
            return false;
        }
        self.world.get_mut(coord).expect("selected tile exists").buildings.push(building);
        self.add_log(format!("{} completed in the {} province.", building.name(), terrain.name()));
        true
    }

    pub fn found_city(&mut self) -> bool {
        let coord = self.selected;
        let Some(tile) = self.world.get(coord) else { return false; };
        if tile.owner != Some(PLAYER_ID) || tile.city.is_some() {
            self.add_log("A new city must be founded in an owned province without an existing city.");
            return false;
        }
        let cost = Resources { food: 8.0, timber: 18.0, stone: 15.0, gold: 28.0, ..Default::default() };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            self.add_log("Founding a city requires more food, timber, stone and gold.");
            return false;
        }
        let names = ["Westhaven", "Stoneford", "New Aurelia", "Brightwater", "Highgarden", "Rivermarch", "Dawnspire", "Silverkeep"];
        let name_index = (self.city_count(PLAYER_ID) + self.turn as usize) % names.len();
        let name = names[name_index].to_owned();
        let tile = self.world.get_mut(coord).expect("selected tile exists");
        tile.city = Some(name.clone());
        tile.population = 3;
        self.add_log(format!("A new settlement is founded: {name}."));
        true
    }

    pub fn recruit_army(&mut self, class: UnitClass) -> bool {
        if self.city_count(PLAYER_ID) == 0 {
            self.add_log("Your realm needs a city before it can recruit an army.");
            return false;
        }
        let (gold, iron, timber) = recruitment_cost(class);
        let cost = Resources { gold, iron, timber, ..Default::default() };
        if !self.factions[PLAYER_ID].resources.pay(&cost) {
            self.add_log("The treasury cannot afford to equip another army.");
            return false;
        }
        let position = self.factions[PLAYER_ID].capital;
        let id = self.next_army_id;
        self.next_army_id += 1;
        self.armies.push(Army::new(id, PLAYER_ID, position, class));
        self.add_log(format!("{} recruited at the capital.", class.label()));
        true
    }

    pub fn offer_trade_agreement(&mut self, target: usize) -> bool {
        if target >= self.factions.len() || target == PLAYER_ID { return false; }
        if self.treaty_between(PLAYER_ID, target) == Treaty::War {
            self.add_log("Trade cannot be negotiated while the two realms are at war.");
            return false;
        }
        if self.relations[PLAYER_ID][target].opinion < -35 {
            self.add_log(format!("{} rejects the trade proposal. Relations are too hostile.", self.factions[target].name));
            return false;
        }
        set_symmetric(&mut self.relations, PLAYER_ID, target, Treaty::TradeAgreement, self.turn);
        let opinion = (self.relations[PLAYER_ID][target].opinion + 8).min(100);
        self.relations[PLAYER_ID][target].opinion = opinion;
        self.relations[target][PLAYER_ID].opinion = opinion;
        self.add_log(format!("A trade agreement is signed with {}.", self.factions[target].name));
        true
    }

    pub fn declare_war(&mut self, target: usize) -> bool {
        if target >= self.factions.len() || target == PLAYER_ID { return false; }
        if self.treaty_between(PLAYER_ID, target) == Treaty::War {
            self.add_log("This realm is already at war with you.");
            return false;
        }
        let name = self.factions[target].name.clone();
        set_symmetric(&mut self.relations, PLAYER_ID, target, Treaty::War, self.turn);
        self.relations[PLAYER_ID][target].opinion = -100;
        self.relations[target][PLAYER_ID].opinion = -100;
        self.factions[PLAYER_ID].war_exhaustion += 5.0;
        self.add_log(format!("WAR DECLARED — the Aurelian League enters a state of war with {name}."));
        true
    }

    pub fn offer_peace(&mut self, target: usize) -> bool {
        if target >= self.factions.len() || target == PLAYER_ID || self.treaty_between(PLAYER_ID, target) != Treaty::War {
            self.add_log("There is no active war with that realm to bring to an end.");
            return false;
        }
        let name = self.factions[target].name.clone();
        set_symmetric(&mut self.relations, PLAYER_ID, target, Treaty::Truce, self.turn);
        self.relations[PLAYER_ID][target].opinion = -15;
        self.relations[target][PLAYER_ID].opinion = -15;
        self.factions[PLAYER_ID].war_exhaustion = (self.factions[PLAYER_ID].war_exhaustion - 15.0).max(0.0);
        self.add_log(format!("A truce is signed with {name}. The armies fall silent."));
        true
    }

    pub fn start_next_research(&mut self) -> bool {
        let faction = &mut self.factions[PLAYER_ID];
        if faction.active_research.is_some() {
            self.add_log("Research is already underway. Let the scholars finish their current work.");
            return false;
        }
        let next = Technology::ALL.into_iter().find(|technology| {
            !faction.technologies.contains(technology)
                && technology.prerequisites().iter().all(|prerequisite| faction.technologies.contains(prerequisite))
        });
        let Some(technology) = next else {
            self.add_log("All currently known technologies have been researched.");
            return false;
        };
        faction.active_research = Some(technology);
        faction.research_progress = 0.0;
        self.add_log(format!("Scholars begin researching {}: {}", technology.name(), technology.description()));
        true
    }

    pub fn move_nearest_army_to_selected(&mut self) -> bool {
        let nearest = self.armies.iter().filter(|army| army.owner == PLAYER_ID)
            .min_by_key(|army| army.position.manhattan(self.selected)).map(|army| army.id);
        let Some(id) = nearest else {
            self.add_log("There is no army available to receive orders.");
            return false;
        };
        self.move_army(id, self.selected)
    }

    pub fn move_army(&mut self, army_id: u64, destination: Coord) -> bool {
        let Some(army_index) = self.armies.iter().position(|army| army.id == army_id) else { return false; };
        let attacker_owner = self.armies[army_index].owner;
        let attacker_name = self.armies[army_index].name.clone();
        if self.armies[army_index].position.manhattan(destination) != 1 {
            self.add_log("Armies can move one adjacent province per turn in this simulation build.");
            return false;
        }
        let Some(tile) = self.world.get(destination) else { return false; };
        if !tile.terrain.is_land() {
            self.add_log("An army cannot cross open ocean without naval transport.");
            return false;
        }
        let target_owner = tile.owner;
        if let Some(owner) = target_owner {
            if owner != attacker_owner && self.treaty_between(attacker_owner, owner) != Treaty::War {
                self.add_log("Entering another realm requires a declaration of war.");
                return false;
            }
        }

        if let Some(defender) = target_owner.filter(|owner| *owner != attacker_owner) {
            let attacker_power = self.armies[army_index].combat_power();
            let defending_armies: f32 = self.armies.iter()
                .filter(|other| other.owner == defender && other.position == destination)
                .map(Army::combat_power).sum();
            let garrison = self.world.get(destination)
                .map(|t| t.population as f32 * 0.7 + if t.city.is_some() { 25.0 } else { 0.0 })
                .unwrap_or(0.0);
            let defense_power = 45.0 + defending_armies + garrison;
            let enemy_name = self.factions[defender].name.clone();
            if attacker_power > defense_power {
                if let Some(tile) = self.world.get_mut(destination) {
                    tile.owner = Some(attacker_owner);
                    tile.population = (tile.population / 2).max(1);
                }
                self.armies.retain(|other| !(other.owner == defender && other.position == destination));
                if let Some(new_index) = self.armies.iter().position(|army| army.id == army_id) {
                    self.armies[new_index].position = destination;
                    self.armies[new_index].strength = (self.armies[new_index].strength * 0.72).max(20.0);
                    self.armies[new_index].experience += 8.0;
                }
                self.add_log(format!("VICTORY — {attacker_name} captures a province from {enemy_name}."));
            } else {
                self.armies[army_index].strength *= 0.48;
                self.armies[army_index].supply *= 0.65;
                self.add_log(format!("DEFEAT — the assault against {enemy_name} is repelled."));
                if self.armies[army_index].strength < 25.0 {
                    self.armies.remove(army_index);
                    self.add_log("The remnants of the defeated army disperse.");
                }
            }
            return true;
        }

        self.armies[army_index].position = destination;
        self.armies[army_index].supply = (self.armies[army_index].supply - 4.0).max(0.0);
        self.add_log(format!("{attacker_name} marches to {}, {}.", destination.x + 1, destination.y + 1));
        true
    }

    pub fn advance_turn(&mut self) {
        self.turn += 1;
        self.year += 1;

        for faction_id in 0..self.factions.len() {
            let production = self.calculate_production(faction_id);
            let cities = self.city_count(faction_id) as f32;
            let army_upkeep: f32 = self.armies.iter().filter(|army| army.owner == faction_id).map(|army| army.strength * 0.035).sum();
            let faction = &mut self.factions[faction_id];
            let consumption = faction.population as f32 * 0.22;
            faction.resources.add(&production);
            faction.resources.food -= consumption;
            faction.resources.gold += faction.population as f32 * faction.tax_rate * 0.12 + cities * 1.8 - army_upkeep;
            faction.resources.influence += 0.8 + cities * 0.55;
            faction.resources.clamp_non_negative();

            if faction.resources.food > faction.population as f32 * 0.85 + 12.0 && faction.stability > 35.0 && self.turn % 3 == 0 {
                faction.population += 1;
                faction.resources.food -= 5.0;
                faction.prestige += 0.25;
            } else if faction.resources.food < 2.0 {
                faction.population = faction.population.saturating_sub(1).max(1);
                faction.stability -= 4.0;
                faction.war_exhaustion = (faction.war_exhaustion + 0.5).min(100.0);
            } else {
                faction.stability = (faction.stability + 0.35).min(100.0);
            }
            faction.stability = (faction.stability - faction.war_exhaustion * 0.006).clamp(0.0, 100.0);
            faction.war_exhaustion = (faction.war_exhaustion - 0.18).max(0.0);
        }

        self.progress_research();
        self.update_trade_income();
        self.process_ai_turns();
        self.update_army_supply();
        self.apply_world_event();

        let provinces = self.owned_count(PLAYER_ID);
        let cities = self.city_count(PLAYER_ID);
        let population = self.factions[PLAYER_ID].population;
        self.add_log(format!("Year {} — {provinces} provinces, {cities} cities, population {population}.", self.year));
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
                province.science += 1.6;
            }
            output.add(&province);
        }
        if faction.has_technology(Technology::Agriculture) { output.food *= 1.15; }
        if faction.has_technology(Technology::Currency) { output.gold *= 1.12; }
        if faction.has_technology(Technology::Metallurgy) { output.iron *= 1.20; }
        if faction.has_technology(Technology::Printing) { output.science *= 1.20; }
        output
    }

    fn progress_research(&mut self) {
        for faction_id in 0..self.factions.len() {
            let active = self.factions[faction_id].active_research;
            let Some(technology) = active else { continue; };
            let available = self.factions[faction_id].resources.science.min(7.0);
            self.factions[faction_id].resources.science -= available;
            self.factions[faction_id].research_progress += available;
            if self.factions[faction_id].research_progress >= technology.cost() {
                self.factions[faction_id].technologies.push(technology);
                self.factions[faction_id].active_research = None;
                self.factions[faction_id].research_progress = 0.0;
                self.factions[faction_id].prestige += 5.0;
                let name = self.factions[faction_id].name.clone();
                self.add_log(format!("{name} discovers {}.", technology.name()));
            }
        }
    }

    fn update_trade_income(&mut self) {
        for a in 0..self.factions.len() {
            for b in (a + 1)..self.factions.len() {
                if self.treaty_between(a, b) == Treaty::TradeAgreement {
                    self.factions[a].resources.gold += 1.2;
                    self.factions[b].resources.gold += 1.2;
                    let opinion = (self.relations[a][b].opinion + 1).min(100);
                    self.relations[a][b].opinion = opinion;
                    self.relations[b][a].opinion = opinion;
                }
            }
        }
    }

    fn process_ai_turns(&mut self) {
        for faction_id in 1..self.factions.len() {
            if self.factions[faction_id].active_research.is_none() {
                let faction = &mut self.factions[faction_id];
                let next = Technology::ALL.into_iter().find(|technology| {
                    !faction.technologies.contains(technology)
                        && technology.prerequisites().iter().all(|p| faction.technologies.contains(p))
                });
                faction.active_research = next;
            }

            let can_expand = self.factions[faction_id].resources.gold >= 8.0 && self.factions[faction_id].resources.influence >= 7.0;
            if can_expand {
                if let Some(coord) = self.best_expansion(faction_id) {
                    let cost = Resources { gold: 8.0, influence: 7.0, ..Default::default() };
                    if self.factions[faction_id].resources.pay(&cost) {
                        if let Some(tile) = self.world.get_mut(coord) { tile.owner = Some(faction_id); tile.discovered = true; }
                    }
                }
            }

            if self.turn % 9 == 0 && self.factions[faction_id].resources.gold >= 25.0 {
                let (gold, iron, timber) = recruitment_cost(UnitClass::Infantry);
                let cost = Resources { gold, iron, timber, ..Default::default() };
                if self.factions[faction_id].resources.pay(&cost) {
                    let id = self.next_army_id;
                    let position = self.factions[faction_id].capital;
                    self.next_army_id += 1;
                    self.armies.push(Army::new(id, faction_id, position, UnitClass::Infantry));
                }
            }
        }
    }

    fn best_expansion(&self, faction_id: usize) -> Option<Coord> {
        let mut best: Option<(Coord, f32)> = None;
        for coord in self.world.coords() {
            let Some(tile) = self.world.get(coord) else { continue; };
            if tile.owner.is_some() || !tile.terrain.is_land() || !self.adjacent_to_faction(coord, faction_id) { continue; }
            let variance = ((coord.x * 17 + coord.y * 31 + faction_id as i32 * 13).rem_euclid(17)) as f32 / 100.0;
            let score = tile.terrain.expansion_value() + variance;
            if best.is_none_or(|(_, current)| score > current) { best = Some((coord, score)); }
        }
        best.map(|(coord, _)| coord)
    }

    fn update_army_supply(&mut self) {
        for army in &mut self.armies {
            let supplied = self.world.get(army.position).is_some_and(|tile| tile.owner == Some(army.owner));
            if supplied { army.supply = (army.supply + 8.0).min(100.0); }
            else { army.supply = (army.supply - 9.0).max(0.0); }
            if army.supply < 20.0 { army.strength = (army.strength - 2.0).max(10.0); }
        }
    }

    fn apply_world_event(&mut self) {
        let Some(event) = WorldEvent::roll(self.seed, self.turn) else { return; };
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
        self.add_log(format!("WORLD EVENT — {}. The realm must adapt to changing fortunes.", event.title()));
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.world.width <= 0 || self.world.height <= 0 || self.world.tiles.len() != (self.world.width * self.world.height) as usize {
            return Err("The saved world map has invalid dimensions.".to_owned());
        }
        if self.factions.is_empty() || PLAYER_ID >= self.factions.len() {
            return Err("The save does not contain a playable realm.".to_owned());
        }
        if self.world.get(self.selected).is_none() {
            return Err("The selected province is outside the saved map.".to_owned());
        }
        if self.relations.len() != self.factions.len() || self.relations.iter().any(|row| row.len() != self.factions.len()) {
            return Err("The diplomacy table is inconsistent with the realm list.".to_owned());
        }
        if self.world.tiles.iter().any(|tile| tile.owner.is_some_and(|owner| owner >= self.factions.len())) {
            return Err("A province refers to a realm that does not exist.".to_owned());
        }
        if self.armies.iter().any(|army| army.owner >= self.factions.len() || self.world.get(army.position).is_none()) {
            return Err("An army refers to an invalid realm or province.".to_owned());
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
    fn player_can_claim_connected_neutral_land() {
        let mut game = GameState::new(42);
        let target = game.world.coords().into_iter().find(|coord| {
            game.world.get(*coord).is_some_and(|tile| tile.terrain.is_land() && tile.owner.is_none())
                && game.adjacent_to_faction(*coord, PLAYER_ID)
        }).expect("generated realm should have an adjacent neutral province");
        game.select(target);
        assert!(game.claim_selected());
        assert_eq!(game.world.get(target).unwrap().owner, Some(PLAYER_ID));
    }

    #[test]
    fn buildings_spend_resources_and_change_province_state() {
        let mut game = GameState::new(7);
        let target = game.world.coords().into_iter().find(|coord| {
            game.world.get(*coord).is_some_and(|tile| tile.owner == Some(PLAYER_ID) && tile.city.is_none() && Building::Farm.allowed_on(tile.terrain))
        }).expect("starting realm should have a developable province");
        let before = game.factions[PLAYER_ID].resources.timber;
        game.select(target);
        assert!(game.build_selected(Building::Farm));
        assert!(game.factions[PLAYER_ID].resources.timber < before);
        assert!(game.world.get(target).unwrap().buildings.contains(&Building::Farm));
    }

    #[test]
    fn diplomacy_changes_both_sides_and_war_can_end() {
        let mut game = GameState::new(9);
        assert!(game.offer_trade_agreement(1));
        assert_eq!(game.treaty_between(1, PLAYER_ID), Treaty::TradeAgreement);
        assert!(game.declare_war(1));
        assert_eq!(game.treaty_between(PLAYER_ID, 1), Treaty::War);
        assert!(game.offer_peace(1));
        assert_eq!(game.treaty_between(PLAYER_ID, 1), Treaty::Truce);
    }

    #[test]
    fn research_completes_and_unlocks_follow_up_technology() {
        let mut game = GameState::new(11);
        assert!(game.start_next_research());
        assert_eq!(game.factions[PLAYER_ID].active_research, Some(Technology::Agriculture));
        for _ in 0..35 { game.advance_turn(); }
        assert!(game.factions[PLAYER_ID].technologies.contains(&Technology::Agriculture));
        assert!(game.start_next_research());
        assert_eq!(game.factions[PLAYER_ID].active_research, Some(Technology::Masonry));
    }

    #[test]
    fn turns_advance_economy_and_world_chronicle() {
        let mut game = GameState::new(12);
        let starting_year = game.year;
        game.advance_turn();
        assert_eq!(game.year, starting_year + 1);
        assert_eq!(game.turn, 2);
        assert!(!game.chronicle.is_empty());
    }

    #[test]
    fn armies_can_be_recruited_and_have_identity() {
        let mut game = GameState::new(14);
        assert!(game.recruit_army(UnitClass::Infantry));
        assert_eq!(game.armies.len(), 1);
        assert_eq!(game.armies[0].owner, PLAYER_ID);
    }
}
