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
    fn communicate(&self);
}

pub struct Explorateur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
}

impl Explorateur {
    pub fn new(map_width: usize, map_height: usize, x: usize, y: usize) -> Self {
        println!(
            "[EXPLORATEUR] Création d'un nouveau robot explorateur en ({}, {})",
            x, y
        );
        let explorateur = Explorateur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
        };

        let pos_x = Arc::clone(&explorateur.position_x);
        let pos_y = Arc::clone(&explorateur.position_y);
        let at_base = Arc::clone(&explorateur.at_base);

        thread::spawn(move || {
            let thread_id = thread::current().id();
            println!("[EXPLORATEUR] Thread {:?} démarré", thread_id);

            loop {
                let x = *pos_x.lock().unwrap();
                let y = *pos_y.lock().unwrap();
                let at_base = *at_base.lock().unwrap();
                println!(
                    "[EXPLORATEUR {:?}] Position: ({}, {}), À la base: {}",
                    thread_id, x, y, at_base
                );
                let mut rng = rand::rng();
                let direction = rng.random_range(0..4);
                match direction {
                    0 => {
                        if y > 0 {
                            *pos_y.lock().unwrap() -= 1;
                        }
                    } // Haut
                    1 => {
                        if y < map_height - 1 {
                            *pos_y.lock().unwrap() += 1
                        }
                    } // Bas
                    2 => {
                        if x > 0 {
                            *pos_x.lock().unwrap() -= 1
                        }
                    } // Gauche
                    _ => {
                        if x < map_width - 1 {
                            *pos_x.lock().unwrap() += 1
                        }
                    } // Droite
                }
                thread::sleep(Duration::from_secs(1));
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

    fn communicate(&self) {
        // todo
    }
}

pub struct Collecteur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
}

impl Collecteur {
    pub fn new(x: usize, y: usize) -> Self {
        println!(
            "[COLLECTEUR] Création d'un nouveau robot collecteur en ({}, {})",
            x, y
        );
        let collecteur = Collecteur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
        };

        let pos_x = Arc::clone(&collecteur.position_x);
        let pos_y = Arc::clone(&collecteur.position_y);
        let at_base = Arc::clone(&collecteur.at_base);

        thread::spawn(move || {
            let thread_id = thread::current().id();
            println!("[COLLECTEUR] Thread {:?} démarré", thread_id);

            loop {
                let x = *pos_x.lock().unwrap();
                let y = *pos_y.lock().unwrap();
                let at_base = *at_base.lock().unwrap();
                println!(
                    "[COLLECTEUR {:?}] Position: ({}, {}), À la base: {}",
                    thread_id, x, y, at_base
                );

                thread::sleep(Duration::from_secs(1));
            }
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

    fn communicate(&self) {
        //todo
    }
}
