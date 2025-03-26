mod base;
mod generation;
mod pathfinding;
mod robot;
mod ui;
use crossterm::terminal;
use std::env;

use crate::base::Base;
use crate::generation::{generate_map, TypeCase};
use crate::ui::run_ui;
use std::sync::Arc;

fn main() {
    // Map settings
    let (width, height) = terminal::size().unwrap();
    let width = (width / 2) as usize;
    let height = height as usize;

    // Default value for the seed
    const SEED_DEFAULT: u32 = 0;

    // Get the seed from the command line arguments
    let args: Vec<String> = env::args().collect();
    let seed = args
        .iter()
        .find(|arg| arg.starts_with("seed="))
        .and_then(|arg| {
            let parts: Vec<&str> = arg.split('=').collect();
            if parts.len() > 1 {
                match parts[1].parse::<u32>() {
                    Ok(seed_value) => Some(seed_value),
                    Err(_) => {
                        println!("The seed must be a positive integer. Using the default seed.");
                        None
                    }
                }
            } else {
                None
            }
        })
        .unwrap_or(SEED_DEFAULT);

    println!("Map generation with the seed: {}", seed);

    let (map, known_map, (base_x, base_y)) = generate_map(width, height, seed);

    // Create the base and start its thread
    let base = Base::new(
        width,
        height,
        base_x,
        base_y,
        map.clone(),
        known_map.clone(),
    );

    Base::start_base_thread(Arc::clone(&base), width, height);

    // Keep the program alive
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
