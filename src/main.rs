extern crate termion;
mod generation;
mod ui;

use generation::{generer_carte, TypeCase};
use ui::run_ui;

use termion::terminal_size;

fn main() {
    let (width, height) = terminal_size().unwrap();
    let width = width as usize; // Réduire la largeur de 10 colonnes
    let height = (height as usize) - 5; // Réduire la hauteur de 5 lignes
    let seed = 5776968;
    let carte = generer_carte(width, height, seed);

    for row in &carte {
        for case in row {
            let symbol = match case {
                TypeCase::Vide => ' ',
                TypeCase::Base => 'B',
                TypeCase::Mur => '#',
                TypeCase::Energie => 'E',
                TypeCase::Minerais => 'M',
                TypeCase::Science => 'S',
                TypeCase::Explorateur => 'X',
                TypeCase::Collecteur => 'C',
            };
            print!("{}", symbol);
        }
        println!();
    }
    let ressources = "Energie: 10, Minerais: 5, Science: 2";

    if let Err(e) = run_ui(carte, ressources) {
        eprintln!("Error running UI: {}", e);
    }
}
