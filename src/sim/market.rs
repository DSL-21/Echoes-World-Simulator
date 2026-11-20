use serde::{Deserialize, Serialize};

use super::economy::Resources;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MarketState {
    pub food_price: f32,
    pub timber_price: f32,
    pub stone_price: f32,
    pub iron_price: f32,
    pub last_supply: Resources,
    pub last_demand: Resources,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            food_price: 1.0,
            timber_price: 1.2,
            stone_price: 0.9,
            iron_price: 2.0,
            last_supply: Resources::default(),
            last_demand: Resources::default(),
        }
    }
}

impl MarketState {
    pub fn update(&mut self, supply: Resources, demand: Resources) {
        self.last_supply = supply;
        self.last_demand = demand;
        self.food_price = Self::adjust_price(self.food_price, 1.0, supply.food, demand.food);
        self.timber_price = Self::adjust_price(self.timber_price, 1.2, supply.timber, demand.timber);
        self.stone_price = Self::adjust_price(self.stone_price, 0.9, supply.stone, demand.stone);
        self.iron_price = Self::adjust_price(self.iron_price, 2.0, supply.iron, demand.iron);
    }

    fn adjust_price(current: f32, base: f32, supply: f32, demand: f32) -> f32 {
        let ratio = (demand.max(0.1) / supply.max(0.1)).sqrt().clamp(0.55, 2.4);
        let target = (base * ratio).clamp(base * 0.45, base * 2.8);
        (current * 0.72 + target * 0.28).clamp(base * 0.4, base * 3.0)
    }

    pub fn trade_index(&self) -> f32 {
        self.food_price * 0.40
            + self.timber_price * 0.30
            + self.stone_price * 0.20
            + self.iron_price * 0.10
    }

    pub fn is_valid(&self) -> bool {
        [
            self.food_price,
            self.timber_price,
            self.stone_price,
            self.iron_price,
        ]
        .iter()
        .all(|price| price.is_finite() && *price > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scarce_commodities_become_more_expensive() {
        let mut market = MarketState::default();
        market.update(
            Resources { food: 4.0, timber: 2.0, stone: 2.0, iron: 1.0, ..Default::default() },
            Resources { food: 40.0, timber: 20.0, stone: 18.0, iron: 10.0, ..Default::default() },
        );
        assert!(market.food_price > 1.0);
        assert!(market.timber_price > 1.2);
        assert!(market.iron_price > 2.0);
        assert!(market.is_valid());
    }

    #[test]
    fn abundant_supply_pushes_prices_toward_floor() {
        let mut market = MarketState::default();
        let initial = market.food_price;
        market.update(
            Resources { food: 80.0, timber: 50.0, stone: 50.0, iron: 30.0, ..Default::default() },
            Resources { food: 2.0, timber: 1.0, stone: 1.0, iron: 1.0, ..Default::default() },
        );
        assert!(market.food_price < initial);
        assert!(market.food_price >= 0.4);
    }

    #[test]
    fn trade_index_aggregates_market_prices() {
        let market = MarketState::default();
        assert!((market.trade_index() - 1.14).abs() < 0.001);
    }
}
