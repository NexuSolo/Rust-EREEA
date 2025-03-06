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
    priority_level: usize,
}

// Implémentation des traits nécessaires pour la file de priorité
impl PartialEq for PrioritizedResource {
    fn eq(&self, other: &Self) -> bool {
        self.priority_level == other.priority_level && self.distance == other.distance
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
        // D'abord comparer le niveau de priorité
        match self.priority_level.cmp(&other.priority_level) {
            Ordering::Equal => {}
            ordering => return ordering,
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

    pub fn demarrer_thread_base(base: Arc<Mutex<Base>>, map_width: usize, map_height: usize) {
        thread::spawn(move || loop {
            let mut explorateurs_count = 0;
            let mut collecteurs_count = 0;
            let mut energie = 0;
            let mut minerais = 0;
            let mut science = 0;
            let mut position_x = 0;
            let mut position_y = 0;

            // Initialisé les variables
            if let Ok(base_guard) = base.lock() {
                position_x = base_guard.position_x;
                position_y = base_guard.position_y;

                if let Ok(robots) = base_guard.robots_deployes.lock() {
                    for robot in robots.iter() {
                        match robot.get_type() {
                            TypeCase::Explorateur => explorateurs_count += 1,
                            TypeCase::Collecteur => collecteurs_count += 1,
                            _ => {}
                        }
                    }
                }

                if let Ok(e) = base_guard.energie.lock() {
                    energie = *e;
                }
                if let Ok(m) = base_guard.minerais.lock() {
                    minerais = *m;
                }
                if let Ok(s) = base_guard.science.lock() {
                    science = *s;
                }
            }

            // Calculer le ratio et déterminer quel robot créer
            let ratio_actuel = if explorateurs_count == 0 {
                0.0
            } else {
                collecteurs_count as f32 / explorateurs_count as f32
            };

            let create_collecteur = ratio_actuel < 2.0
                && collecteurs_count > 0
                && science >= 1
                && minerais >= 5
                && energie >= 4;

            let create_explorateur = (ratio_actuel >= 2.0 || explorateurs_count == 0)
                && science >= 4
                && minerais >= 3
                && energie >= 2;

            //Création des robots
            if create_collecteur || create_explorateur {
                if let Ok(mut base_guard) = base.lock() {
                    if create_collecteur {
                        // Ressources pour un collecteur 1 Science, 5 Minerais, 4 Energie
                        if let Ok(mut s) = base_guard.science.lock() {
                            *s -= 1;
                        }
                        if let Ok(mut m) = base_guard.minerais.lock() {
                            *m -= 5;
                        }
                        if let Ok(mut e) = base_guard.energie.lock() {
                            *e -= 4;
                        }

                        base_guard.ajouter_robot(Box::new(Collecteur::new(
                            position_x,
                            position_y,
                            Arc::clone(&base),
                        )));
                    } else if create_explorateur {
                        // Ressources pour un explorateur 4 Sciences, 3 Minerais, 2 Energie
                        if let Ok(mut s) = base_guard.science.lock() {
                            *s -= 4;
                        }
                        if let Ok(mut m) = base_guard.minerais.lock() {
                            *m -= 3;
                        }
                        if let Ok(mut e) = base_guard.energie.lock() {
                            *e -= 2;
                        }

                        base_guard.ajouter_robot(Box::new(Explorateur::new(
                            map_width,
                            map_height,
                            position_x,
                            position_y,
                            Arc::clone(&base),
                        )));
                    }
                }
            }

            thread::sleep(Duration::from_secs(4));
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

        // Trouver le compteur de ressource le plus élevé
        let max_resource_count = energie_count.max(minerais_count).max(science_count);

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

                        // Calculer le niveau de priorité basé sur la différence avec le compteur le plus élevé
                        let priority_level = match case {
                            TypeCase::Energie => max_resource_count.saturating_sub(energie_count),
                            TypeCase::Minerais => max_resource_count.saturating_sub(minerais_count),
                            TypeCase::Science => max_resource_count.saturating_sub(science_count),
                            _ => 0,
                        };

                        priority_queue.push(PrioritizedResource {
                            x,
                            y,
                            distance,
                            priority_level,
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
