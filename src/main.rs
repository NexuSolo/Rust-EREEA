mod base;
mod generation;
mod pathfinding;
mod robot;
mod ui;
use crossterm::terminal;
use std::env;

use crate::base::Base;
use crate::generation::{generer_carte, TypeCase};
use crate::ui::run_ui;
use std::sync::Arc;

fn main() {
    // Paramètres de la carte
    let (width, height) = terminal::size().unwrap();
    let width = (width / 2) as usize;
    let height = height as usize;

    // Valeur par défaut pour la seed
    const SEED_DEFAULT: u32 = 0;

    // Récupérer la seed depuis les arguments de ligne de commande
    let args: Vec<String> = env::args().collect();
    let seed = args.iter()
        .find(|arg| arg.starts_with("seed="))
        .and_then(|arg| {
            let parts: Vec<&str> = arg.split('=').collect();
            if parts.len() > 1 {
                match parts[1].parse::<u32>() {
                    Ok(seed_value) => Some(seed_value),
                    Err(_) => {
                        println!("La seed doit être un nombre entier positif. Utilisation de la seed par défaut.");
                        None
                    }
                }
            } else {
                None
            }
        })
        .unwrap_or(SEED_DEFAULT);

    println!("Génération de la carte avec la seed: {}", seed);

    let (carte, carte_connue, (base_x, base_y)) = generer_carte(width, height, seed);

    // Créer la base et démarrer son thread
    let base = Base::new(
        width,
        height,
        base_x,
        base_y,
        carte.clone(),
        carte_connue.clone(),
    );

    Base::demarrer_thread_base(Arc::clone(&base), width, height);

    // Garder le programme en vie
    loop {
        if let Ok(base_guard) = base.lock() {
            let energie = *base_guard.energie.lock().unwrap();
            let minerais = *base_guard.minerais.lock().unwrap();
            let science = *base_guard.science.lock().unwrap();
            let nb_robots = base_guard.robots_deployes.lock().unwrap().len();

            let mut nb_explorateurs = 0;
            let mut nb_collecteurs = 0;
            if let Ok(robots) = base_guard.robots_deployes.lock() {
                for robot in robots.iter() {
                    match robot.get_type() {
                        TypeCase::Explorateur => nb_explorateurs += 1,
                        TypeCase::Collecteur => nb_collecteurs += 1,
                        _ => {}
                    }
                }
            }

            let ressources = format!(
                "Ressources: {} énergie, {} minerais, {} science | Robots: {} totaux ({} explorateurs, {} collecteurs)",
                energie, minerais, science, nb_robots, nb_explorateurs, nb_collecteurs
            );

            run_ui(
                &base_guard.carte_connue,
                &ressources,
                &base_guard.robots_deployes,
            )
            .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
