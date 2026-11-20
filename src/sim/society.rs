use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernmentType {
    #[default]
    Monarchy,
    Republic,
    MerchantLeague,
    Theocracy,
    MilitaryJunta,
    Council,
}

impl GovernmentType {
    pub fn for_faction(id: usize) -> Self {
        match id % 5 {
            0 => Self::Republic,
            1 => Self::Council,
            2 => Self::MilitaryJunta,
            3 => Self::Theocracy,
            _ => Self::MerchantLeague,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Monarchy => "Monarchy",
            Self::Republic => "Republic",
            Self::MerchantLeague => "Merchant league",
            Self::Theocracy => "Theocracy",
            Self::MilitaryJunta => "Military junta",
            Self::Council => "Council government",
        }
    }

    pub fn modifiers(self) -> SocietyModifiers {
        match self {
            Self::Monarchy => SocietyModifiers { tax: 0.10, stability: 0.10, ..Default::default() },
            Self::Republic => SocietyModifiers { gold: 0.04, science: 0.08, stability: -0.02, ..Default::default() },
            Self::MerchantLeague => SocietyModifiers { gold: 0.12, trade_income: 0.55, influence: 0.05, ..Default::default() },
            Self::Theocracy => SocietyModifiers { science: 0.04, stability: 0.24, ..Default::default() },
            Self::MilitaryJunta => SocietyModifiers { army_upkeep: 0.14, influence: 0.10, stability: -0.06, ..Default::default() },
            Self::Council => SocietyModifiers { influence: 0.16, stability: 0.12, ..Default::default() },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CivicPolicy {
    #[default]
    Balanced,
    Agrarian,
    FreeTrade,
    Militarist,
    Scholarly,
    CivicRights,
}

impl CivicPolicy {
    pub const ALL: [Self; 6] = [
        Self::Balanced,
        Self::Agrarian,
        Self::FreeTrade,
        Self::Militarist,
        Self::Scholarly,
        Self::CivicRights,
    ];

    pub fn for_faction(id: usize) -> Self {
        match id % 5 {
            0 => Self::Balanced,
            1 => Self::Agrarian,
            2 => Self::Militarist,
            3 => Self::Scholarly,
            _ => Self::FreeTrade,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced administration",
            Self::Agrarian => "Agrarian charter",
            Self::FreeTrade => "Free trade",
            Self::Militarist => "Militarist doctrine",
            Self::Scholarly => "Scholarly patronage",
            Self::CivicRights => "Civic rights",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Balanced => "A moderate approach with no major strengths or penalties.",
            Self::Agrarian => "Prioritizes food security and rural prosperity.",
            Self::FreeTrade => "Encourages commerce, markets and international exchange.",
            Self::Militarist => "Supports the armed forces and reduces army upkeep.",
            Self::Scholarly => "Funds learning and accelerates technological progress.",
            Self::CivicRights => "Strengthens public order, legitimacy and civic participation.",
        }
    }

    pub fn modifiers(self) -> SocietyModifiers {
        match self {
            Self::Balanced => SocietyModifiers::default(),
            Self::Agrarian => SocietyModifiers { food: 0.14, tax: -0.04, unrest: -0.18, ..Default::default() },
            Self::FreeTrade => SocietyModifiers { gold: 0.10, trade_income: 0.45, tax: 0.02, ..Default::default() },
            Self::Militarist => SocietyModifiers { army_upkeep: 0.20, influence: 0.08, unrest: 0.08, ..Default::default() },
            Self::Scholarly => SocietyModifiers { science: 0.18, gold: -0.035, stability: -0.025, ..Default::default() },
            Self::CivicRights => SocietyModifiers { stability: 0.28, unrest: -0.30, influence: 0.06, tax: -0.06, ..Default::default() },
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|policy| *policy == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SocietyModifiers {
    pub food: f32,
    pub timber: f32,
    pub stone: f32,
    pub iron: f32,
    pub gold: f32,
    pub science: f32,
    pub influence: f32,
    pub tax: f32,
    pub trade_income: f32,
    pub army_upkeep: f32,
    pub stability: f32,
    pub unrest: f32,
}

impl SocietyModifiers {
    pub fn combine(self, other: Self) -> Self {
        Self {
            food: self.food + other.food,
            timber: self.timber + other.timber,
            stone: self.stone + other.stone,
            iron: self.iron + other.iron,
            gold: self.gold + other.gold,
            science: self.science + other.science,
            influence: self.influence + other.influence,
            tax: self.tax + other.tax,
            trade_income: self.trade_income + other.trade_income,
            army_upkeep: self.army_upkeep + other.army_upkeep,
            stability: self.stability + other.stability,
            unrest: self.unrest + other.unrest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_faction_has_a_distinct_government_identity() {
        let governments: Vec<_> = (0..5).map(GovernmentType::for_faction).collect();
        assert_eq!(governments.len(), 5);
        assert!(governments.contains(&GovernmentType::MerchantLeague));
        assert!(governments.contains(&GovernmentType::Theocracy));
        assert!(governments.contains(&GovernmentType::MilitaryJunta));
    }

    #[test]
    fn policies_have_tradeoffs_and_cycle_without_duplicates() {
        for policy in CivicPolicy::ALL {
            let next = policy.next();
            assert_ne!(policy, next);
            let modifiers = policy.modifiers();
            assert!(modifiers.food.is_finite());
            assert!(modifiers.unrest.is_finite());
        }
    }

    #[test]
    fn government_and_policy_modifiers_combine() {
        let modifiers = GovernmentType::MerchantLeague.modifiers()
            .combine(CivicPolicy::FreeTrade.modifiers());
        assert!(modifiers.gold > 0.2);
        assert!(modifiers.trade_income > 0.9);
    }
}
