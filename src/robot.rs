use crate::base::Base;
use crate::generation::TypeCase;
use crate::pathfinding::find_path;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub trait Robot: Send {
    fn get_type(&self) -> TypeCase;
    fn get_position_x(&self) -> usize;
    fn get_position_y(&self) -> usize;
}

pub struct Explorateur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
}

impl Explorateur {
    pub fn new(
        map_width: usize,
        map_height: usize,
        x: usize,
        y: usize,
        base_ref: Arc<Mutex<Base>>,
    ) -> Self {
        let explorateur = Explorateur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
        };

        let position_x = Arc::clone(&explorateur.position_x);
        let position_y = Arc::clone(&explorateur.position_y);
        let base = Arc::clone(&base_ref);

        thread::spawn(move || {
            loop {
                let x = *position_x.lock().unwrap();
                let y = *position_y.lock().unwrap();
                let mut rng = rand::rng();

                // On va évaluer chaque direction possible
                let mut directions = vec![];
                let possible_moves = [
                    (0, -1, 0), // Haut
                    (0, 1, 1),  // Bas
                    (-1, 0, 2), // Gauche
                    (1, 0, 3),  // Droite
                ];

                if let Ok(base) = base.lock() {
                    if let Ok(carte_connue) = base.carte_connue.lock() {
                        if let Ok(carte_reelle) = base.carte_reelle.lock() {
                            for (dx, dy, dir) in possible_moves.iter() {
                                let new_x = x as i32 + dx;
                                let new_y = y as i32 + dy;

                                if new_x >= 0
                                    && new_x < map_width as i32
                                    && new_y >= 0
                                    && new_y < map_height as i32
                                {
                                    let new_x = new_x as usize;
                                    let new_y = new_y as usize;

                                    // Ajout d'une logique de priorité pour les cases inconnues qui va augmenter la probabilité de se diriger vers elles
                                    if let Some(row) = carte_reelle.get(new_y) {
                                        if let Some(case_type) = row.get(new_x) {
                                            if *case_type != TypeCase::Mur {
                                                let weight = if let Some(known_row) =
                                                    carte_connue.get(new_y)
                                                {
                                                    if let Some(known_type) = known_row.get(new_x) {
                                                        if *known_type == TypeCase::Inconnu {
                                                            3 // Donner plus de poids aux cases inexplorées
                                                        } else {
                                                            1
                                                        }
                                                    } else {
                                                        3
                                                    }
                                                } else {
                                                    3
                                                };

                                                // Ajouter la direction weight fois pour augmenter sa probabilité
                                                for _ in 0..weight {
                                                    directions.push(*dir);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Si aucune direction n'est possible, on ne bouge pas
                if !directions.is_empty() {
                    let direction = directions[rng.random_range(0..directions.len())];
                    let (dx, dy) = match direction {
                        0 => (0, -1), // Haut
                        1 => (0, 1),  // Bas
                        2 => (-1, 0), // Gauche
                        _ => (1, 0),  // Droite
                    };

                    let new_x = (x as i32 + dx) as usize;
                    let new_y = (y as i32 + dy) as usize;

                    *position_x.lock().unwrap() = new_x;
                    *position_y.lock().unwrap() = new_y;
                }

                // Communication avec la base après le déplacement (mise à jour de la vision)
                let x = *position_x.lock().unwrap();
                let y = *position_y.lock().unwrap();

                if let Ok(base) = base.lock() {
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let new_x = x as i32 + dx;
                            let new_y = y as i32 + dy;

                            if (dx.abs() + dy.abs()) <= 2 {
                                if new_x >= 0
                                    && new_y >= 0
                                    && new_x < map_width as i32
                                    && new_y < map_height as i32
                                {
                                    let new_x = new_x as usize;
                                    let new_y = new_y as usize;

                                    if let Ok(carte_reelle) = base.carte_reelle.lock() {
                                        if let Some(case_type) =
                                            carte_reelle.get(new_y).and_then(|row| row.get(new_x))
                                        {
                                            let case_type = case_type.clone();
                                            base.mettre_a_jour_carte(new_x, new_y, case_type);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });

        explorateur
    }
}

impl Robot for Explorateur {
    fn get_type(&self) -> TypeCase {
        TypeCase::Explorateur
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }
}

pub struct Collecteur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    base_ref: Arc<Mutex<Base>>,
    at_base: Arc<Mutex<bool>>,
    path: Arc<Mutex<Vec<(usize, usize)>>>,
    collected_resource: Arc<Mutex<Option<TypeCase>>>,
    destination: Arc<Mutex<Option<(usize, usize)>>>,
}

impl Collecteur {
    pub fn new(x: usize, y: usize, base_ref: Arc<Mutex<Base>>) -> Self {
        let collecteur = Collecteur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
            base_ref: Arc::clone(&base_ref),
            path: Arc::new(Mutex::new(Vec::new())),
            collected_resource: Arc::new(Mutex::new(None)),
            destination: Arc::new(Mutex::new(None)),
        };

        let position_x = Arc::clone(&collecteur.position_x);
        let position_y = Arc::clone(&collecteur.position_y);
        let base = Arc::clone(&base_ref);

        thread::spawn(move || loop {
            let x = *position_x.lock().unwrap();
            let y = *position_y.lock().unwrap();
        });

        collecteur
    }
}

impl Robot for Collecteur {
    fn get_type(&self) -> TypeCase {
        TypeCase::Collecteur
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }
}
