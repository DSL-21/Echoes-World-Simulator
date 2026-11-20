use std::collections::{HashMap, HashSet, VecDeque};

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
                Treaty::None if game.turn % 12 == 0 && opinion >= 18 => {
                    set_symmetric(&mut game.relations, a, b, Treaty::TradeAgreement, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("{first} and {second} sign a trade agreement."));
                }
                Treaty::None if game.turn % 12 == 0 && opinion <= -48
                    && game.army_count(a) > 0 && game.army_count(b) > 0 => {
                    set_symmetric(&mut game.relations, a, b, Treaty::War, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("WAR — border tensions erupt into open conflict between {first} and {second}."));
                }
                Treaty::TradeAgreement if game.turn % 12 == 0 && opinion >= 48 => {
                    set_symmetric(&mut game.relations, a, b, Treaty::Alliance, game.turn);
                    let first = game.factions[a].name.clone();
                    let second = game.factions[b].name.clone();
                    game.add_log(format!("{first} and {second} formalize their friendship in an alliance."));
                }
                Treaty::Truce if game.turn.saturating_sub(changed_turn) >= 24 && opinion >= 5 => {
                    set_symmetric(&mut game.relations, a, b, Treaty::None, game.turn);
                }
                _ => {}
            }
        }
    }
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
    fn rival_realms_expand_and_develop_without_corrupting_the_save_state() {
        let mut game = GameState::new(2026);
        for _ in 0..20 { game.advance_turn(); }
        assert!(game.validate().is_ok());
        assert!(game.factions.iter().skip(1).any(|faction| game.owned_count(faction.id) > 4));
    }
}
