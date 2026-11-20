use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Technology {
    Agriculture,
    Masonry,
    Currency,
    Metallurgy,
    Statecraft,
    Logistics,
    Navigation,
    Medicine,
    Printing,
}

impl Technology {
    pub const ALL: [Self; 9] = [
        Self::Agriculture, Self::Masonry, Self::Currency, Self::Metallurgy,
        Self::Statecraft, Self::Logistics, Self::Navigation, Self::Medicine, Self::Printing,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Agriculture => "Crop rotation",
            Self::Masonry => "Advanced masonry",
            Self::Currency => "Minted currency",
            Self::Metallurgy => "Ironworking",
            Self::Statecraft => "State administration",
            Self::Logistics => "Military logistics",
            Self::Navigation => "Blue-water navigation",
            Self::Medicine => "Practical medicine",
            Self::Printing => "Movable type",
        }
    }

    pub fn cost(self) -> f32 {
        match self {
            Self::Agriculture | Self::Masonry | Self::Currency => 45.0,
            Self::Metallurgy | Self::Statecraft | Self::Navigation | Self::Medicine => 75.0,
            Self::Logistics | Self::Printing => 110.0,
        }
    }

    pub fn prerequisites(self) -> &'static [Technology] {
        match self {
            Self::Agriculture | Self::Masonry | Self::Currency => &[],
            Self::Metallurgy => &[Self::Masonry],
            Self::Statecraft => &[Self::Currency],
            Self::Logistics => &[Self::Metallurgy],
            Self::Navigation => &[Self::Currency],
            Self::Medicine => &[Self::Agriculture],
            Self::Printing => &[Self::Statecraft],
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Agriculture => "Improves food output and population growth.",
            Self::Masonry => "Unlocks stronger fortifications and advanced stoneworks.",
            Self::Currency => "Improves tax collection and market income.",
            Self::Metallurgy => "Improves iron production and army equipment.",
            Self::Statecraft => "Strengthens administration and diplomatic influence.",
            Self::Logistics => "Improves army supply and reduces upkeep.",
            Self::Navigation => "Unlocks ocean trade and maritime expansion.",
            Self::Medicine => "Reduces the impact of disease and improves stability.",
            Self::Printing => "Increases research output and cultural prestige.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn technology_tree_has_valid_prerequisites() {
        for technology in Technology::ALL {
            for prerequisite in technology.prerequisites() {
                assert!(Technology::ALL.contains(prerequisite));
                assert_ne!(*prerequisite, technology);
            }
        }
        assert!(Technology::Logistics.prerequisites().contains(&Technology::Metallurgy));
    }
}
