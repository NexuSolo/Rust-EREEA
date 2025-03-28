mod base;
mod config;
mod generation;
mod pathfinding;
mod robot;
mod ui;

use crossterm::terminal;

use crate::base::Base;
use crate::config::Config;
use crate::generation::{generate_map, TypeCase};
use crate::ui::run_ui;
use std::sync::Arc;

fn main() {
    // Charger la configuration
    let config = Config::load().expect("Impossible de charger la configuration");

    let (width, height) = terminal::size().unwrap();
    let width = (width / 2) as usize;
    let height = height as usize;

    // Utiliser la seed de la configuration
    let seed = config.map.seed;

    let (map, known_map, (base_x, base_y)) = generate_map(width, height, seed, &config);

    // Créer la base avec la configuration
    let base = Base::new(
        width,
        height,
        base_x,
        base_y,
        map.clone(),
        known_map.clone(),
        config,
    );

    Base::start_base_thread(Arc::clone(&base), width, height);

    // Garder le programme en vie
    loop {
        if let Ok(base_guard) = base.lock() {
            let energy = *base_guard.energy.lock().unwrap();
            let ore = *base_guard.ore.lock().unwrap();
            let science = *base_guard.science.lock().unwrap();
            let nb_robots = base_guard.deployed_robots.lock().unwrap().len();

            let mut nb_explorers = 0;
            let mut nb_collectors = 0;
            if let Ok(robots) = base_guard.deployed_robots.lock() {
                for robot in robots.iter() {
                    match robot.get_type() {
                        TypeCase::Explorer => nb_explorers += 1,
                        TypeCase::Collector => nb_collectors += 1,
                        _ => {}
                    }
                }
            }

            let resources = format!(
                "Resources: {} energy, {} ore, {} science | Robots: {} total ({} explorers, {} collectors)",
                energy, ore, science, nb_robots, nb_explorers, nb_collectors
            );

            run_ui(
                &base_guard.known_map,
                &resources,
                &base_guard.deployed_robots,
            )
            .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
