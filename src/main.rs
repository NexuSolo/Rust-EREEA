mod base;
mod generation;
mod robot;
mod ui;
use crossterm::terminal;

use crate::base::Base;
use crate::generation::{generer_carte, TypeCase};
use crate::ui::run_ui;

fn main() {
    println!("=== DÉMARRAGE DU PROGRAMME ===");

    // Paramètres de la carte
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;
    let seed = 577679768;
    println!(
        "[MAIN] Initialisation de la carte {}x{} avec seed {}",
        width, height, seed
    );

    // Générer la carte
    let carte = generer_carte(width, height, seed);
    println!("[MAIN] Carte générée avec succès");

    // Trouver la position de la base dans la carte générée
    let mut base_x = 0;
    let mut base_y = 0;
    for y in 0..height {
        for x in 0..width {
            if carte[y][x] == TypeCase::Base {
                base_x = x;
                base_y = y;
                println!(
                    "[MAIN] Base trouvée aux coordonnées ({}, {})",
                    base_x, base_y
                );
                break;
            }
        }
    }

    // Créer la base et démarrer son thread
    println!("[MAIN] Création de la base...");
    let base = Base::new(width, height, base_x, base_y);
    println!("[MAIN] Démarrage du thread de la base...");
    base.demarrer_thread_base();

    println!("===============================\n");

    println!("[MAIN] Programme en cours d'exécution...");
    // Garder le programme en vie
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        run_ui(&carte, "Ressources: 100 énergie, 50 minerais, 25 science").unwrap();
    }
}
