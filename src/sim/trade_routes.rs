use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{
    diplomacy::Treaty,
    market::Commodity,
    world::Coord,
    GameState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeRoute {
    pub id: u64,
    pub owner: usize,
    pub partner: usize,
    pub origin: Coord,
    pub destination: Coord,
    pub commodity: Commodity,
    pub path: Vec<Coord>,
    pub capacity: f32,
    pub active: bool,
    pub last_trade_turn: u32,
}

pub fn find_land_route(
    game: &GameState,
    owner: usize,
    partner: usize,
    origin: Coord,
    destination: Coord,
) -> Option<Vec<Coord>> {
    if owner == partner || game.world.get(origin).is_none() || game.world.get(destination).is_none() {
        return None;
    }
    let mut queue = VecDeque::from([origin]);
    let mut visited = HashSet::from([origin]);
    let mut parents: HashMap<Coord, Coord> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if current == destination {
            let mut path = vec![destination];
            let mut cursor = destination;
            while cursor != origin {
                cursor = *parents.get(&cursor)?;
                path.push(cursor);
            }
            path.reverse();
            return Some(path);
        }

        for next in game.world.neighbors(current) {
            if !visited.insert(next) {
                continue;
            }
            let Some(tile) = game.world.get(next) else {
                continue;
            };
            if !tile.terrain.is_land() {
                continue;
            }
            if tile.owner.is_some_and(|realm| realm != owner && realm != partner) {
                continue;
            }
            parents.insert(next, current);
            queue.push_back(next);
        }
    }
    None
}

pub fn route_is_open(game: &GameState, route: &TradeRoute) -> bool {
    if route.owner >= game.factions.len()
        || route.partner >= game.factions.len()
        || route.owner == route.partner
        || route.path.len() < 2
        || route.path.first() != Some(&route.origin)
        || route.path.last() != Some(&route.destination)
    {
        return false;
    }

    if !matches!(
        game.treaty_between(route.owner, route.partner),
        Treaty::TradeAgreement | Treaty::Alliance
    ) {
        return false;
    }

    for (index, coord) in route.path.iter().enumerate() {
        let Some(tile) = game.world.get(*coord) else {
            return false;
        };
        if !tile.terrain.is_land()
            || tile.owner.is_some_and(|realm| realm != route.owner && realm != route.partner)
        {
            return false;
        }
        if index > 0 && route.path[index - 1].manhattan(*coord) != 1 {
            return false;
        }
    }

    game.world
        .get(route.origin)
        .is_some_and(|tile| tile.city.is_some() && tile.owner == Some(route.owner))
        && game.world.get(route.destination).is_some_and(|tile| {
            tile.city.is_some() && tile.owner == Some(route.partner)
        })
}

pub fn route_capacity(game: &GameState, path: &[Coord]) -> f32 {
    if path.len() < 2 {
        return 0.0;
    }
    let road_count = path
        .iter()
        .filter(|coord| {
            game.world
                .get(**coord)
                .is_some_and(|tile| tile.buildings.contains(&super::economy::Building::Road))
        })
        .count() as f32;
    let road_share = road_count / path.len() as f32;
    (5.0 - path.len() as f32 * 0.055 + road_share * 3.0).clamp(1.5, 8.0)
}

pub fn reserve_stock(commodity: Commodity, population: u32) -> f32 {
    match commodity {
        Commodity::Food => population as f32 * 0.65 + 12.0,
        Commodity::Timber => 8.0,
        Commodity::Stone => 5.0,
        Commodity::Iron => 4.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{diplomacy::set_symmetric, GameState, PLAYER_ID};

    #[test]
    fn land_route_connects_adjacent_partner_city() {
        let mut game = GameState::new(2031);
        let origin = game.factions[PLAYER_ID].capital;
        let destination = game.world.neighbors(origin)[0];
        let tile = game.world.get_mut(destination).unwrap();
        tile.terrain = super::super::world::Terrain::Grassland;
        tile.owner = Some(1);
        tile.city = Some("Trade Gate".to_owned());
        set_symmetric(
            &mut game.relations,
            PLAYER_ID,
            1,
            Treaty::TradeAgreement,
            game.turn,
        );

        let path = find_land_route(&game, PLAYER_ID, 1, origin, destination).unwrap();
        assert_eq!(path, vec![origin, destination]);
        let mut route = TradeRoute {
            id: 1,
            owner: PLAYER_ID,
            partner: 1,
            origin,
            destination,
            commodity: Commodity::Food,
            capacity: 4.0,
            path,
            active: true,
            last_trade_turn: 0,
        };
        assert!(route_is_open(&game, &route));
        route.path.reverse();
        assert!(!route_is_open(&game, &route));
    }

    #[test]
    fn road_infrastructure_increases_route_capacity() {
        let mut game = GameState::new(2032);
        let origin = game.factions[PLAYER_ID].capital;
        let destination = game.world.neighbors(origin)[0];
        let path = vec![origin, destination];
        let without_roads = route_capacity(&game, &path);
        game.world.get_mut(origin).unwrap().buildings.push(super::super::economy::Building::Road);
        let with_road = route_capacity(&game, &path);
        assert!(with_road > without_roads);
    }

    #[test]
    fn food_export_keeps_a_reserve_for_the_population() {
        assert_eq!(reserve_stock(Commodity::Food, 20), 25.0);
        assert_eq!(reserve_stock(Commodity::Iron, 20), 4.0);
    }
}
