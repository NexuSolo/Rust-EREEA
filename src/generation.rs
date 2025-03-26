use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::{Arc, Mutex};

use crate::config::Config;

#[derive(Clone, PartialEq, Debug)]
pub enum TypeCase {
    Void,
    Base,
    Wall,
    Energy,
    Ore,
    Science,
    Explorer,
    Collector,
    Unknown,
}

pub fn generate_map(
    width: usize,
    height: usize,
    seed: u32,
    config: &Config,
) -> (
    Arc<Mutex<Vec<Vec<TypeCase>>>>,
    Arc<Mutex<Vec<Vec<TypeCase>>>>,
    (usize, usize),
) {
    let perlin = Perlin::new(seed);
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut map = vec![vec![TypeCase::Void; width]; height];
    let mut known_map = vec![vec![TypeCase::Unknown; width]; height];

    // Generate the terrain
    for y in 0..height {
        for x in 0..width {
            let noise_value = perlin.get([x as f64 / 9.25, y as f64 / 8.0]);
            map[y][x] = match noise_value {
                v if v < -0.55 => TypeCase::Wall,
                v if v < -0.53 => TypeCase::Ore,
                _ => TypeCase::Void,
            };
        }
    }

    // Place the base
    let mut base_x;
    let mut base_y;
    loop {
        base_x = rng.random_range(0..width);
        base_y = rng.random_range(0..height);
        if map[base_y][base_x] == TypeCase::Void {
            map[base_y][base_x] = TypeCase::Base;
            break;
        }
    }

    // Reveal the area around the base in the known map
    for dy in -3..=3 {
        for dx in -3..=3 {
            let new_x = base_x as i32 + dx;
            let new_y = base_y as i32 + dy;
            if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                known_map[new_y as usize][new_x as usize] =
                    map[new_y as usize][new_x as usize].clone();
            }
        }
    }

    let map_size = width * height;
    let percentage_spawn = config.map.generation_rate as f64;

    // Add energy points
    let nb_energy = (map_size as f64 * percentage_spawn).round() as usize;
    for _ in 0..nb_energy {
        let mut x;
        let mut y;
        loop {
            x = rng.random_range(0..width);
            y = rng.random_range(0..height);
            if map[y][x] == TypeCase::Void {
                map[y][x] = TypeCase::Energy;
                break;
            }
        }
    }

    // Add science points
    let nb_science = (map_size as f64 * percentage_spawn).round() as usize;
    for _ in 0..nb_science {
        let mut x;
        let mut y;
        loop {
            x = rng.random_range(0..width);
            y = rng.random_range(0..height);
            if map[y][x] == TypeCase::Void {
                map[y][x] = TypeCase::Science;
                break;
            }
        }
    }

    (
        Arc::new(Mutex::new(map)),
        Arc::new(Mutex::new(known_map)),
        (base_x, base_y),
    )
}
