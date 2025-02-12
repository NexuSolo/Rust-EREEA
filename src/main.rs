mod base;
mod generation;
mod robot;
mod ui;
use crossterm::terminal;

use crate::base::Base;
use crate::generation::{generer_carte, TypeCase};
use crate::ui::run_ui;
use std::sync::Arc;

fn main() {
    // Paramètres de la carte
    let (width, height) = terminal::size().unwrap();
    let width = (width / 2) as usize;
    let height = height as usize;
    let seed = 577679768;

    let (carte, carte_connue) = generer_carte(width, height, seed);

    let mut base_x = 0;
    let mut base_y = 0;
    for y in 0..height {
        for x in 0..width {
            if carte[y][x] == TypeCase::Base {
                base_x = x;
                base_y = y;
                break;
            }
        }
    }

    // Créer la base et démarrer son thread
    let base = Base::new(width, height, base_x, base_y, carte, carte_connue.clone());
    let robots = if let Ok(base_guard) = base.lock() {
        Arc::clone(&base_guard.robots_deployes)
    } else {
        panic!("Impossible d'accéder à la base")
    };
    let carte_connue = if let Ok(base_guard) = base.lock() {
        Arc::clone(&base_guard.carte_connue)
    } else {
        panic!("Impossible d'accéder à la base")
    };

    Base::demarrer_thread_base(Arc::clone(&base), width, height);

    // Garder le programme en vie
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Ok(carte) = carte_connue.lock() {
            run_ui(
                &carte,
                "Ressources: 100 énergie, 50 minerais, 25 science",
                &robots,
            )
            .unwrap();
        }
    }
}
