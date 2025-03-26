use crate::base::Base;
use crate::generation::TypeCase;
use crate::pathfinding::find_path;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;

pub trait Robot: Send {
    fn get_type(&self) -> TypeCase;
    fn get_position_x(&self) -> usize;
    fn get_position_y(&self) -> usize;
}

pub struct Explorer {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
}

impl Explorer {
    pub fn new(
        map_width: usize,
        map_height: usize,
        x: usize,
        y: usize,
        base_ref: Arc<Mutex<Base>>,
    ) -> Self {
        let explorateur = Explorer {
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

                // Evaluate each possible direction
                let mut directions = vec![];
                let possible_moves = [
                    (0, -1, 0), // Up
                    (0, 1, 1),  // Down
                    (-1, 0, 2), // Left
                    (1, 0, 3),  // Right
                ];

                if let Ok(base) = base.lock() {
                    if let Ok(known_map) = base.known_map.lock() {
                        if let Ok(real_map) = base.real_map.lock() {
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

                                    // Add a priority logic for unknown cases that will increase the probability of moving towards them
                                    if let Some(row) = real_map.get(new_y) {
                                        if let Some(case_type) = row.get(new_x) {
                                            if *case_type != TypeCase::Wall {
                                                let weight = if let Some(known_row) =
                                                    known_map.get(new_y)
                                                {
                                                    if let Some(known_type) = known_row.get(new_x) {
                                                        if *known_type == TypeCase::Unknown {
                                                            3 // More weight to unknown cases
                                                        } else {
                                                            1
                                                        }
                                                    } else {
                                                        3
                                                    }
                                                } else {
                                                    3
                                                };

                                                // Add the direction weight times to the possible directions
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

                // If no direction is possible, do not move
                if !directions.is_empty() {
                    let direction = directions[rng.random_range(0..directions.len())];
                    let (dx, dy) = match direction {
                        0 => (0, -1), // Up
                        1 => (0, 1),  // Down
                        2 => (-1, 0), // Left
                        _ => (1, 0),  // Right
                    };

                    let new_x = (x as i32 + dx) as usize;
                    let new_y = (y as i32 + dy) as usize;

                    *position_x.lock().unwrap() = new_x;
                    *position_y.lock().unwrap() = new_y;
                }

                // Update the known map with the new vision
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

                                    if let Ok(real_map) = base.real_map.lock() {
                                        if let Some(case_type) =
                                            real_map.get(new_y).and_then(|row| row.get(new_x))
                                        {
                                            let case_type = case_type.clone();
                                            base.update_map(new_x, new_y, case_type);
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

impl Robot for Explorer {
    fn get_type(&self) -> TypeCase {
        TypeCase::Explorer
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }
}

pub struct Collector {
    position_x: Arc<Mutex<usize>>,
    position_y: Arc<Mutex<usize>>,
    at_base: Arc<Mutex<bool>>,
    path: Arc<Mutex<Vec<(usize, usize)>>>,
    collected_resource: Arc<Mutex<Option<TypeCase>>>,
    destination: Arc<Mutex<Option<(usize, usize)>>>,
}

impl Collector {
    pub fn new(x: usize, y: usize, base_ref: Arc<Mutex<Base>>) -> Self {
        let collecteur = Collector {
            position_x: Arc::new(Mutex::new(x)),
            position_y: Arc::new(Mutex::new(y)),
            at_base: Arc::new(Mutex::new(true)),
            path: Arc::new(Mutex::new(Vec::new())),
            collected_resource: Arc::new(Mutex::new(None)),
            destination: Arc::new(Mutex::new(None)),
        };

        let position_x = Arc::clone(&collecteur.position_x);
        let position_y = Arc::clone(&collecteur.position_y);
        let path = Arc::clone(&collecteur.path);
        let at_base = Arc::clone(&collecteur.at_base);
        let collected_resource = Arc::clone(&collecteur.collected_resource);
        let destination = Arc::clone(&collecteur.destination);
        let base = Arc::clone(&base_ref);

        thread::spawn(move || {
            loop {
                if let Ok(base_guard) = base.lock() {
                    let curr_x = *position_x.lock().unwrap();
                    let curr_y = *position_y.lock().unwrap();
                    let mut path_guard = path.lock().unwrap();
                    let is_at_base =
                        curr_x == base_guard.position_x && curr_y == base_guard.position_y;
                    *at_base.lock().unwrap() = is_at_base;
                    let has_resource = collected_resource.lock().unwrap().is_some();

                    // If the robot is at the base and has no resource, look for a new destination
                    if is_at_base && !has_resource && path_guard.is_empty() {
                        if let Some((target_x, target_y)) = base_guard.next_resource() {
                            if let Ok(known_map) = base_guard.known_map.lock() {
                                if let Some(new_path) =
                                    find_path((target_x, target_y), (curr_x, curr_y), &known_map)
                                {
                                    *path_guard = new_path;
                                    *destination.lock().unwrap() = Some((target_x, target_y));
                                }
                            }
                        }
                    }
                    // If the robot has a resource and is at the base, drop it
                    else if is_at_base && has_resource {
                        if let Some(resource) = collected_resource.lock().unwrap().clone() {
                            base_guard.add_resource(resource);
                        }
                        *collected_resource.lock().unwrap() = None;
                    }
                    // If the robot has a destination and is not on a path
                    else if path_guard.is_empty() {
                        if let Some((target_x, target_y)) = *destination.lock().unwrap() {
                            // if the robot is on the target, collect the resource
                            if curr_x == target_x && curr_y == target_y {
                                // collect the resource
                                if !has_resource {
                                    if let Ok(mut map) = base_guard.real_map.lock() {
                                        let resource = map[curr_y][curr_x].clone();
                                        *collected_resource.lock().unwrap() =
                                            Some(resource.clone());

                                        //Update the map
                                        map[curr_y][curr_x] = TypeCase::Void;
                                        if let Ok(mut known_map) = base_guard.known_map.lock() {
                                            known_map[curr_y][curr_x] = TypeCase::Void;
                                        }

                                        base_guard.release_resource(curr_x, curr_y);

                                        // Look for a new destination
                                        if let Ok(known_map) = base_guard.known_map.lock() {
                                            if let Some(new_path) = find_path(
                                                (base_guard.position_x, base_guard.position_y),
                                                (curr_x, curr_y),
                                                &known_map,
                                            ) {
                                                *path_guard = new_path;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // If the robot is on a path, follow the path
                    else if let Some((next_x, next_y)) = path_guard.pop() {
                        *position_x.lock().unwrap() = next_x;
                        *position_y.lock().unwrap() = next_y;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });

        collecteur
    }
}

impl Robot for Collector {
    fn get_type(&self) -> TypeCase {
        TypeCase::Collector
    }

    fn get_position_x(&self) -> usize {
        *self.position_x.lock().unwrap()
    }

    fn get_position_y(&self) -> usize {
        *self.position_y.lock().unwrap()
    }
}
