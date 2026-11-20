use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Treaty {
    #[default]
    None,
    TradeAgreement,
    Alliance,
    War,
    Truce,
}

impl Treaty {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No treaty",
            Self::TradeAgreement => "Trade agreement",
            Self::Alliance => "Alliance",
            Self::War => "At war",
            Self::Truce => "Truce",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    pub opinion: i16,
    pub treaty: Treaty,
    pub changed_turn: u32,
}

impl Default for Relation {
    fn default() -> Self { Self { opinion: 0, treaty: Treaty::None, changed_turn: 0 } }
}

pub fn new_matrix(faction_count: usize) -> Vec<Vec<Relation>> {
    let mut matrix = vec![vec![Relation::default(); faction_count]; faction_count];
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = Relation { opinion: 100, treaty: Treaty::Alliance, changed_turn: 1 };
    }
    matrix
}

pub fn set_symmetric(matrix: &mut [Vec<Relation>], a: usize, b: usize, treaty: Treaty, turn: u32) {
    if a >= matrix.len() || b >= matrix.len() || a == b { return; }
    matrix[a][b].treaty = treaty;
    matrix[a][b].changed_turn = turn;
    matrix[b][a].treaty = treaty;
    matrix[b][a].changed_turn = turn;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treaties_are_symmetric() {
        let mut matrix = new_matrix(3);
        set_symmetric(&mut matrix, 0, 2, Treaty::War, 7);
        assert_eq!(matrix[0][2].treaty, Treaty::War);
        assert_eq!(matrix[2][0].treaty, Treaty::War);
        assert_eq!(matrix[0][2].changed_turn, 7);
    }
}
