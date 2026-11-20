use serde::{Deserialize, Serialize};

use super::economy::Building;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub const fn new(x: i32, y: i32) -> Self { Self { x, y } }
    pub fn manhattan(self, other: Self) -> i32 { (self.x - other.x).abs() + (self.y - other.y).abs() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Ocean,
    Coast,
    Grassland,
    Plains,
    Forest,
    Desert,
    Mountain,
    Tundra,
}

impl Terrain {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ocean => "Deep ocean",
            Self::Coast => "Coastal land",
            Self::Grassland => "Grassland",
            Self::Plains => "Plains",
            Self::Forest => "Forest",
            Self::Desert => "Desert",
            Self::Mountain => "Mountains",
            Self::Tundra => "Tundra",
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Ocean => [0.035, 0.12, 0.22],
            Self::Coast => [0.17, 0.30, 0.31],
            Self::Grassland => [0.29, 0.46, 0.28],
            Self::Plains => [0.52, 0.51, 0.32],
            Self::Forest => [0.12, 0.31, 0.23],
            Self::Desert => [0.66, 0.46, 0.27],
            Self::Mountain => [0.43, 0.46, 0.49],
            Self::Tundra => [0.54, 0.61, 0.62],
        }
    }

    pub fn is_land(self) -> bool { self != Self::Ocean }

    pub fn expansion_value(self) -> f32 {
        match self {
            Self::Ocean => -100.0,
            Self::Coast => 2.7,
            Self::Grassland => 3.0,
            Self::Plains => 2.8,
            Self::Forest => 3.5,
            Self::Desert => 1.5,
            Self::Mountain => 2.6,
            Self::Tundra => 1.9,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub owner: Option<usize>,
    pub city: Option<String>,
    pub population: u32,
    pub buildings: Vec<Building>,
    pub discovered: bool,
}

impl Tile {
    fn new(terrain: Terrain) -> Self {
        Self { terrain, owner: None, city: None, population: 0, buildings: Vec::new(), discovered: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    pub seed: u64,
    pub tiles: Vec<Tile>,
}

impl WorldMap {
    pub fn generate(width: i32, height: i32, seed: u64) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let broad = value_noise(x as f32 * 0.095, y as f32 * 0.095, seed);
                let detail = value_noise(x as f32 * 0.27 + 41.0, y as f32 * 0.27 - 13.0, seed + 17);
                let moisture = value_noise(x as f32 * 0.14 - 29.0, y as f32 * 0.14 + 53.0, seed + 31);
                let nx = (x as f32 / width.max(1) as f32 - 0.5) * 2.0;
                let ny = (y as f32 / height.max(1) as f32 - 0.5) * 2.0;
                let continental_bias = 0.14 * (1.0 - (nx * nx * 0.62 + ny * ny * 0.82).sqrt());
                let elevation = broad * 0.66 + detail * 0.24 + continental_bias;

                let latitude = (ny.abs() * 0.75 + 1.0 - moisture) * 0.5;
                let terrain = if elevation < 0.39 {
                    Terrain::Ocean
                } else if elevation > 0.78 {
                    Terrain::Mountain
                } else if latitude > 0.87 {
                    Terrain::Tundra
                } else if moisture < 0.23 && elevation < 0.70 {
                    Terrain::Desert
                } else if moisture > 0.67 {
                    Terrain::Forest
                } else if detail > 0.61 {
                    Terrain::Plains
                } else {
                    Terrain::Grassland
                };
                tiles.push(Tile::new(terrain));
            }
        }

        let mut map = Self { width, height, seed, tiles };
        let mut coastline = Vec::new();
        for coord in map.coords() {
            let tile = map.get(coord).expect("coordinate generated inside map");
            if tile.terrain == Terrain::Ocean && map.neighbors(coord).iter().any(|n| map.get(*n).is_some_and(|t| t.terrain.is_land())) {
                coastline.push(coord);
            }
        }
        for coord in coastline {
            if let Some(tile) = map.get_mut(coord) { tile.terrain = Terrain::Coast; }
        }
        map
    }

    pub fn index(&self, coord: Coord) -> Option<usize> {
        if coord.x < 0 || coord.y < 0 || coord.x >= self.width || coord.y >= self.height { None }
        else { Some((coord.y * self.width + coord.x) as usize) }
    }

    pub fn get(&self, coord: Coord) -> Option<&Tile> {
        self.index(coord).and_then(|index| self.tiles.get(index))
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut Tile> {
        self.index(coord).and_then(|index| self.tiles.get_mut(index))
    }

    pub fn coords(&self) -> Vec<Coord> {
        (0..self.height).flat_map(|y| (0..self.width).map(move |x| Coord::new(x, y))).collect()
    }

    pub fn neighbors(&self, coord: Coord) -> Vec<Coord> {
        [Coord::new(1, 0), Coord::new(-1, 0), Coord::new(0, 1), Coord::new(0, -1)]
            .into_iter()
            .map(|d| Coord::new(coord.x + d.x, coord.y + d.y))
            .filter(|p| self.index(*p).is_some())
            .collect()
    }

    pub fn nearest_land(&self, target: Coord, existing: &[Coord]) -> Coord {
        let mut best = target;
        let mut best_score = f32::MAX;
        for coord in self.coords() {
            let Some(tile) = self.get(coord) else { continue; };
            if !tile.terrain.is_land() { continue; }
            let mut score = coord.manhattan(target) as f32;
            for other in existing {
                let distance = coord.manhattan(*other) as f32;
                if distance < 8.0 { score += (8.0 - distance) * 6.0; }
            }
            if score < best_score { best_score = score; best = coord; }
        }
        best
    }
}

fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let a = hash01(x0, y0, seed);
    let b = hash01(x0 + 1, y0, seed);
    let c = hash01(x0, y0 + 1, seed);
    let d = hash01(x0 + 1, y0 + 1, seed);
    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

fn hash01(x: i32, y: i32, seed: u64) -> f32 {
    let mut value = (x as u64).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ seed;
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    ((value >> 40) as f32) / ((1u32 << 24) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_map_has_expected_size_and_multiple_terrain_types() {
        let map = WorldMap::generate(44, 28, 42);
        assert_eq!(map.tiles.len(), 44 * 28);
        assert!(map.tiles.iter().any(|t| t.terrain == Terrain::Ocean));
        assert!(map.tiles.iter().any(|t| t.terrain.is_land()));
        let distinct = map.tiles.iter().map(|t| t.terrain).collect::<std::collections::HashSet<_>>();
        assert!(distinct.len() >= 4);
    }

    #[test]
    fn map_generation_is_deterministic() {
        let a = WorldMap::generate(30, 20, 123);
        let b = WorldMap::generate(30, 20, 123);
        assert_eq!(a.tiles.iter().map(|t| t.terrain).collect::<Vec<_>>(), b.tiles.iter().map(|t| t.terrain).collect::<Vec<_>>());
    }

    #[test]
    fn neighbors_never_leave_map() {
        let map = WorldMap::generate(8, 8, 1);
        assert_eq!(map.neighbors(Coord::new(0, 0)).len(), 2);
        assert!(map.get(Coord::new(-1, 0)).is_none());
    }
}
