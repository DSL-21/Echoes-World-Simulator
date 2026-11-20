use std::{cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet, VecDeque}};

use super::{
    diplomacy::{set_symmetric, Treaty},
    economy::{Building, Resources},
    military::{Army, NavalFleet, UnitClass, recruitment_cost},
    AiPersonality, GameState, PLAYER_ID,
    world::{Coord, Terrain},
};

/// Advances the parts of the world that are controlled by rival realms.
pub fn process_turn(game: &mut GameState) {
    develop_cities(game);
    develop_ports(game);
    develop_provinces(game);
    simulate_diplomacy(game);
    issue_war_orders(game);
    process_naval_warfare(game);
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

fn develop_ports(game: &mut GameState) {
    for faction_id in 1..game.factions.len() {
        let already_has_port = game.world.coords().into_iter().any(|coord| {
            game.world.get(coord).is_some_and(|tile| {
                tile.owner == Some(faction_id)
                    && tile.city.is_some()
                    && tile.buildings.contains(&Building::Port)
            })
        });
        if already_has_port {
            continue;
        }

        let candidate = game.world.coords().into_iter().filter_map(|coord| {
            let tile = game.world.get(coord)?;
            if tile.owner != Some(faction_id)
                || tile.city.is_none()
                || !Building::Port.allowed_on(tile.terrain)
            {
                return None;
            }
            Some((coord, tile.population))
        }).max_by_key(|(_, population)| *population);

        let Some((coord, _)) = candidate else {
            continue;
        };
        if !game.factions[faction_id].resources.pay(&Building::Port.cost()) {
            continue;
        }
        if let Some(tile) = game.world.get_mut(coord) {
            tile.buildings.push(Building::Port);
        }
        let realm = game.factions[faction_id].name.clone();
        let city = game.world.get(coord).and_then(|tile| tile.city.clone()).unwrap_or_default();
        game.add_log(format!("{realm} commissions a new port at {city}."));
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

/// Plans and executes amphibious invasions for AI realms at war.
/// Fleets only sail through navigable tiles and land on a coast adjacent to hostile land.
fn process_naval_warfare(game: &mut GameState) {
    for owner in 1..game.factions.len() {
        let enemies: Vec<usize> = (0..game.factions.len())
            .filter(|other| *other != owner && game.treaty_between(owner, *other) == Treaty::War)
            .collect();
        if enemies.is_empty() {
            continue;
        }

        let ports: Vec<Coord> = game.world.coords().into_iter().filter(|coord| {
            game.world.get(*coord).is_some_and(|tile| {
                tile.owner == Some(owner)
                    && tile.city.is_some()
                    && tile.terrain == Terrain::Coast
                    && tile.buildings.contains(&Building::Port)
            })
        }).collect();
        if ports.is_empty() {
            continue;
        }

        // Continue an active expedition before considering a new embarkation.
        // This prevents the AI from commissioning duplicate fleets while troops are at sea.
        let active_fleet = game.naval_fleets.iter().find(|fleet| {
            fleet.owner == owner
                && game.armies.iter().any(|army| army.embarked_in == Some(fleet.id))
        }).map(|fleet| fleet.id);

        let (fleet_id, should_embark) = if let Some(id) = active_fleet {
            (id, false)
        } else {
            // Recruit at a port when the realm has no army there; keep the force cap low.
            let port_with_army = ports.iter().copied().find(|port| {
                game.armies.iter().any(|army| {
                    army.owner == owner && army.position == *port && army.embarked_in.is_none()
                })
            });
            let embark_port = if let Some(port) = port_with_army {
                port
            } else {
                if game.army_count(owner) >= 3 || game.turn % 4 != owner as u32 % 4 {
                    continue;
                }
                let port = ports[0];
                let (gold, iron, timber) = recruitment_cost(UnitClass::Infantry);
                let cost = Resources { gold, iron, timber, ..Default::default() };
                if !game.factions[owner].resources.pay(&cost) {
                    continue;
                }
                let id = game.next_army_id;
                game.next_army_id += 1;
                game.armies.push(Army::new(id, owner, port, UnitClass::Infantry));
                port
            };

            let existing = game.naval_fleets.iter().find(|fleet| {
                fleet.owner == owner && fleet.position == embark_port
            }).map(|fleet| fleet.id);
            let id = if let Some(id) = existing {
                id
            } else {
                let cost = Resources { gold: 45.0, timber: 30.0, iron: 8.0, ..Default::default() };
                if !game.factions[owner].resources.pay(&cost) {
                    continue;
                }
                let id = game.next_naval_fleet_id;
                game.next_naval_fleet_id += 1;
                game.naval_fleets.push(NavalFleet::new(id, owner, embark_port));
                let realm = game.factions[owner].name.clone();
                game.add_log(format!("{realm} commissions a transport fleet for an amphibious campaign."));
                id
            };
            (id, true)
        };

        if should_embark {
            let capacity = game.naval_fleets.iter().find(|fleet| fleet.id == fleet_id)
                .map(|fleet| fleet.capacity).unwrap_or(0);
            let aboard = game.armies.iter().filter(|army| army.embarked_in == Some(fleet_id)).count();
            if aboard < capacity {
                if let Some(army) = game.armies.iter_mut().find(|army| {
                    army.owner == owner
                        && army.position == game.naval_fleets.iter().find(|fleet| fleet.id == fleet_id).map(|fleet| fleet.position).unwrap_or_default()
                        && army.embarked_in.is_none()
                }) {
                    army.embarked_in = Some(fleet_id);
                    let army_name = army.name.clone();
                    game.add_log(format!("{army_name} embarks for an amphibious invasion."));
                }
            }
        }

        let fleet_position = match game.naval_fleets.iter().find(|fleet| fleet.id == fleet_id) {
            Some(fleet) => fleet.position,
            None => continue,
        };
        let Some((landing, first_step)) = find_ai_landing_route(game, owner, &enemies, fleet_position) else {
            continue;
        };

        if fleet_position == landing {
            let mut landed = 0;
            for army in &mut game.armies {
                if army.embarked_in == Some(fleet_id) {
                    army.embarked_in = None;
                    army.position = landing;
                    landed += 1;
                }
            }
            if landed > 0 {
                let realm = game.factions[owner].name.clone();
                game.add_log(format!("{realm} lands {landed} army unit(s) on a hostile coastline."));
            }
        } else if let Some(step) = first_step {
            if let Some(fleet) = game.naval_fleets.iter_mut().find(|fleet| fleet.id == fleet_id) {
                fleet.position = step;
                fleet.supply = (fleet.supply - 1.5).max(0.0);
            }
            for army in &mut game.armies {
                if army.embarked_in == Some(fleet_id) {
                    army.position = step;
                }
            }
        }
    }
}

/// Returns a reachable friendly/neutral coast adjacent to a realm at war and
/// the first sea step toward it. Hostile provinces themselves are never landing tiles.
fn find_ai_landing_route(
    game: &GameState,
    owner: usize,
    enemies: &[usize],
    start: Coord,
) -> Option<(Coord, Option<Coord>)> {
    let mut queue = VecDeque::from([(start, None::<Coord>)]);
    let mut visited = HashSet::from([start]);
    while let Some((current, first_step)) = queue.pop_front() {
        let tile = game.world.get(current)?;
        let safe_landing = tile.terrain == Terrain::Coast
            && tile.owner.is_none_or(|other| other == owner)
            && game.world.neighbors(current).iter().any(|neighbor| {
                game.world.get(*neighbor).is_some_and(|adjacent| {
                    adjacent.terrain.is_land()
                        && adjacent.owner.is_some_and(|other| enemies.contains(&other))
                })
            });
        if safe_landing {
            return Some((current, first_step));
        }
        for next in game.world.neighbors(current) {
            if !visited.insert(next) {
                continue;
            }
            if game.world.get(next).is_some_and(|next_tile| {
                matches!(next_tile.terrain, Terrain::Ocean | Terrain::Coast)
                    && next_tile.owner.is_none_or(|other| other == owner)
            }) {
                queue.push_back((next, first_step.or(Some(next))));
            }
        }
    }
    None
}

fn issue_war_orders(game: &mut GameState) {
    let orders: Vec<(u64, Coord)> = game.armies.iter()
        .filter(|army| army.owner != PLAYER_ID && army.embarked_in.is_none())
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
    fn rival_realms_build_ports_in_coastal_cities() {
        let mut game = GameState::new(2039);
        let capital = game.factions[1].capital;
        game.world.get_mut(capital).unwrap().terrain = super::super::world::Terrain::Coast;

        develop_ports(&mut game);

        assert!(game.world.get(capital).unwrap().buildings.contains(&Building::Port));
        assert!(game.validate().is_ok());
    }

    #[test]
    fn merchant_personality_makes_trade_proposals_more_likely() {
        let mut game = GameState::new(2040);
        game.factions[1].personality = AiPersonality::Merchant;
        game.factions[2].personality = AiPersonality::Balanced;
        assert!(!trade_proposal_accepted(&game, 1, 2, 17));
        assert!(trade_proposal_accepted(&game, 1, 2, 30));
        game.factions[1].personality = AiPersonality::Expansionist;
        assert!(!trade_proposal_accepted(&game, 1, 2, 30));
    }

    #[test]
    fn alliance_proposals_require_trust_or_a_shared_enemy() {
        let mut game = GameState::new(2041);
        assert!(!alliance_proposal_accepted(&game, 1, 2, 55));
        assert!(alliance_proposal_accepted(&game, 1, 2, 68));
        set_symmetric(&mut game.relations, 1, 3, Treaty::War, game.turn);
        set_symmetric(&mut game.relations, 2, 3, Treaty::War, game.turn);
        assert!(alliance_proposal_accepted(&game, 1, 2, 48));
    }

    #[test]
    fn war_proposals_need_hostility_border_pressure_and_armies() {
        let mut game = GameState::new(2042);
        assert!(!war_proposal_accepted(&game, 1, 2, -90));
        game.factions[1].personality = AiPersonality::Expansionist;
        game.factions[2].personality = AiPersonality::Defender;
        let first = game.factions[1].capital;
        let second = game.world.neighbors(first).into_iter().find(|coord| {
            game.world.get(*coord).is_some_and(|tile| tile.terrain.is_land())
        }).unwrap();
        game.world.get_mut(second).unwrap().owner = Some(2);
        game.armies.push(Army::new(1, 1, first, super::super::military::UnitClass::Infantry));
        game.armies.push(Army::new(2, 2, second, super::super::military::UnitClass::Infantry));
        assert!(war_proposal_accepted(&game, 1, 2, -55));
    }

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
    fn ai_commissions_fleet_and_lands_troops_across_a_sea_lane() {
        let mut game = GameState::new(2043);
        let port = game.factions[1].capital;
        let sea = Coord::new(port.x - 1, port.y);
        let landing = Coord::new(port.x - 2, port.y);
        let enemy_land = Coord::new(port.x - 3, port.y);
        assert!(game.world.get(enemy_land).is_some());

        {
            let tile = game.world.get_mut(port).unwrap();
            tile.terrain = Terrain::Coast;
            tile.buildings.push(Building::Port);
        }
        game.world.get_mut(sea).unwrap().terrain = Terrain::Ocean;
        {
            let tile = game.world.get_mut(landing).unwrap();
            tile.terrain = Terrain::Coast;
            tile.owner = None;
        }
        {
            let tile = game.world.get_mut(enemy_land).unwrap();
            tile.terrain = Terrain::Grassland;
            tile.owner = Some(PLAYER_ID);
        }
        game.armies.push(Army::new(1, 1, port, UnitClass::Infantry));
        game.next_army_id = 2;
        set_symmetric(&mut game.relations, 1, PLAYER_ID, Treaty::War, game.turn);

        for _ in 0..4 {
            process_naval_warfare(&mut game);
        }

        assert!(game.naval_fleets.iter().any(|fleet| fleet.owner == 1));
        assert!(game.armies.iter().any(|army| {
            army.owner == 1 && army.position == landing && army.embarked_in.is_none()
        }));
        assert!(game.validate().is_ok());
    }

    #[test]
    fn rival_realms_expand_and_develop_without_corrupting_the_save_state() {
        let mut game = GameState::new(2026);
        for _ in 0..20 { game.advance_turn(); }
        assert!(game.validate().is_ok());
        assert!(game.factions.iter().skip(1).any(|faction| game.owned_count(faction.id) > 4));
    }
}
