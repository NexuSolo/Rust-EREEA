use crate::base::Base;
use crate::generation::TypeCase;
use crate::pathfinding::find_path;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub trait Robot: Send {
    fn next_move(&self);
    fn get_type(&self) -> TypeCase;
    fn get_position_x(&self) -> usize;
    fn get_position_y(&self) -> usize;
    fn is_at_base(&self) -> bool;
    fn set_destination(&self, x: usize, y: usize);
}

pub struct Explorateur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
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
            at_base: Arc::new(Mutex::new(true)),
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
    fn next_move(&self) {}

    fn get_type(&self) -> TypeCase {
        TypeCase::Explorateur
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }

    fn is_at_base(&self) -> bool {
        *self.at_base.lock().unwrap()
    }

    fn set_destination(&self, x: usize, y: usize) {}
}

pub struct Collecteur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    base: Arc<Mutex<Base>>,
    at_base: Arc<Mutex<bool>>,
    path: Arc<Mutex<Vec<(usize, usize)>>>,
    collected_resource: Arc<Mutex<Option<TypeCase>>>,
    destination: Arc<Mutex<Option<(usize, usize)>>>,
}

impl Collecteur {
    pub fn new(x: usize, y: usize, base: Arc<Mutex<Base>>) -> Self {
        let collecteur = Collecteur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
            base: Arc::clone(&base),
            path: Arc::new(Mutex::new(Vec::new())),
            collected_resource: Arc::new(Mutex::new(None)),
            destination: Arc::new(Mutex::new(None)),
        };

        let pos_x = Arc::clone(&collecteur.position_x);
        let pos_y = Arc::clone(&collecteur.position_y);
        let at_base = Arc::clone(&collecteur.at_base);
        let path = Arc::clone(&collecteur.path);
        let collected_resource = Arc::clone(&collecteur.collected_resource);
        let destination = Arc::clone(&collecteur.destination);
        let base = Arc::clone(&base);

        thread::spawn(move || loop {
            let x = *pos_x.lock().unwrap();
            let y = *pos_y.lock().unwrap();
            let is_at_base = at_base.lock().unwrap();
            let mut path_guard = path.lock().unwrap();
            let mut collected = collected_resource.lock().unwrap();
            let destination_guard = destination.lock().unwrap();

            if let Some((dest_x, dest_y)) = *destination_guard {
                if path_guard.is_empty() {
                    println!("Calcul du chemin vers la destination");
                    // Calculer le chemin vers la destination
                    if let Ok(base_guard) = base.lock() {
                        if let Ok(carte_connue) = base_guard.carte_connue.lock() {
                            if let Some(new_path) =
                                find_path((x, y), (dest_x, dest_y), &carte_connue)
                            {
                                *path_guard = new_path;
                            }
                        }
                    }
                } else {
                    println!("Déplacement vers la destination");
                    // Vérifier si on est arrivé à destination
                    if x == dest_x && y == dest_y {
                        println!("E");
                        // Si à la base et qu'on a une ressource, la déposer
                        if *is_at_base && collected.is_some() {
                            if let Ok(base_guard) = base.lock() {
                                if let Some(resource) = collected.take() {
                                    base_guard.ajouter_ressource(resource);
                                }
                            }
                        }
                        // Si pas à la base et pas de ressource, collecter
                        else if !*is_at_base && collected.is_none() {
                            println!("A");
                            if let Ok(base_guard) = base.lock() {
                                println!("B");
                                if let Ok(carte_reelle) = base_guard.carte_reelle.lock() {
                                    println!("C");
                                    if let Some(ressource) =
                                        carte_reelle.get(y).and_then(|row| row.get(x))
                                    {
                                        println!("D");
                                        match ressource {
                                            TypeCase::Minerais
                                            | TypeCase::Energie
                                            | TypeCase::Science => {
                                                *collected = Some(ressource.clone());
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        // Effacer la destination une fois l'objectif atteint
                        *destination.lock().unwrap() = None;
                        path_guard.clear();
                    } else if !path_guard.is_empty() {
                        println!("Continuer le chemin");
                        // Continuer le chemin
                        if let Some(&(next_x, next_y)) = path_guard.first() {
                            *pos_x.lock().unwrap() = next_x;
                            *pos_y.lock().unwrap() = next_y;
                            path_guard.remove(0);
                        }
                    }
                }
            } else {
                println!("autre");
                // Si on n'est pas à la base et qu'on a une ressource, retourner à la base
                if !*is_at_base && collected.is_some() {
                    if let Ok(base_guard) = base.lock() {
                        let base_pos = (base_guard.position_x, base_guard.position_y);
                        if let Ok(carte_connue) = base_guard.carte_connue.lock() {
                            if let Some(new_path) = find_path((x, y), base_pos, &carte_connue) {
                                *path_guard = new_path;
                            }
                            if x == base_pos.0 && y == base_pos.1 {
                                *at_base.lock().unwrap() = true;
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        });

        collecteur
    }
}

impl Robot for Collecteur {
    fn next_move(&self) {
        //todo
    }

    fn get_type(&self) -> TypeCase {
        TypeCase::Collecteur
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }

    fn is_at_base(&self) -> bool {
        *self.at_base.lock().unwrap()
    }

    fn set_destination(&self, x: usize, y: usize) {
        *self.destination.lock().unwrap() = Some((x, y));
    }
}
