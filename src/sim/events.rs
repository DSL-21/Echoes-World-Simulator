#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldEvent {
    BountifulHarvest,
    Drought,
    MerchantCaravan,
    Plague,
    IronDiscovery,
    PublicFestival,
    BanditRaids,
    PoliticalScandal,
}

impl WorldEvent {
    pub fn roll(seed: u64, turn: u32) -> Option<Self> {
        if turn == 0 || turn % 4 != 0 {
            return None;
        }
        let mut value = seed ^ (turn as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        value ^= value >> 30;
        value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value ^= value >> 27;
        match value % 8 {
            0 => Some(Self::BountifulHarvest),
            1 => Some(Self::Drought),
            2 => Some(Self::MerchantCaravan),
            3 => Some(Self::Plague),
            4 => Some(Self::IronDiscovery),
            5 => Some(Self::PublicFestival),
            6 => Some(Self::BanditRaids),
            _ => Some(Self::PoliticalScandal),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::BountifulHarvest => "Bountiful harvest",
            Self::Drought => "Drought",
            Self::MerchantCaravan => "Merchant caravan",
            Self::Plague => "Plague",
            Self::IronDiscovery => "Iron discovery",
            Self::PublicFestival => "Public festival",
            Self::BanditRaids => "Bandit raids",
            Self::PoliticalScandal => "Political scandal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_deterministic_and_periodic() {
        assert_eq!(WorldEvent::roll(7, 3), None);
        assert_eq!(WorldEvent::roll(7, 8), WorldEvent::roll(7, 8));
        assert_ne!(WorldEvent::roll(7, 4), None);
    }
}
