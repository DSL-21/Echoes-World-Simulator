use std::{cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet, VecDeque}};

use super::{
    diplomacy::{set_symmetric, Treaty},
    economy::{Building, Resources},
    military::Army,
    AiPersonality, GameState, PLAYER_ID,
    world::Coord,
};

/// Advances the parts of the world that are controlled by rival realms.
pub fn process_turn(game: &mut GameState) {
    develop_cities(game);
    develop_provinces(game);
    simulate_diplomacy(game);
    issue_war_orders(game);
}

fn develop_cities(game: &mut GameState) {
    for faction_id in 1..game.factions.len() {
        let interval = 10 + faction_id as u32 * 2;
        if game.turn % interval != 0
            || game.owned_count(faction_id) < 6
            || game.city_count(faction_id) >= 4
        {
            continue;
        }

        let capital = game.factions[faction_id].capital;
        let candidate = game.world.coords().into_iter().filter_map(|coord| {
            let tile = game.world.get(coord)?;
            if tile.owner != Some(faction_id) || tile.city.is_some() { return None; }
            let score = tile.terrain.expansion_value() * 10.0 - coord.manhattan(capital) as f32 * 0.25;
            Some((coord, score))
        }).max_by(|a, b| a.1.total_cmp(&b.1));

        let Some((coord, _)) = candidate else { continue; };
        let cost = Resources { food: 8.0, timber: 18.0, stone: 15.0, gold: 28.0, ..Default::default() };
        if !game.factions[faction_id].resources.pay(&cost) { continue; }

        let number = game.city_count(faction_id) + 1;
        let name = format!("{} {}", game.factions[faction_id].name.split_whitespace().next().unwrap_or("Settlement"), number);
        if let Some(tile) = game.world.get_mut(coord) {
            tile.city = Some(name.clone());
            tile.population = 3;
        }
        game.factions[faction_id].population += 3;
        let realm = game.factions[faction_id].name.clone();
        game.add_log(format!("{realm} founds a new city: {name}."));
    }
}

fn develop_provinces(game: &mut GameState) {
    for faction_id in 1..game.factions.len() {
        if game.turn % 4 != faction_id as u32 % 4 { continue; }
        let personality = game.factions[faction_id].personality;
        let priorities: &[Building] = match personality {
            AiPersonality::Expansionist => &[Building::Farm, Building::Quarry, Building::Market],
            AiPersonality::Merchant => &[Building::Market, Building::LumberMill, Building::Farm],
            AiPersonality::Scholar => &[Building::Library, Building::Market, Building::Farm],
            AiPersonality::Defender => &[Building::Barracks, Building::Quarry, Building::Farm],
            AiPersonality::Balanced => &[Building::Farm, Building::Market, Building::LumberMill],
        };

        for building in priorities {
            let candidate = game.world.coords().into_iter().filter_map(|coord| {
                let tile = game.world.get(coord)?;
                if tile.owner != Some(faction_id) || tile.city.is_some()
                    || tile.buildings.len() >= 2 || tile.buildings.contains(building)
                    || !building.allowed_on(tile.terrain) { return None; }
                let bonus = building.yield_bonus();
                let score = bonus.food + bonus.timber + bonus.stone * 1.3
                    + bonus.iron * 1.8 + bonus.gold * 1.2
                    + bonus.science * 1.5 + bonus.influence;
                Some((coord, score + tile.terrain.expansion_value() * 0.05))
            }).max_by(|a, b| a.1.total_cmp(&b.1));

            let Some((coord, _)) = candidate else { continue; };
            if !game.factions[faction_id].resources.pay(&building.cost()) { continue; }
            if let Some(tile) = game.world.get_mut(coord) { tile.buildings.push(*building); }
            break;
        }
    }
}

fn simulate_diplomacy(game: &mut GameState) {
    let count = game.factions.len();
    for a in 0..count {
        for b in (a + 1)..count {
            let treaty = game.relations[a][b].treaty;
            let changed_turn = game.relations[a][b].changed_turn;

            if treaty == Treaty::War {
                game.factions[a].war_exhaustion = (game.factions[a].war_exhaustion + 0.22).min(100.0);
                game.factions[b].war_exhaustion = (game.factions[b].war_exhaustion + 0.22).min(100.0);
                if game.turn.saturating_sub(changed_turn) >= 30 {
                    set_symmetric(&mut game.relations, a, b, Treaty::Truce, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("A negotiated truce ends the war between {first} and {second}."));
                }
                continue;
            }

            let border_tension = shares_border(game, a, b);
            let pa = game.factions[a].personality;
            let pb = game.factions[b].personality;
            let friendly = matches!((pa, pb),
                (AiPersonality::Merchant, AiPersonality::Scholar)
                | (AiPersonality::Scholar, AiPersonality::Merchant)
                | (AiPersonality::Merchant, AiPersonality::Balanced)
                | (AiPersonality::Balanced, AiPersonality::Merchant)
                | (AiPersonality::Scholar, AiPersonality::Balanced)
                | (AiPersonality::Balanced, AiPersonality::Scholar)
                | (AiPersonality::Defender, AiPersonality::Defender));
            let initial_affinity = if friendly { 18 } else if pa == AiPersonality::Expansionist || pb == AiPersonality::Expansionist { -8 } else { 4 };
            let mut opinion = game.relations[a][b].opinion;
            if game.turn == 2 && opinion == 0 {
                opinion = initial_affinity;
            } else if border_tension {
                opinion = (opinion - if pa == AiPersonality::Expansionist || pb == AiPersonality::Expansionist { 2 } else { 1 }).max(-100);
            } else if friendly && game.turn % 4 == 0 {
                opinion = (opinion + 1).min(100);
            } else if !friendly && game.turn % 10 == 0 {
                opinion = (opinion - 1).max(-100);
            }
            game.relations[a][b].opinion = opinion;
            game.relations[b][a].opinion = opinion;

            match treaty {
                Treaty::None if game.turn % 12 == 0 && trade_proposal_accepted(game, a, b, opinion) => {
                    set_symmetric(&mut game.relations, a, b, Treaty::TradeAgreement, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("DIPLOMACY — {first} proposes a commercial treaty; {second} accepts the trade agreement."));
                }
                Treaty::None if game.turn % 12 == 0 && war_proposal_accepted(game, a, b, opinion) => {
                    set_symmetric(&mut game.relations, a, b, Treaty::War, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("WAR — diplomatic negotiations collapse and {first} declares war on {second}."));
                }
                Treaty::TradeAgreement if game.turn % 12 == 0 && alliance_proposal_accepted(game, a, b, opinion) => {
                    set_symmetric(&mut game.relations, a, b, Treaty::Alliance, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("ALLIANCE — {first} proposes a defensive pact and {second} accepts."));
                }
                Treaty::Truce if game.turn.saturating_sub(changed_turn) >= 24 && opinion >= 5 => {
                    set_symmetric(&mut game.relations, a, b, Treaty::None, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("The truce between {first} and {second} expires; normal relations resume."));
                }
                _ => {}
            }
        }
    }
}

fn trade_proposal_accepted(game: &GameState, a: usize, b: usize, opinion: i16) -> bool {
    if opinion < 18 {
        return false;
    }
    let first = &game.factions[a];
    let second = &game.factions[b];
    let personality_bonus = [first.personality, second.personality]
        .into_iter()
        .map(|personality| match personality {
            AiPersonality::Merchant => 12,
            AiPersonality::Scholar => 5,
            AiPersonality::Expansionist => -5,
            AiPersonality::Balanced | AiPersonality::Defender => 0,
        })
        .sum::<i16>();
    let economic_need = if first.resources.gold < 35.0 || second.resources.gold < 35.0 {
        8
    } else {
        0
    };
    opinion + personality_bonus + economic_need >= 30
}

fn alliance_proposal_accepted(game: &GameState, a: usize, b: usize, opinion: i16) -> bool {
    if opinion < 48 {
        return false;
    }
    let shared_enemy = (0..game.factions.len()).any(|other| {
        other != a
            && other != b
            && game.treaty_between(a, other) == Treaty::War
            && game.treaty_between(b, other) == Treaty::War
    });
    let defensive_pair = game.factions[a].personality == AiPersonality::Defender
        && game.factions[b].personality == AiPersonality::Defender;
    opinion >= 68 || shared_enemy || defensive_pair
}

fn war_proposal_accepted(game: &GameState, a: usize, b: usize, opinion: i16) -> bool {
    if opinion > -48 || game.army_count(a) == 0 || game.army_count(b) == 0 {
        return false;
    }
    let expansionist = game.factions[a].personality == AiPersonality::Expansionist
        || game.factions[b].personality == AiPersonality::Expansionist;
    let border_pressure = shares_border(game, a, b);
    let army_power = |owner: usize| {
        game.armies.iter()
            .filter(|army| army.owner == owner)
            .map(|army| army.combat_power())
            .sum::<f32>()
    };
    let power_a = army_power(a);
    let power_b = army_power(b);
    let prepared = power_a.max(power_b) >= power_a.min(power_b) * 0.75;
    border_pressure && prepared && (expansionist || opinion <= -70)
}

fn shares_border(game: &GameState, a: usize, b: usize) -> bool {
    for coord in game.world.coords() {
        if game.world.get(coord).is_some_and(|tile| tile.owner == Some(a))
            && game.world.neighbors(coord).iter().any(|neighbor| {
                game.world.get(*neighbor).is_some_and(|tile| tile.owner == Some(b))
            })
        {
            return true;
        }
    }
    false
}

/// Finds the first legal land step from an army's current position to a selected province.
/// Neutral and friendly land is traversable; hostile territory is only traversable during war.
/// Finds a least-cost land route and returns its first step.
///
/// Roads reduce movement cost, while difficult terrain and unbridged rivers increase it.
/// Enemy provinces can only be crossed when the owner is at war with that realm.
pub fn next_step_toward_destination(
    game: &GameState,
    owner: usize,
    start: Coord,
    destination: Coord,
) -> Option<Coord> {
    if start == destination || game.world.get(destination).is_none() {
        return None;
    }

    let start_index = game.world.index(start)?;
    let destination_index = game.world.index(destination)?;
    let width = game.world.width as usize;
    let mut costs = vec![u32::MAX; game.world.tiles.len()];
    let mut first_steps: Vec<Option<Coord>> = vec![None; game.world.tiles.len()];
    let mut queue = BinaryHeap::new();

    costs[start_index] = 0;
    queue.push(Reverse((0_u32, start_index)));

    while let Some(Reverse((current_cost, current_index))) = queue.pop() {
        if current_cost != costs[current_index] {
            continue;
        }
        if current_index == destination_index {
            return first_steps[destination_index];
        }

        let current = Coord::new(
            (current_index % width) as i32,
            (current_index / width) as i32,
        );
        for next in game.world.neighbors(current) {
            let Some(step_cost) = province_movement_cost(game, owner, next) else {
                continue;
            };
            let Some(next_index) = game.world.index(next) else {
                continue;
            };
            let next_cost = current_cost.saturating_add(step_cost);
            if next_cost >= costs[next_index] {
                continue;
            }

            costs[next_index] = next_cost;
            first_steps[next_index] = if current_index == start_index {
                Some(next)
            } else {
                first_steps[current_index]
            };
            queue.push(Reverse((next_cost, next_index)));
        }
    }
    None
}

fn province_movement_cost(game: &GameState, owner: usize, coord: Coord) -> Option<u32> {
    let tile = game.world.get(coord)?;
    if !tile.terrain.is_land() {
        return None;
    }
    if tile.owner.is_some_and(|other| {
        other != owner && game.treaty_between(owner, other) != Treaty::War
    }) {
        return None;
    }

    let terrain_cost = match tile.terrain {
        super::world::Terrain::Ocean => return None,
        super::world::Terrain::Coast => 1,
        super::world::Terrain::Grassland | super::world::Terrain::Plains => 0,
        super::world::Terrain::Forest | super::world::Terrain::Desert => 2,
        super::world::Terrain::Tundra => 3,
        super::world::Terrain::Mountain => 5,
    };
    let has_road = tile.buildings.contains(&Building::Road);
    let river_cost = if tile.river && !has_road { 4 } else { 0 };
    let raw_cost: u32 = 10 + terrain_cost + river_cost;
    Some(if has_road { raw_cost.saturating_sub(7).max(2) } else { raw_cost })
}

fn issue_war_orders(game: &mut GameState) {
    let orders: Vec<(u64, Coord)> = game.armies.iter()
        .filter(|army| army.owner != PLAYER_ID)
        .filter_map(|army| next_step_toward_enemy(game, army).map(|coord| (army.id, coord)))
        .collect();

    for (army_id, destination) in orders {
        game.move_army(army_id, destination);
    }
}

fn next_step_toward_enemy(game: &GameState, army: &Army) -> Option<Coord> {
    let start = army.position;
    let mut queue = VecDeque::from([start]);
    let mut visited = HashSet::from([start]);
    let mut first_step: HashMap<Coord, Coord> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        for next in game.world.neighbors(current) {
            if !visited.insert(next) { continue; }
            let Some(tile) = game.world.get(next) else { continue; };
            if !tile.terrain.is_land() { continue; }

            if let Some(owner) = tile.owner.filter(|owner| *owner != army.owner) {
                if game.treaty_between(army.owner, owner) == Treaty::War {
                    return if current == start { Some(next) } else { first_step.get(&current).copied() };
                }
                continue;
            }

            let step = if current == start { next } else { *first_step.get(&current)? };
            first_step.insert(next, step);
            queue.push_back(next);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roads_reduce_army_path_cost_on_land_provinces() {
        let mut game = GameState::new(2030);
        let coord = game.world.coords().into_iter().find(|coord| {
            game.world.get(*coord).is_some_and(|tile| {
                tile.terrain.is_land()
                    && tile.owner.is_none_or(|owner| owner == PLAYER_ID)
                    && !tile.buildings.contains(&Building::Road)
            })
        }).expect("generated map contains land");
        let without_road = province_movement_cost(&game, PLAYER_ID, coord).unwrap();
        game.world.get_mut(coord).unwrap().buildings.push(Building::Road);
        let with_road = province_movement_cost(&game, PLAYER_ID, coord).unwrap();
        assert!(with_road < without_road);
    }

    #[test]
    fn player_destination_orders_find_a_legal_first_step() {
        let game = GameState::new(2026);
        let start = game.factions[PLAYER_ID].capital;
        let target = game.world.coords().into_iter().find(|coord| {
            start.manhattan(*coord) > 1
                && game.world.get(*coord).is_some_and(|tile| {
                    tile.terrain.is_land()
                        && tile.owner.is_none()
                        && next_step_toward_destination(&game, PLAYER_ID, start, *coord).is_some()
                })
        }).expect("generated map should contain reachable neutral land beyond the capital border");
        let step = next_step_toward_destination(&game, PLAYER_ID, start, target).unwrap();
        assert_eq!(step.manhattan(start), 1);
        assert!(game.world.get(step).unwrap().terrain.is_land());
    }

    #[test]
    fn rival_realms_expand_and_develop_without_corrupting_the_save_state() {
        let mut game = GameState::new(2026);
        for _ in 0..20 { game.advance_turn(); }
        assert!(game.validate().is_ok());
        assert!(game.factions.iter().skip(1).any(|faction| game.owned_count(faction.id) > 4));
    }
}
