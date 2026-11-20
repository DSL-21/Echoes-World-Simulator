use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{
    diplomacy::Treaty,
    market::Commodity,
    world::{Coord, Terrain},
    GameState,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeRouteType {
    #[default]
    Land,
    Maritime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeRoute {
    pub id: u64,
    pub owner: usize,
    pub partner: usize,
    pub origin: Coord,
    pub destination: Coord,
    pub commodity: Commodity,
    #[serde(default)]
    pub route_type: TradeRouteType,
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

pub fn find_maritime_route(
    game: &GameState,
    owner: usize,
    partner: usize,
    origin: Coord,
    destination: Coord,
) -> Option<Vec<Coord>> {
    use super::economy::Building;

    if owner == partner || game.world.get(origin).is_none() || game.world.get(destination).is_none() {
        return None;
    }
    let has_port = |coord: Coord, faction: usize| {
        game.world.get(coord).is_some_and(|tile| {
            tile.owner == Some(faction)
                && tile.city.is_some()
                && tile.buildings.contains(&Building::Port)
                && tile.terrain == Terrain::Coast
        })
    };
    if !has_port(origin, owner) || !has_port(destination, partner) {
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
            if !visited.insert(next) { continue; }
            let Some(tile) = game.world.get(next) else { continue; };
            if next != destination && !matches!(tile.terrain, Terrain::Ocean | Terrain::Coast) { continue; }
            if next != destination && tile.owner.is_some_and(|realm| realm != owner && realm != partner) { continue; }
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
        let Some(tile) = game.world.get(*coord) else { return false; };
        let terrain_valid = match route.route_type {
            TradeRouteType::Land => tile.terrain.is_land(),
            TradeRouteType::Maritime => matches!(tile.terrain, Terrain::Ocean | Terrain::Coast),
        };
        if !terrain_valid || tile.owner.is_some_and(|realm| realm != route.owner && realm != route.partner) { return false; }
        if index > 0 && route.path[index - 1].manhattan(*coord) != 1 { return false; }
    }

    let has_city = |coord: Coord, owner: usize| game.world.get(coord).is_some_and(|tile| {
        tile.city.is_some() && tile.owner == Some(owner)
            && (route.route_type == TradeRouteType::Land || (tile.terrain == Terrain::Coast && tile.buildings.contains(&super::economy::Building::Port)))
    });
    has_city(route.origin, route.owner) && has_city(route.destination, route.partner)
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
    use crate::sim::{diplomacy::set_symmetric, PLAYER_ID};

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
            route_type: TradeRouteType::Land,
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
    fn maritime_routes_require_ports_and_follow_navigable_water() {
        use super::super::economy::Building;
        use super::super::world::Terrain;

        let mut game = GameState::new(2033);
        let origin = Coord::new(2, 2);
        let destination = Coord::new(5, 2);
        for coord in [origin, Coord::new(3, 2), Coord::new(4, 2), destination] {
            let tile = game.world.get_mut(coord).unwrap();
            tile.terrain = if coord == origin || coord == destination {
                Terrain::Coast
            } else {
                Terrain::Ocean
            };
            tile.owner = None;
            tile.city = None;
            tile.buildings.clear();
        }
        {
            let tile = game.world.get_mut(origin).unwrap();
            tile.owner = Some(PLAYER_ID);
            tile.city = Some("Harbour City".to_owned());
            tile.buildings.push(Building::Port);
        }
        {
            let tile = game.world.get_mut(destination).unwrap();
            tile.owner = Some(1);
            tile.city = Some("Farhaven".to_owned());
            tile.buildings.push(Building::Port);
        }
        set_symmetric(
            &mut game.relations,
            PLAYER_ID,
            1,
            Treaty::TradeAgreement,
            game.turn,
        );

        let path = find_maritime_route(&game, PLAYER_ID, 1, origin, destination).unwrap();
        assert_eq!(path, vec![origin, Coord::new(3, 2), Coord::new(4, 2), destination]);
        let route = TradeRoute {
            id: 1,
            owner: PLAYER_ID,
            partner: 1,
            origin,
            destination,
            commodity: Commodity::Food,
            route_type: TradeRouteType::Maritime,
            path,
            capacity: 4.0,
            active: true,
            last_trade_turn: 0,
        };
        assert!(route_is_open(&game, &route));

        game.world.get_mut(destination).unwrap().buildings.clear();
        assert!(find_maritime_route(&game, PLAYER_ID, 1, origin, destination).is_none());
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
