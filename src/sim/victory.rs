use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VictoryPath {
    TerritorialHegemony,
    ScientificAscendancy,
    EconomicDominion,
    CulturalLegacy,
}

impl VictoryPath {
    pub fn name(self) -> &'static str {
        match self {
            Self::TerritorialHegemony => "Territorial hegemony",
            Self::ScientificAscendancy => "Scientific ascendancy",
            Self::EconomicDominion => "Economic dominion",
            Self::CulturalLegacy => "Cultural legacy",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::TerritorialHegemony => "Control at least forty percent of all land provinces.",
            Self::ScientificAscendancy => "Complete seven technologies in the research tree.",
            Self::EconomicDominion => "Reach four cities, thirty-five population and six hundred gold.",
            Self::CulturalLegacy => "Reach one hundred and twenty prestige with stable, legitimate rule.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignOutcome {
    pub winner_id: usize,
    pub year: u32,
    pub path: VictoryPath,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn victory_paths_have_distinct_descriptions() {
        let descriptions: Vec<_> = [
            VictoryPath::TerritorialHegemony,
            VictoryPath::ScientificAscendancy,
            VictoryPath::EconomicDominion,
            VictoryPath::CulturalLegacy,
        ]
        .iter()
        .map(|path| path.description())
        .collect();
        assert_eq!(descriptions.len(), 4);
        assert!(descriptions.iter().all(|description| !description.is_empty()));
        assert_ne!(descriptions[0], descriptions[1]);
    }
}
