use crate::base::Base;
use crate::generation::TypeCase;
use log::{debug, info, trace, warn};
use rand::Rng;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub trait Robot: Send + Any {
    fn next_move(&self);
    fn get_type(&self) -> TypeCase;
    fn get_position_x(&self) -> usize;
    fn get_position_y(&self) -> usize;
    fn is_at_base(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
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
        trace!(
            "[EXPLORATEUR] Initialisation de l'explorateur à la position ({}, {})",
            x,
            y
        );
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
                    debug!("[EXPLORATEUR] Déplacement vers ({}, {})", new_x, new_y);
                }

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
                                            debug!(
                                                "[EXPLORATEUR] Mise à jour de la carte à ({}, {})",
                                                new_x, new_y
                                            );
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Collecteur {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
    chemin: Arc<Mutex<Vec<(usize, usize)>>>,
    destination: Arc<Mutex<Option<(usize, usize)>>>,
    carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    ressource_transportee: Arc<Mutex<Option<TypeCase>>>,
}

impl Collecteur {
    pub fn new(
        x: usize,
        y: usize,
        carte_connue: Arc<Mutex<Vec<Vec<TypeCase>>>>,
        base_ref: Arc<Mutex<Base>>,
    ) -> Self {
        trace!(
            "[COLLECTEUR] Initialisation du collecteur à la position ({}, {})",
            x,
            y
        );
        let collecteur = Collecteur {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
            chemin: Arc::new(Mutex::new(Vec::new())),
            destination: Arc::new(Mutex::new(None)),
            carte_connue: Arc::clone(&carte_connue),
            ressource_transportee: Arc::new(Mutex::new(None)),
        };

        let pos_x = Arc::clone(&collecteur.position_x);
        let pos_y = Arc::clone(&collecteur.position_y);
        let at_base = Arc::clone(&collecteur.at_base);
        let chemin = Arc::clone(&collecteur.chemin);
        let ressource = Arc::clone(&collecteur.ressource_transportee);
        let destination = Arc::clone(&collecteur.destination);
        let carte_connue = Arc::clone(&collecteur.carte_connue);
        let base = Arc::clone(&base_ref);

        thread::spawn(move || loop {
            let (x, y, is_at_base) = {
                let x = *pos_x.lock().unwrap();
                let y = *pos_y.lock().unwrap();
                let is_at_base = *at_base.lock().unwrap();
                (x, y, is_at_base)
            };

            // Gestion du dépôt de ressources
            if is_at_base {
                let ressource_deposee = {
                    let mut res_trans = ressource.lock().unwrap();
                    res_trans.take()
                };

                if let Some(ressource_type) = ressource_deposee {
                    trace!("[COLLECTEUR] robot.rs l257");
                    let base_guard = base.lock().unwrap();
                    base_guard.ajouter_ressource(ressource_type);
                    info!("[COLLECTEUR] Dépôt de ressource à la base");
                }
            }

            // Calcul du prochain déplacement
            let mut nouveau_chemin = None;
            let mut nouvelle_destination = None;

            // Si on est à la base et qu'on n'a pas de chemin
            if is_at_base {
                if chemin.lock().unwrap().is_empty() {
                    if let Ok(dest) = destination.lock() {
                        if let Some(dest_coords) = *dest {
                            if let Ok(carte) = carte_connue.lock() {
                                if let Some(path) = Self::a_star_static((x, y), dest_coords, &carte)
                                {
                                    nouveau_chemin = Some(path);
                                    debug!(
                                        "[COLLECTEUR] Nouveau chemin calculé vers ({}, {})",
                                        dest_coords.0, dest_coords.1
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                // Vérification de l'arrivée à destination
                let dest_atteinte = {
                    if let Ok(dest) = destination.lock() {
                        if let Some(dest_coords) = *dest {
                            x == dest_coords.0 && y == dest_coords.1
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if dest_atteinte {
                    // Vérifier d'abord le type de case
                    let case_type = {
                        if let Ok(carte) = carte_connue.lock() {
                            carte.get(y).and_then(|row| row.get(x)).map(|c| c.clone())
                        } else {
                            None
                        }
                    };

                    // Ensuite gérer la ressource si nécessaire
                    let mut ressource_prise = false;
                    if let Some(case) = case_type {
                        match case {
                            TypeCase::Energie | TypeCase::Minerais | TypeCase::Science => {
                                let mut res_trans = ressource.lock().unwrap();
                                if res_trans.is_none() {
                                    *res_trans = Some(case.clone());
                                    ressource_prise = true;
                                    info!("[COLLECTEUR] Ressource prise: {:?}", case);
                                }
                            }
                            _ => {}
                        }
                    }

                    if ressource_prise {
                        // Calculer le chemin de retour sans maintenir le lock sur la ressource
                        trace!("[COLLECTEUR] robot.rs l329");
                        let base_pos = if let Ok(base_guard) = base.lock() {
                            Some((base_guard.position_x, base_guard.position_y))
                        } else {
                            None
                        };

                        if let Some((base_x, base_y)) = base_pos {
                            if let Ok(carte) = carte_connue.lock() {
                                if let Some(new_path) =
                                    Self::a_star_static((x, y), (base_x, base_y), &carte)
                                {
                                    nouveau_chemin = Some(new_path);
                                    nouvelle_destination = Some(None);
                                    debug!("[COLLECTEUR] Chemin de retour calculé vers la base");
                                }
                            }
                        }
                    }
                }

                // Si on n'a pas de chemin et qu'on n'est pas à la base
                if chemin.lock().unwrap().is_empty() && !dest_atteinte {
                    trace!("[COLLECTEUR] robot.rs l352");
                    if let Ok(base_guard) = base.lock() {
                        let base_pos = (base_guard.position_x, base_guard.position_y);
                        if let Ok(carte) = carte_connue.lock() {
                            if let Some(new_path) = Self::a_star_static((x, y), base_pos, &carte) {
                                nouveau_chemin = Some(new_path);
                                nouvelle_destination = Some(None);
                                debug!("[COLLECTEUR] Chemin calculé vers la base");
                            }
                        }
                    }
                }
            }

            // Application des changements calculés
            if let Some(new_path) = nouveau_chemin {
                let mut chemin_guard = chemin.lock().unwrap();
                *chemin_guard = new_path;
            }

            if let Some(new_dest) = nouvelle_destination {
                let mut dest_guard = destination.lock().unwrap();
                *dest_guard = new_dest;
            }

            // Déplacement
            let mut movement_done = false;
            if let Ok(mut chemin_guard) = chemin.lock() {
                if !chemin_guard.is_empty() {
                    let (next_x, next_y) = chemin_guard[0];
                    let can_move = {
                        if let Ok(carte) = carte_connue.lock() {
                            carte
                                .get(next_y)
                                .and_then(|row| row.get(next_x))
                                .map(|case| *case != TypeCase::Mur)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    };

                    if can_move {
                        chemin_guard.remove(0);
                        *pos_x.lock().unwrap() = next_x;
                        *pos_y.lock().unwrap() = next_y;
                        movement_done = true;
                        debug!("[COLLECTEUR] Déplacement vers ({}, {})", next_x, next_y);

                        // Mise à jour de at_base après le déplacement
                        trace!("[COLLECTEUR] robot.rs l402");
                        let is_at_base = {
                            let base_guard = base.lock().unwrap();
                            next_x == base_guard.position_x && next_y == base_guard.position_y
                        };
                        *at_base.lock().unwrap() = is_at_base;
                    } else {
                        // Mur détecté, on vide le chemin
                        chemin_guard.clear();
                        warn!("[COLLECTEUR] Mur détecté, chemin vidé");
                    }
                }
            }

            // Mise à jour de la vision (seulement si on s'est déplacé)
            if movement_done {
                trace!("[COLLECTEUR] robot.rs l417");
                let (new_x, new_y) = (*pos_x.lock().unwrap(), *pos_y.lock().unwrap());
                let case_type = {
                    let base_guard = base.lock().unwrap();
                    let carte_reelle = base_guard.carte_reelle.lock().unwrap();
                    carte_reelle.get(new_y).and_then(|row| row.get(new_x)).cloned()
                };

                if let Some(case_type) = case_type {
                    let base_guard = base.lock().unwrap();
                    base_guard.mettre_a_jour_carte(new_x, new_y, case_type);
                    debug!(
                        "[COLLECTEUR] Mise à jour de la carte à ({}, {})",
                        new_x, new_y
                    );
                }
            }

            thread::sleep(Duration::from_millis(100));
        });
        collecteur
    }

    fn a_star_static(
        start: (usize, usize),
        goal: (usize, usize),
        map: &Vec<Vec<TypeCase>>,
    ) -> Option<Vec<(usize, usize)>> {
        trace!(
            "[COLLECTEUR] Calcul du chemin avec A* de ({}, {}) à ({}, {})",
            start.0,
            start.1,
            goal.0,
            goal.1
        );
        use std::cmp::Ordering;
        use std::collections::{BinaryHeap, HashMap};

        #[derive(Copy, Clone, Eq, PartialEq)]
        struct Node {
            position: (usize, usize),
            f_score: usize,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                other.f_score.cmp(&self.f_score)
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let height = map.len();
        let width = map[0].len();

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();

        // Fonction heuristique (distance de Manhattan)
        let h = |pos: (usize, usize)| -> usize {
            ((pos.0 as i32 - goal.0 as i32).abs() + (pos.1 as i32 - goal.1 as i32).abs()) as usize
        };

        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            f_score: h(start),
        });

        while let Some(current) = open_set.pop() {
            let current_pos = current.position;

            if current_pos == goal {
                // Reconstruction du chemin
                let mut path = vec![goal];
                let mut current = goal;
                while let Some(&prev) = came_from.get(&current) {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                // On retire le point de départ car on y est déjà
                if !path.is_empty() {
                    path.remove(0);
                }
                return Some(path);
            }

            let current_g = *g_score.get(&current_pos).unwrap();

            // Vérification des voisins (4 directions)
            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
            for (dx, dy) in directions.iter() {
                let new_x = current_pos.0 as i32 + dx;
                let new_y = current_pos.1 as i32 + dy;

                if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                    let neighbor = (new_x as usize, new_y as usize);

                    // Vérifier si la case n'est pas un mur
                    if map[neighbor.1][neighbor.0] != TypeCase::Mur {
                        let tentative_g = current_g + 1;

                        if !g_score.contains_key(&neighbor)
                            || tentative_g < *g_score.get(&neighbor).unwrap()
                        {
                            came_from.insert(neighbor, current_pos);
                            g_score.insert(neighbor, tentative_g);
                            open_set.push(Node {
                                position: neighbor,
                                f_score: tentative_g + h(neighbor),
                            });
                        }
                    }
                }
            }
        }

        None // Aucun chemin trouvé
    }

    pub fn set_destination(&self, x: usize, y: usize) {
        trace!("[COLLECTEUR] Définition de la destination à ({}, {})", x, y);
        if let Ok(mut dest) = self.destination.lock() {
            *dest = Some((x, y));
        }
    }

    pub fn ramasser_ressource(&self, ressource: TypeCase) -> bool {
        trace!(
            "[COLLECTEUR] Tentative de ramassage de la ressource: {:?}",
            ressource
        );
        if let Ok(mut res_trans) = self.ressource_transportee.lock() {
            if res_trans.is_none() {
                match ressource {
                    TypeCase::Energie | TypeCase::Minerais | TypeCase::Science => {
                        *res_trans = Some(ressource);
                        return true;
                    }
                    _ => return false,
                }
            }
        }
        false
    }

    pub fn deposer_ressource(&self) -> Option<TypeCase> {
        trace!("[COLLECTEUR] Tentative de dépôt de la ressource");
        if let Ok(mut res_trans) = self.ressource_transportee.lock() {
            let ressource = res_trans.take();
            ressource
        } else {
            None
        }
    }

    pub fn get_destination(&self) -> Option<(usize, usize)> {
        if let Ok(dest) = self.destination.lock() {
            *dest
        } else {
            None
        }
    }
}

impl Robot for Collecteur {
    fn next_move(&self) {}

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

    fn as_any(&self) -> &dyn Any {
        self
    }
}
