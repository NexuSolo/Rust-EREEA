use noise::{NoiseFn, Perlin};
use rand::Rng;

#[derive(Clone, PartialEq, Debug)]
pub enum TypeCase {
    Vide,
    Base,
    Mur,
    Energie,
    Minerais,
    Science,
    Explorateur,
    Collecteur,
}

pub fn generer_carte(width: usize, height: usize, seed: u32) -> Vec<Vec<TypeCase>> {
    let perlin = Perlin::new(seed);
    let mut rng = rand::thread_rng();
    let mut carte = vec![vec![TypeCase::Vide; width]; height];

    let base_x = rng.gen_range(0..width);
    let base_y = rng.gen_range(0..height);
    carte[base_y][base_x] = TypeCase::Base;
    print!("Base en ({}, {})\n", base_x, base_y);
    for y in 0..height {
        for x in 0..width {
            if carte[y][x] == TypeCase::Vide {
                let noise_value = perlin.get([x as f64 / 9.25, y as f64 / 8.0]);
                carte[y][x] = match noise_value {
                    v if v < -0.55 => TypeCase::Mur,
                    v if v < -0.53 => TypeCase::Minerais,
                    _ => TypeCase::Vide,
                };
            }
        }
    }

    carte
}
