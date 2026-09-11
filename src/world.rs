//! Procedural cave generation: cellular-automata cave shapes, smoothed,
//! then verified traversable with a flood fill. Deterministic from seed.

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall,
    Water,
    Hazard,
    Resource(ResourceKind),
    Extraction,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceKind {
    Food,
    Water,
    Medical,
    Material,
}

pub struct Layer {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    /// ambient temperature for this layer, fed into SurvivalState::tick
    pub ambient_temp: f32,
    pub seed: u64,
}

impl Layer {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            Tile::Wall
        } else {
            self.tiles[y as usize][x as usize]
        }
    }

    pub fn set(&mut self, x: usize, y: usize, t: Tile) {
        self.tiles[y][x] = t;
    }
}

/// Generate a traversable cave layer for the given seed. Regenerates with a
/// derived seed internally (bounded attempts) until the extraction point is
/// reachable from the start, so callers never see an unsolvable layer.
pub fn generate_layer(seed: u64, width: usize, height: usize, ambient_temp: f32) -> Layer {
    let mut attempt_seed = seed;
    for _ in 0..64 {
        let tiles = generate_raw(attempt_seed, width, height);
        if reachable(&tiles, 2, 2, width - 3, height - 3) {
            let mut tiles = tiles;
            tiles[height - 3][width - 3] = Tile::Extraction;
            scatter_resources(&mut tiles, attempt_seed, width, height);
            return Layer {
                width,
                height,
                tiles,
                ambient_temp,
                seed,
            };
        }
        attempt_seed = attempt_seed.wrapping_add(0x9E3779B97F4A7C15);
    }
    // extremely unlikely fallback: a guaranteed-open corridor layer
    let mut tiles = vec![vec![Tile::Wall; width]; height];
    let mid = height / 2;
    for x in 0..width {
        tiles[mid][x] = Tile::Empty;
    }
    tiles[mid][width - 3] = Tile::Extraction;
    Layer {
        width,
        height,
        tiles,
        ambient_temp,
        seed,
    }
}

fn generate_raw(seed: u64, width: usize, height: usize) -> Vec<Vec<Tile>> {
    let mut rng = StdRng::seed_from_u64(seed);
    // 1. random noise seed grid: true = wall
    let mut wall = vec![vec![false; width]; height];
    for y in 0..height {
        for x in 0..width {
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                wall[y][x] = true;
            } else {
                wall[y][x] = rng.gen_bool(0.45);
            }
        }
    }
    // 2. cellular automata smoothing passes -> organic cave shapes
    for _ in 0..4 {
        wall = smooth(&wall, width, height);
    }
    // keep the start area guaranteed open
    for y in 1..5.min(height - 1) {
        for x in 1..5.min(width - 1) {
            wall[y][x] = false;
        }
    }

    let mut tiles = vec![vec![Tile::Empty; width]; height];
    for y in 0..height {
        for x in 0..width {
            tiles[y][x] = if wall[y][x] { Tile::Wall } else { Tile::Empty };
        }
    }
    tiles
}

fn smooth(wall: &[Vec<bool>], width: usize, height: usize) -> Vec<Vec<bool>> {
    let mut out = vec![vec![false; width]; height];
    for y in 0..height {
        for x in 0..width {
            let mut wall_neighbors = 0;
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let is_wall = if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height
                    {
                        true // treat out-of-bounds as wall, keeps edges solid
                    } else {
                        wall[ny as usize][nx as usize]
                    };
                    if is_wall {
                        wall_neighbors += 1;
                    }
                }
            }
            out[y][x] = wall_neighbors >= 5;
        }
    }
    out
}

fn scatter_resources(tiles: &mut [Vec<Tile>], seed: u64, width: usize, height: usize) {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xABCDEF12345);
    for y in 0..height {
        for x in 0..width {
            if tiles[y][x] != Tile::Empty {
                continue;
            }
            let roll: f64 = rng.gen();
            tiles[y][x] = if roll < 0.03 {
                Tile::Resource(ResourceKind::Food)
            } else if roll < 0.055 {
                Tile::Resource(ResourceKind::Water)
            } else if roll < 0.07 {
                Tile::Resource(ResourceKind::Medical)
            } else if roll < 0.10 {
                Tile::Resource(ResourceKind::Material)
            } else if roll < 0.13 {
                Tile::Hazard
            } else if roll < 0.15 {
                Tile::Water
            } else {
                Tile::Empty
            };
        }
    }
}

fn reachable(tiles: &[Vec<Tile>], sx: usize, sy: usize, tx: usize, ty: usize) -> bool {
    let height = tiles.len();
    let width = tiles[0].len();
    let mut visited = vec![vec![false; width]; height];
    let mut q = VecDeque::new();
    if tiles[sy][sx] == Tile::Wall {
        return false;
    }
    q.push_back((sx, sy));
    visited[sy][sx] = true;
    while let Some((x, y)) = q.pop_front() {
        if (x, y) == (tx, ty) {
            return true;
        }
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];
        for (nx, ny) in neighbors {
            if nx < width && ny < height && !visited[ny][nx] && tiles[ny][nx] != Tile::Wall {
                visited[ny][nx] = true;
                q.push_back((nx, ny));
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic_for_a_given_seed() {
        let a = generate_layer(1234, 40, 24, 50.0);
        let b = generate_layer(1234, 40, 24, 50.0);
        for y in 0..a.height {
            for x in 0..a.width {
                assert_eq!(a.tiles[y][x], b.tiles[y][x], "mismatch at ({x},{y})");
            }
        }
    }

    #[test]
    fn different_seeds_usually_produce_different_layouts() {
        let a = generate_layer(1, 40, 24, 50.0);
        let b = generate_layer(2, 40, 24, 50.0);
        let mut differences = 0;
        for y in 0..a.height {
            for x in 0..a.width {
                if a.tiles[y][x] != b.tiles[y][x] {
                    differences += 1;
                }
            }
        }
        assert!(differences > 20, "seeds 1 and 2 produced near-identical layouts");
    }

    #[test]
    fn every_generated_layer_is_solvable_across_many_seeds() {
        for seed in 0..100u64 {
            let layer = generate_layer(seed, 40, 24, 50.0);
            assert!(
                reachable(&layer.tiles, 2, 2, layer.width - 3, layer.height - 3),
                "seed {seed} produced an unreachable extraction point"
            );
        }
    }

    #[test]
    fn extraction_tile_is_actually_placed() {
        let layer = generate_layer(77, 40, 24, 50.0);
        assert_eq!(layer.tiles[layer.height - 3][layer.width - 3], Tile::Extraction);
    }
}
