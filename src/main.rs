mod generation;

use generation::{generer_carte, TypeCase};

fn main() {
    let width = 50;
    let height = 50;
    let seed = 577679768;
    let carte = generer_carte(width, height, seed);

    for row in carte {
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
}
