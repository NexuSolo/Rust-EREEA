use crate::generation::TypeCase;
use crate::robot::{Collecteur, Explorateur, Robot};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Base {
    pub carte_reelle: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub robots_deployes: Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
    pub energie: Arc<Mutex<usize>>,
    pub minerais: Arc<Mutex<usize>>,
    pub science: Arc<Mutex<usize>>,
    pub position_x: usize,
    pub position_y: usize,
}

impl Base {
    pub fn new(
        width: usize,
        height: usize,
        position_x: usize,
        position_y: usize,
        carte_reelle: Arc<Mutex<Vec<Vec<TypeCase>>>>,
        carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    ) -> Arc<Mutex<Self>> {
        let robots_deployes = Arc::new(Mutex::new(Vec::new()));
        let energie = Arc::new(Mutex::new(0));
        let minerais = Arc::new(Mutex::new(0));
        let science = Arc::new(Mutex::new(0));

        let base = Arc::new(Mutex::new(Base {
            carte_reelle: Arc::clone(&carte_reelle),
            carte_connue: Arc::clone(&carte_connue),
            robots_deployes: Arc::clone(&robots_deployes),
            energie: Arc::clone(&energie),
            minerais: Arc::clone(&minerais),
            science: Arc::clone(&science),
            position_x,
            position_y,
        }));

        // Ajouter les robots initiaux
        if let Ok(mut base_guard) = base.lock() {
            base_guard.ajouter_robot(Box::new(Explorateur::new(
                width,
                height,
                position_x,
                position_y,
                Arc::clone(&base),
            )));
            base_guard.ajouter_robot(Box::new(Explorateur::new(
                width,
                height,
                position_x,
                position_y,
                Arc::clone(&base),
            )));
            base_guard.ajouter_robot(Box::new(Collecteur::new(
                position_x,
                position_y,
                Arc::clone(&base),
            )));
        }

        base
    }

    pub fn demarrer_thread_base(base: Arc<Mutex<Base>>, map_width: usize, map_height: usize) {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
        });
    }

    pub fn ajouter_robot(&mut self, robot: Box<dyn Robot + Send>) {
        self.robots_deployes.lock().unwrap().push(robot);
    }

    pub fn mettre_a_jour_carte(&self, x: usize, y: usize, case: TypeCase) {
        let mut carte = self.carte_connue.lock().unwrap();
        if x < carte[0].len() && y < carte.len() {
            carte[y][x] = case;
        }
    }
}
