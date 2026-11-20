use super::{GameState, PLAYER_ID};

#[test]
fn multi_year_campaigns_preserve_invariants_across_world_seeds() {
    for seed in [1_u64, 42, 2026, 20_261_120] {
        let mut game = GameState::new(seed);

        for _ in 0..100 {
            if game.is_ended() {
                break;
            }

            game.advance_turn();
            game.validate().unwrap_or_else(|error| {
                panic!("campaign seed {seed} became invalid on turn {}: {error}", game.turn)
            });

            for faction in &game.factions {
                assert!(
                    faction.resources.is_finite_non_negative(),
                    "seed {seed}, turn {}: {} has an invalid stockpile",
                    game.turn,
                    faction.name
                );
                assert!((0.0..=100.0).contains(&faction.stability));
                assert!((0.0..=100.0).contains(&faction.unrest));
                assert!((0.0..=100.0).contains(&faction.legitimacy));
            }

            assert!(game.market.is_valid());
            assert!(game.factions[PLAYER_ID].tax_rate.is_finite());
        }
    }
}

#[test]
fn campaigns_with_the_same_seed_replay_identically() {
    let seed = 20_261_120;
    let mut first = GameState::new(seed);
    let mut second = GameState::new(seed);

    for _ in 0..60 {
        if first.is_ended() || second.is_ended() {
            break;
        }
        first.advance_turn();
        second.advance_turn();
    }

    assert_eq!(first.turn, second.turn);
    assert_eq!(first.year, second.year);
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
}
