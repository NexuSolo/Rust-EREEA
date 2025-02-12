use crate::base::Base;
use crate::generation::TypeCase;
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
                let direction = rng.random_range(0..4);

                let mut new_x = x;
                let mut new_y = y;
                let can_move = match direction {
                    0 => {
                        if y > 0 {
                            new_y = y - 1;
                            true
                        } else {
                            false
                        }
                    } // Haut
                    1 => {
                        if y < map_height - 1 {
                            new_y = y + 1;
                            true
                        } else {
                            false
                        }
                    } // Bas
                    2 => {
                        if x > 0 {
                            new_x = x - 1;
                            true
                        } else {
                            false
                        }
                    } // Gauche
                    _ => {
                        if x < map_width - 1 {
                            new_x = x + 1;
                            true
                        } else {
                            false
                        }
                    } // Droite
                };

                // Vérifier si le déplacement est possible (pas de mur)
                if can_move {
                    if let Ok(base) = base.lock() {
                        if let Ok(carte_reelle) = base.carte_reelle.lock() {
                            if let Some(row) = carte_reelle.get(new_y) {
                                if let Some(case_type) = row.get(new_x) {
                                    if *case_type != TypeCase::Mur {
                                        // Déplacement autorisé
                                        *position_x.lock().unwrap() = new_x;
                                        *position_y.lock().unwrap() = new_y;
                                    }
                                }
                            }
                        }
                    }
                }

                // Communication avec la base après le déplacement
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

                thread::sleep(Duration::from_millis(50));
            }
        });

        explorateur
    }
}

impl Robot for Explorateur {
    fn next_move(&self) {
        //todo
    }

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
}

pub struct Collecteur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
}

impl Collecteur {
    pub fn new(x: usize, y: usize) -> Self {
        let collecteur = Collecteur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
        };

        let pos_x = Arc::clone(&collecteur.position_x);
        let pos_y = Arc::clone(&collecteur.position_y);
        let at_base = Arc::clone(&collecteur.at_base);

        thread::spawn(move || loop {
            let x = *pos_x.lock().unwrap();
            let y = *pos_y.lock().unwrap();
            let at_base = *at_base.lock().unwrap();
            thread::sleep(Duration::from_secs(1));
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
}
