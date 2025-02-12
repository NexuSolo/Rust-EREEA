mod base;
mod generation;
mod robot;
mod ui;
use crossterm::terminal;
use env_logger::Env;
use log::{debug, info, trace};

use crate::base::Base;
use crate::generation::{generer_carte, TypeCase};
use crate::ui::run_ui;
use std::sync::Arc;

fn main() {
    // Initialiser le logger
    env_logger::init_from_env(Env::default().default_filter_or("trace"));

    // Paramètres de la carte
    let (width, height) = terminal::size().unwrap();
    let width = (width / 2) as usize;
    let height = height as usize;
    let seed = 577679768;

    let (carte, carte_connue, base_x, base_y) = generer_carte(width, height, seed);

    // Créer la base et démarrer son thread
    let base = Base::new(width, height, base_x, base_y, carte, carte_connue.clone());
    trace!("[MAIN] main.rs l28");
    let robots = if let Ok(base_guard) = base.lock() {
        trace!("[MAIN] Accès à la base pour obtenir les robots");
        Arc::clone(&base_guard.robots_deployes)
    } else {
        panic!("Impossible d'accéder à la base")
    };
    trace!("[MAIN] main.rs l35");
    let carte_connue = if let Ok(base_guard) = base.lock() {
        trace!("[MAIN] Accès à la base pour obtenir la carte connue");
        Arc::clone(&base_guard.carte_connue)
    } else {
        panic!("Impossible d'accéder à la base")
    };

    Base::demarrer_thread_base(Arc::clone(&base), width, height);

    // Garder le programme en vie
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Ok(carte) = carte_connue.lock() {
            trace!("[MAIN] Accès à la carte connue");
            trace!("[MAIN] main.rs l50");
            if let Ok(base_guard) = base.lock() {
                trace!("[MAIN] Accès à la base pour obtenir les ressources");
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
                run_ui(&carte, &ressources, &robots).unwrap();
            }
        }
    }
}
