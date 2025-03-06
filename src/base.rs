use crate::generation::TypeCase;
use crate::robot::{Collecteur, Explorateur, Robot};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Structure pour trier les ressources dans la file de priorité
#[derive(Clone, Debug, Eq)]
struct PrioritizedResource {
    x: usize,
    y: usize,
    distance: usize,
    priority: bool,
}

// Implémentation des traits nécessaires pour la file de priorité
impl PartialEq for PrioritizedResource {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.priority == other.priority
    }
}

// On inverse l'ordre pour que les ressources prioritaires et proches soient au début
impl PartialOrd for PrioritizedResource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedResource {
    fn cmp(&self, other: &Self) -> Ordering {
        // D'abord comparer la priorité
        match (self.priority, other.priority) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => {}
        }

        // Ensuite comparer la distance
        other.distance.cmp(&self.distance)
    }
}

pub struct Base {
    pub carte_reelle: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub robots_deployes: Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
    pub energie: Arc<Mutex<usize>>,
    pub minerais: Arc<Mutex<usize>>,
    pub science: Arc<Mutex<usize>>,
    pub position_x: usize,
    pub position_y: usize,
    reserved_resources: Arc<Mutex<HashSet<(usize, usize)>>>,
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
        let reserved_resources = Arc::new(Mutex::new(HashSet::new()));

        let base = Arc::new(Mutex::new(Base {
            carte_reelle: Arc::clone(&carte_reelle),
            carte_connue: Arc::clone(&carte_connue),
            robots_deployes: Arc::clone(&robots_deployes),
            energie: Arc::clone(&energie),
            minerais: Arc::clone(&minerais),
            science: Arc::clone(&science),
            position_x,
            position_y,
            reserved_resources,
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
            base_guard.ajouter_robot(Box::new(Collecteur::new(
                position_x,
                position_y,
                Arc::clone(&base),
            )));
        }

        base
    }

    pub fn demarrer_thread_base(base: Arc<Mutex<Base>>, _map_width: usize, _map_height: usize) {
        thread::spawn(move || loop {
            // Logique de gestion des robots et des ressources
            if let Ok(_base) = base.lock() {
                // Future logique de gestion de la base
            }
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

    pub fn next_resource(&self) -> Option<(usize, usize)> {
        let carte_connue = self.carte_connue.lock().unwrap();
        let energie_count = *self.energie.lock().unwrap();
        let minerais_count = *self.minerais.lock().unwrap();
        let science_count = *self.science.lock().unwrap();
        let reserved = self.reserved_resources.lock().unwrap();

        let height = carte_connue.len();
        let width = carte_connue[0].len();

        // Déterminer quelle ressource a le compte le plus bas
        let min_resource_count = energie_count.min(minerais_count).min(science_count);

        let mut priority_queue: BinaryHeap<PrioritizedResource> = BinaryHeap::new();

        // Parcourir toute la carte pour trouver les ressources
        for y in 0..height {
            for x in 0..width {
                // Vérifier si la case n'est pas déjà réservée
                if reserved.contains(&(x, y)) {
                    continue;
                }

                let case = &carte_connue[y][x];
                match case {
                    TypeCase::Energie | TypeCase::Minerais | TypeCase::Science => {
                        // Calculer la distance entre la ressource et la base
                        let distance =
                            Self::manhattan_distance(self.position_x, self.position_y, x, y);

                        // Déterminer si cette ressource est prioritaire
                        let is_priority = match case {
                            TypeCase::Energie => energie_count == min_resource_count,
                            TypeCase::Minerais => minerais_count == min_resource_count,
                            TypeCase::Science => science_count == min_resource_count,
                            _ => false,
                        };

                        priority_queue.push(PrioritizedResource {
                            x,
                            y,
                            distance,
                            priority: is_priority,
                        });
                    }
                    _ => continue,
                }
            }
        }

        // Prendre la ressource la plus prioritaire
        if let Some(resource) = priority_queue.pop() {
            drop(carte_connue);
            drop(reserved);
            self.reserved_resources
                .lock()
                .unwrap()
                .insert((resource.x, resource.y));
            return Some((resource.x, resource.y));
        }

        None
    }

    fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
        ((x1 as isize - x2 as isize).abs() + (y1 as isize - y2 as isize).abs()) as usize
    }

    pub fn release_resource(&self, x: usize, y: usize) {
        if let Ok(mut reserved) = self.reserved_resources.lock() {
            reserved.remove(&(x, y));
        }
    }

    pub fn ajouter_ressource(&self, ressource: TypeCase) {
        match ressource {
            TypeCase::Energie => {
                if let Ok(mut energie) = self.energie.lock() {
                    *energie += 1;
                }
            }
            TypeCase::Minerais => {
                if let Ok(mut minerais) = self.minerais.lock() {
                    *minerais += 1;
                }
            }
            TypeCase::Science => {
                if let Ok(mut science) = self.science.lock() {
                    *science += 1;
                }
            }
            _ => {}
        }
    }
}
