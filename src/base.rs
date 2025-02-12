use crate::generation::TypeCase;
use crate::robot::{Collecteur, Explorateur, Robot};
use log::{debug, info, trace};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
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
    pub running: Arc<AtomicBool>,
}

impl Base {
    pub fn new(
        width: usize,
        height: usize,
        position_x: usize,
        position_y: usize,
        carte_reelle: Vec<Vec<TypeCase>>,
        carte_connue_init: Vec<Vec<TypeCase>>,
    ) -> Arc<Mutex<Self>> {
        trace!("[BASE] Initialisation de la base avec width: {}, height: {}, position_x: {}, position_y: {}", width, height, position_x, position_y);
        info!(
            "[BASE] Création de la base à la position ({}, {})",
            position_x, position_y
        );

        let carte_reelle = Arc::new(Mutex::new(carte_reelle));
        let carte_connue = Arc::new(Mutex::new(carte_connue_init));
        let robots_deployes = Arc::new(Mutex::new(Vec::new()));
        let energie = Arc::new(Mutex::new(0));
        let minerais = Arc::new(Mutex::new(0));
        let science = Arc::new(Mutex::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let base = Arc::new(Mutex::new(Base {
            carte_reelle: Arc::clone(&carte_reelle),
            carte_connue: Arc::clone(&carte_connue),
            robots_deployes: Arc::clone(&robots_deployes),
            energie: Arc::clone(&energie),
            minerais: Arc::clone(&minerais),
            science: Arc::clone(&science),
            position_x,
            position_y,
            running: Arc::clone(&running),
        }));

        // Ajouter les robots initiaux
        trace!("[BASE] base.rs l58");
        if let Ok(mut base_guard) = base.lock() {
            trace!("[BASE] Ajout des robots initiaux");
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
                Arc::clone(&carte_connue),
                Arc::clone(&base),
            )));
        }

        info!("[BASE] Base créée avec succès");
        base
    }

    pub fn demarrer_thread_base(base: Arc<Mutex<Base>>, map_width: usize, map_height: usize) {
        info!("[BASE] Démarrage du thread de la base");
        thread::spawn(move || {
            let mut rng = rand::rng();
            while base.lock().unwrap().running.load(Ordering::SeqCst) {
                trace!("[BASE] Début de la boucle principale du thread de la base");
                // Vérifier d'abord si on peut créer un robot
                let should_create_robot = {
                    trace!("[BASE] base.rs l88");
                    if let Ok(base_guard) = base.lock() {
                        let energie = *base_guard.energie.lock().unwrap();
                        let minerais = *base_guard.minerais.lock().unwrap();
                        let science = *base_guard.science.lock().unwrap();
                        trace!(
                            "[BASE] Ressources actuelles - Energie: {}, Minerais: {}, Science: {}",
                            energie,
                            minerais,
                            science
                        );
                        energie >= 3 && minerais >= 3 && science >= 3
                    } else {
                        false
                    }
                };

                // Gérer les collecteurs
                trace!("[BASE] base.rs l106");
                if let Ok(base_guard) = base.lock() {
                    trace!("[BASE] Gestion des collecteurs");
                    base_guard.gerer_collecteurs();
                }

                // Créer un nouveau robot si possible
                if should_create_robot {
                    trace!("[BASE] base.rs l114");
                    if let Ok(base_guard) = base.lock() {
                        let pos_x = base_guard.position_x;
                        let pos_y = base_guard.position_y;
                        let base_thread = Arc::clone(&base);

                        // Créer le robot
                        let nouveau_robot: Box<dyn Robot + Send> = if rng.random_bool(0.5) {
                            trace!("[BASE] Création d'un nouveau robot Explorateur");
                            Box::new(Explorateur::new(
                                map_width,
                                map_height,
                                pos_x,
                                pos_y,
                                Arc::clone(&base_thread),
                            ))
                        } else {
                            trace!("[BASE] Création d'un nouveau robot Collecteur");
                            Box::new(Collecteur::new(
                                pos_x,
                                pos_y,
                                Arc::clone(&base_guard.carte_connue),
                                Arc::clone(&base_thread),
                            ))
                        };

                        // Réduire les ressources et ajouter le robot
                        {
                            trace!("[BASE] Réduction des ressources pour la création du robot");
                            *base_guard.energie.lock().unwrap() -= 3;
                            *base_guard.minerais.lock().unwrap() -= 3;
                            *base_guard.science.lock().unwrap() -= 3;
                            base_guard
                                .robots_deployes
                                .lock()
                                .unwrap()
                                .push(nouveau_robot);
                            info!("[BASE] Nouveau robot créé et ajouté à la base");
                        }
                    }
                }

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn gerer_collecteurs(&self) {
        info!("[BASE] Gestion des collecteurs");

        // Récupérer les informations des ressources d'abord
        let ressources_info = {
            let energie = *self.energie.lock().unwrap();
            let minerais = *self.minerais.lock().unwrap();
            let science = *self.science.lock().unwrap();
            trace!(
                "[BASE] Ressources actuelles - Energie: {}, Minerais: {}, Science: {}",
                energie,
                minerais,
                science
            );
            vec![
                (TypeCase::Energie, energie),
                (TypeCase::Minerais, minerais),
                (TypeCase::Science, science),
            ]
        };

        // Trier les ressources par priorité
        let mut ressources_prioritaires = ressources_info.clone();
        ressources_prioritaires.sort_by_key(|&(_, count)| count);
        trace!(
            "[BASE] Ressources triées par priorité: {:?}",
            ressources_prioritaires
        );

        // Collecter les informations sur les ressources disponibles
        let ressources_positions = if let Ok(carte) = self.carte_connue.lock() {
            let mut positions = Vec::new();
            for y in 0..carte.len() {
                for x in 0..carte[0].len() {
                    match carte[y][x] {
                        TypeCase::Energie | TypeCase::Minerais | TypeCase::Science => {
                            positions.push((x, y, carte[y][x].clone()));
                        }
                        _ => {}
                    }
                }
            }
            trace!(
                "[BASE] Positions des ressources disponibles: {:?}",
                positions
            );
            positions
        } else {
            return;
        };

        // Obtenir les collecteurs disponibles une seule fois
        let robots_guard = if let Ok(guard) = self.robots_deployes.lock() {
            guard
        } else {
            return;
        };

        // Identifier les collecteurs disponibles et leurs destinations actuelles
        let mut collecteurs_info = Vec::new();
        let mut destinations_occupees = Vec::new();
        for (index, robot) in robots_guard.iter().enumerate() {
            if let Some(collecteur) = robot.as_any().downcast_ref::<Collecteur>() {
                if collecteur.is_at_base() && collecteur.get_destination().is_none() {
                    collecteurs_info.push(index);
                }
                if let Some(dest) = collecteur.get_destination() {
                    destinations_occupees.push(dest);
                }
            }
        }
        trace!("[BASE] Collecteurs disponibles: {:?}", collecteurs_info);
        trace!("[BASE] Destinations occupées: {:?}", destinations_occupees);

        // Assigner les ressources aux collecteurs disponibles
        for (ressource_type, _) in ressources_prioritaires {
            let ressources_dispo: Vec<_> = ressources_positions
                .iter()
                .filter(|(x, y, type_case)| {
                    *type_case == ressource_type && !destinations_occupees.contains(&(*x, *y))
                })
                .collect();

            for (x, y, _) in ressources_dispo {
                if let Some(collector_index) = collecteurs_info.first() {
                    if let Some(collecteur) = robots_guard[*collector_index]
                        .as_any()
                        .downcast_ref::<Collecteur>()
                    {
                        trace!(
                            "[BASE] Assignation de la ressource ({}, {}) au collecteur",
                            x,
                            y
                        );
                        collecteur.set_destination(*x, *y);
                        destinations_occupees.push((*x, *y));
                        collecteurs_info.remove(0);
                        info!("[BASE] Collecteur assigné à la ressource ({}, {})", x, y);
                    }

                    if collecteurs_info.is_empty() {
                        return;
                    }
                }
            }
        }
    }

    pub fn ajouter_robot(&mut self, robot: Box<dyn Robot + Send>) {
        trace!("[BASE] Ajout d'un robot à la base");
        self.robots_deployes.lock().unwrap().push(robot);
        info!("[BASE] Robot ajouté à la base");
    }

    pub fn ajouter_ressource(&self, ressource: TypeCase) {
        match ressource {
            TypeCase::Energie => {
                *self.energie.lock().unwrap() += 1;
                info!("[BASE] Ajout d'une unité d'énergie");
            }
            TypeCase::Minerais => {
                *self.minerais.lock().unwrap() += 1;
                info!("[BASE] Ajout d'une unité de minerais");
            }
            TypeCase::Science => {
                *self.science.lock().unwrap() += 1;
                info!("[BASE] Ajout d'une unité de science");
            }
            _ => (),
        }
    }

    pub fn mettre_a_jour_carte(&self, x: usize, y: usize, case: TypeCase) {
        let mut carte = self.carte_connue.lock().unwrap();
        if x < carte[0].len() && y < carte.len() {
            if carte[y][x] != case {
                debug!(
                    "[BASE] Changement détecté à ({}, {}): {:?} -> {:?}",
                    x, y, carte[y][x], case
                );
                carte[y][x] = case;
                debug!("[BASE] Mise à jour de la carte à ({}, {})", x, y);
            }
        }
    }
}
