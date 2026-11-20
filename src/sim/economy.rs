use serde::{Deserialize, Serialize};

use super::world::Terrain;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub food: f32,
    pub timber: f32,
    pub stone: f32,
    pub iron: f32,
    pub gold: f32,
    pub science: f32,
    pub influence: f32,
}

impl Resources {
    pub fn starting() -> Self {
        Self { food: 55.0, timber: 42.0, stone: 28.0, iron: 12.0, gold: 90.0, science: 0.0, influence: 32.0 }
    }

    pub fn can_pay(&self, cost: &Self) -> bool {
        self.food >= cost.food && self.timber >= cost.timber && self.stone >= cost.stone
            && self.iron >= cost.iron && self.gold >= cost.gold
            && self.science >= cost.science && self.influence >= cost.influence
    }

    pub fn pay(&mut self, cost: &Self) -> bool {
        if !self.can_pay(cost) { return false; }
        self.food -= cost.food;
        self.timber -= cost.timber;
        self.stone -= cost.stone;
        self.iron -= cost.iron;
        self.gold -= cost.gold;
        self.science -= cost.science;
        self.influence -= cost.influence;
        true
    }

    pub fn add(&mut self, other: &Self) {
        self.food += other.food;
        self.timber += other.timber;
        self.stone += other.stone;
        self.iron += other.iron;
        self.gold += other.gold;
        self.science += other.science;
        self.influence += other.influence;
    }

    pub fn clamp_non_negative(&mut self) {
        self.food = self.food.max(0.0);
        self.timber = self.timber.max(0.0);
        self.stone = self.stone.max(0.0);
        self.iron = self.iron.max(0.0);
        self.gold = self.gold.max(0.0);
        self.science = self.science.max(0.0);
        self.influence = self.influence.max(0.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Building {
    Farm,
    LumberMill,
    Quarry,
    IronMine,
    Market,
    Barracks,
    Library,
    Aqueduct,
}

impl Building {
    pub fn name(self) -> &'static str {
        match self {
            Self::Farm => "Farm",
            Self::LumberMill => "Lumber mill",
            Self::Quarry => "Quarry",
            Self::IronMine => "Iron mine",
            Self::Market => "Market",
            Self::Barracks => "Barracks",
            Self::Library => "Library",
            Self::Aqueduct => "Aqueduct",
        }
    }

    pub fn cost(self) -> Resources {
        match self {
            Self::Farm => Resources { timber: 8.0, gold: 4.0, ..Default::default() },
            Self::LumberMill => Resources { timber: 10.0, stone: 3.0, gold: 5.0, ..Default::default() },
            Self::Quarry => Resources { timber: 8.0, stone: 4.0, gold: 5.0, ..Default::default() },
            Self::IronMine => Resources { timber: 12.0, stone: 8.0, gold: 8.0, ..Default::default() },
            Self::Market => Resources { timber: 10.0, stone: 5.0, gold: 10.0, ..Default::default() },
            Self::Barracks => Resources { timber: 14.0, stone: 10.0, iron: 3.0, gold: 12.0, ..Default::default() },
            Self::Library => Resources { timber: 12.0, stone: 8.0, gold: 14.0, ..Default::default() },
            Self::Aqueduct => Resources { stone: 16.0, gold: 12.0, ..Default::default() },
        }
    }

    pub fn allowed_on(self, terrain: Terrain) -> bool {
        match self {
            Self::Farm | Self::Market | Self::Barracks | Self::Library | Self::Aqueduct => terrain.is_land(),
            Self::LumberMill => matches!(terrain, Terrain::Forest | Terrain::Grassland),
            Self::Quarry => matches!(terrain, Terrain::Mountain | Terrain::Plains | Terrain::Tundra),
            Self::IronMine => matches!(terrain, Terrain::Mountain | Terrain::Tundra),
        }
    }

    pub fn yield_bonus(self) -> Resources {
        match self {
            Self::Farm => Resources { food: 4.0, ..Default::default() },
            Self::LumberMill => Resources { timber: 3.5, ..Default::default() },
            Self::Quarry => Resources { stone: 3.0, ..Default::default() },
            Self::IronMine => Resources { iron: 2.2, ..Default::default() },
            Self::Market => Resources { gold: 4.0, ..Default::default() },
            Self::Barracks => Resources { influence: 0.4, ..Default::default() },
            Self::Library => Resources { science: 2.5, ..Default::default() },
            Self::Aqueduct => Resources { food: 1.0, influence: 0.2, ..Default::default() },
        }
    }
}

pub fn terrain_yield(terrain: Terrain) -> Resources {
    match terrain {
        Terrain::Ocean => Resources::default(),
        Terrain::Coast => Resources { food: 0.8, gold: 0.35, ..Default::default() },
        Terrain::Grassland => Resources { food: 2.4, ..Default::default() },
        Terrain::Plains => Resources { food: 1.8, gold: 0.15, ..Default::default() },
        Terrain::Forest => Resources { food: 0.9, timber: 1.7, ..Default::default() },
        Terrain::Desert => Resources { gold: 0.7, ..Default::default() },
        Terrain::Mountain => Resources { stone: 1.7, iron: 0.45, ..Default::default() },
        Terrain::Tundra => Resources { food: 0.55, timber: 0.4, iron: 0.25, ..Default::default() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buildings_have_distinct_costs_and_outputs() {
        assert!(Building::Farm.cost().timber > 0.0);
        assert!(Building::Library.yield_bonus().science > 0.0);
        assert!(Building::IronMine.yield_bonus().iron > 0.0);
    }

    #[test]
    fn payment_is_atomic() {
        let mut stock = Resources::starting();
        let before = stock;
        let impossible = Resources { gold: 9999.0, ..Default::default() };
        assert!(!stock.pay(&impossible));
        assert_eq!(stock, before);
    }

    #[test]
    fn terrain_restricts_specialized_buildings() {
        assert!(Building::LumberMill.allowed_on(Terrain::Forest));
        assert!(!Building::LumberMill.allowed_on(Terrain::Desert));
        assert!(Building::IronMine.allowed_on(Terrain::Mountain));
    }
}
