use crate::generation::TypeCase;
use crate::robot::{Collecteur, Explorateur, Robot};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Base {
    pub carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub robots_deployes: Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
    pub energie: Arc<Mutex<usize>>,
    pub minerais: Arc<Mutex<usize>>,
    pub science: Arc<Mutex<usize>>,
    pub position_x: usize,
    pub position_y: usize,
}

impl Base {
    pub fn new(width: usize, height: usize, position_x: usize, position_y: usize) -> Self {
        println!(
            "[BASE] Création d'une nouvelle base aux coordonnées ({}, {})",
            position_x, position_y
        );
        let mut base = Base {
            carte_connue: Arc::new(Mutex::new(vec![vec![TypeCase::Vide; width]; height])),
            robots_deployes: Arc::new(Mutex::new(Vec::new())),
            energie: Arc::new(Mutex::new(0)),
            minerais: Arc::new(Mutex::new(0)),
            science: Arc::new(Mutex::new(0)),
            position_x,
            position_y,
        };
        println!("[BASE] Ajout des robots initiaux...");
        base.ajouter_robot(Box::new(Explorateur::new(
            width, height, position_x, position_y,
        )));
        base.ajouter_robot(Box::new(Collecteur::new(position_x, position_y)));
        base
    }

    pub fn demarrer_thread_base(&self, map_width: usize, map_height: usize) {
        println!("[BASE] Démarrage du thread principal de la base");
        let energie = Arc::clone(&self.energie);
        let minerais = Arc::clone(&self.minerais);
        let science = Arc::clone(&self.science);
        let robots = Arc::clone(&self.robots_deployes);
        let pos_x = self.position_x;
        let pos_y = self.position_y;

        thread::spawn(move || {
            let mut rng = rand::rng();

            loop {
                let energie_val = *energie.lock().unwrap();
                let minerais_val = *minerais.lock().unwrap();
                let science_val = *science.lock().unwrap();

                println!(
                    "[BASE] État des ressources - Énergie: {}, Minerais: {}, Science: {}",
                    energie_val, minerais_val, science_val
                );
                println!(
                    "[BASE] Nombre de robots actifs: {}",
                    robots.lock().unwrap().len()
                );

                if energie_val >= 3 && minerais_val >= 3 && science_val >= 3 {
                    let mut robots = robots.lock().unwrap();
                    if rng.random_range(0..3) == 0 {
                        println!(
                            "[BASE] Création d'un nouveau robot explorateur en ({}, {})",
                            pos_x, pos_y
                        );
                        robots.push(Box::new(Explorateur::new(
                            map_width, map_height, pos_x, pos_y,
                        )));
                    } else {
                        println!(
                            "[BASE] Création d'un nouveau robot collecteur en ({}, {})",
                            pos_x, pos_y
                        );
                        robots.push(Box::new(Collecteur::new(pos_x, pos_y)));
                    }

                    *energie.lock().unwrap() -= 3;
                    *minerais.lock().unwrap() -= 3;
                    *science.lock().unwrap() -= 3;
                    println!("[BASE] Consommation de ressources pour la création du robot");
                }

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn ajouter_robot(&mut self, robot: Box<dyn Robot + Send>) {
        println!(
            "[BASE] Ajout d'un robot de type {:?} à la position ({}, {})",
            robot.get_type(),
            robot.get_position_x(),
            robot.get_position_y()
        );
        self.robots_deployes.lock().unwrap().push(robot);
    }

    pub fn ajouter_ressource(&self, ressource: TypeCase) {
        match ressource {
            TypeCase::Energie => {
                *self.energie.lock().unwrap() += 1;
                println!("[BASE] Ajout d'une unité d'énergie");
            }
            TypeCase::Minerais => {
                *self.minerais.lock().unwrap() += 1;
                println!("[BASE] Ajout d'une unité de minerais");
            }
            TypeCase::Science => {
                *self.science.lock().unwrap() += 1;
                println!("[BASE] Ajout d'une unité de science");
            }
            _ => (),
        }
    }

    pub fn mettre_a_jour_carte(&self, x: usize, y: usize, case: TypeCase) {
        let mut carte = self.carte_connue.lock().unwrap();
        if x < carte[0].len() && y < carte.len() {
            println!(
                "[BASE] Mise à jour de la carte en ({}, {}) : {:?}",
                x, y, &case
            );
            carte[y][x] = case;
        }
    }
}
