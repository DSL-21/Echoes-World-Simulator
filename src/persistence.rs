use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::sim::GameState;

const SAVE_FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct SaveEnvelope {
    format_version: u32,
    game: GameState,
}

pub fn save_game(game: &GameState, path: impl AsRef<Path>) -> Result<(), String> {
    game.validate()?;
    let envelope = SaveEnvelope { format_version: SAVE_FORMAT_VERSION, game: game.clone() };
    let json = serde_json::to_vec_pretty(&envelope).map_err(|error| format!("Could not serialize campaign: {error}"))?;
    fs::write(path.as_ref(), json).map_err(|error| format!("Could not write save file: {error}"))
}

pub fn load_game(path: impl AsRef<Path>) -> Result<GameState, String> {
    let bytes = fs::read(path.as_ref()).map_err(|error| format!("Could not read save file: {error}"))?;
    let envelope: SaveEnvelope = serde_json::from_slice(&bytes).map_err(|error| format!("Save file is invalid or corrupted: {error}"))?;
    if envelope.format_version != SAVE_FORMAT_VERSION {
        return Err(format!("Unsupported save version {}. This build supports version {}.", envelope.format_version, SAVE_FORMAT_VERSION));
    }
    envelope.game.validate()?;
    Ok(envelope.game)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_roundtrip_preserves_campaign_state() {
        let game = GameState::new(2026);
        let path = std::env::temp_dir().join(format!("echoes-save-test-{}.json", std::process::id()));
        save_game(&game, &path).unwrap();
        let loaded = load_game(&path).unwrap();
        assert_eq!(loaded.year, game.year);
        assert_eq!(loaded.turn, game.turn);
        assert_eq!(loaded.factions.len(), game.factions.len());
        assert_eq!(loaded.world.tiles.len(), game.world.tiles.len());
        let _ = fs::remove_file(path);
    }
}
