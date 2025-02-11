use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut carte = vec![vec![TypeCase::Vide; width]; height];

    // Génération du terrain
    for y in 0..height {
        for x in 0..width {
            let noise_value = perlin.get([x as f64 / 9.25, y as f64 / 8.0]);
            carte[y][x] = match noise_value {
                v if v < -0.55 => TypeCase::Mur,
                v if v < -0.53 => TypeCase::Minerais,
                _ => TypeCase::Vide,
            };
        }
    }

    // Placement de la base
    let mut base_x;
    let mut base_y;
    loop {
        base_x = rng.random_range(0..width);
        base_y = rng.random_range(0..height);
        if carte[base_y][base_x] == TypeCase::Vide {
            carte[base_y][base_x] = TypeCase::Base;
        }
    }

    carte[base_y][base_x] = TypeCase::Base;
    print!("Base en ({}, {})\n", base_x, base_y);

    // Ajout de points d'énergie
    let nb_energie = rng.random_range(10..=20);
    for _ in 0..nb_energie {
        let mut x;
        let mut y;
        loop {
            x = rng.random_range(0..width);
            y = rng.random_range(0..height);
            if carte[y][x] == TypeCase::Vide {
                carte[y][x] = TypeCase::Energie;
                break;
            }
        }
    }

    // Ajout de points de science
    let nb_science = rng.random_range(10..=20);
    for _ in 0..nb_science {
        let mut x;
        let mut y;
        loop {
            x = rng.random_range(0..width);
            y = rng.random_range(0..height);
            if carte[y][x] == TypeCase::Vide {
                carte[y][x] = TypeCase::Science;
                break;
            }
        }
    }

    carte
}
