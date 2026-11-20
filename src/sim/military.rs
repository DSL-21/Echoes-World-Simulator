use serde::{Deserialize, Serialize};

use super::world::Coord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitClass {
    Infantry,
    Cavalry,
    Siege,
}

impl UnitClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Infantry => "Infantry",
            Self::Cavalry => "Cavalry",
            Self::Siege => "Siege",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Army {
    pub id: u64,
    pub owner: usize,
    pub name: String,
    pub position: Coord,
    pub strength: f32,
    pub supply: f32,
    pub class: UnitClass,
    pub experience: f32,
}

impl Army {
    pub fn new(id: u64, owner: usize, position: Coord, class: UnitClass) -> Self {
        let label = match class {
            UnitClass::Infantry => "Field Army",
            UnitClass::Cavalry => "Mounted Host",
            UnitClass::Siege => "Siege Train",
        };
        Self { id, owner, name: format!("{label} {id}"), position, strength: 100.0, supply: 100.0, class, experience: 0.0 }
    }

    pub fn combat_power(&self) -> f32 {
        let class_bonus = match self.class { UnitClass::Infantry => 1.0, UnitClass::Cavalry => 1.12, UnitClass::Siege => 0.92 };
        self.strength * class_bonus * (0.65 + self.supply / 250.0) * (1.0 + self.experience / 250.0)
    }
}

pub fn recruitment_cost(class: UnitClass) -> (f32, f32, f32) {
    match class {
        UnitClass::Infantry => (25.0, 8.0, 0.0),
        UnitClass::Cavalry => (38.0, 12.0, 0.0),
        UnitClass::Siege => (42.0, 16.0, 5.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supply_and_experience_affect_army_power() {
        let mut army = Army::new(1, 0, Coord::new(2, 2), UnitClass::Infantry);
        let supplied = army.combat_power();
        army.supply = 0.0;
        assert!(army.combat_power() < supplied);
        army.supply = 100.0;
        army.experience = 50.0;
        assert!(army.combat_power() > supplied);
    }
}
