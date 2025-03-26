use crate::generation::TypeCase;
use crate::robot::{Collector, Explorer, Robot};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Sort resource by priority level
#[derive(Clone, Debug, Eq)]
struct PrioritizedResource {
    x: usize,
    y: usize,
    distance: usize,
    priority_level: usize,
}

// Implentation of necessary traits for the priority queue
impl PartialEq for PrioritizedResource {
    fn eq(&self, other: &Self) -> bool {
        self.priority_level == other.priority_level && self.distance == other.distance
    }
}

// Reverse the order so that the most prioritized and closest resources are at the beginning
impl PartialOrd for PrioritizedResource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedResource {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare the priority level first
        match self.priority_level.cmp(&other.priority_level) {
            Ordering::Equal => {}
            ordering => return ordering,
        }

        // Then compare the distance
        other.distance.cmp(&self.distance)
    }
}

pub struct Base {
    pub real_map: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub known_map: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    pub deployed_robots: Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
    pub energy: Arc<Mutex<usize>>,
    pub ore: Arc<Mutex<usize>>,
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
        real_map: Arc<Mutex<Vec<Vec<TypeCase>>>>,
        known_map: Arc<Mutex<Vec<Vec<TypeCase>>>>,
    ) -> Arc<Mutex<Self>> {
        let deployed_robots = Arc::new(Mutex::new(Vec::new()));
        let energy = Arc::new(Mutex::new(0));
        let ore = Arc::new(Mutex::new(0));
        let science = Arc::new(Mutex::new(0));
        let reserved_resources = Arc::new(Mutex::new(HashSet::new()));

        let base = Arc::new(Mutex::new(Base {
            real_map: Arc::clone(&real_map),
            known_map: Arc::clone(&known_map),
            deployed_robots: Arc::clone(&deployed_robots),
            energy: Arc::clone(&energy),
            ore: Arc::clone(&ore),
            science: Arc::clone(&science),
            position_x,
            position_y,
            reserved_resources,
        }));

        // Add initial robots
        if let Ok(mut base_guard) = base.lock() {
            base_guard.add_robot(Box::new(Explorer::new(
                width,
                height,
                position_x,
                position_y,
                Arc::clone(&base),
            )));
            base_guard.add_robot(Box::new(Collector::new(
                position_x,
                position_y,
                Arc::clone(&base),
            )));
        }

        base
    }

    pub fn start_base_thread(base: Arc<Mutex<Base>>, map_width: usize, map_height: usize) {
        thread::spawn(move || loop {
            let mut explorers_count = 0;
            let mut collectors_count = 0;
            let mut energy = 0;
            let mut ore = 0;
            let mut science = 0;
            let mut position_x = 0;
            let mut position_y = 0;

            // Init variables
            if let Ok(base_guard) = base.lock() {
                position_x = base_guard.position_x;
                position_y = base_guard.position_y;

                if let Ok(robots) = base_guard.deployed_robots.lock() {
                    for robot in robots.iter() {
                        match robot.get_type() {
                            TypeCase::Explorer => explorers_count += 1,
                            TypeCase::Collector => collectors_count += 1,
                            _ => {}
                        }
                    }
                }

                if let Ok(e) = base_guard.energy.lock() {
                    energy = *e;
                }
                if let Ok(m) = base_guard.ore.lock() {
                    ore = *m;
                }
                if let Ok(s) = base_guard.science.lock() {
                    science = *s;
                }
            }

            // Calculate the ratio and determine which robot to create
            let current_ratio = if explorers_count == 0 {
                0.0
            } else {
                collectors_count as f32 / explorers_count as f32
            };

            let create_collector = current_ratio < 2.0
                && collectors_count > 0
                && science >= 1
                && ore >= 5
                && energy >= 4;

            let create_explorer = (current_ratio >= 2.0 || explorers_count == 0)
                && science >= 4
                && ore >= 3
                && energy >= 2;

            //Create robots
            if create_collector || create_explorer {
                if let Ok(mut base_guard) = base.lock() {
                    if create_collector {
                        // Ressources pour un collecteur 1 Science, 5 Ore, 4 Energy
                        if let Ok(mut s) = base_guard.science.lock() {
                            *s -= 1;
                        }
                        if let Ok(mut m) = base_guard.ore.lock() {
                            *m -= 5;
                        }
                        if let Ok(mut e) = base_guard.energy.lock() {
                            *e -= 4;
                        }

                        base_guard.add_robot(Box::new(Collector::new(
                            position_x,
                            position_y,
                            Arc::clone(&base),
                        )));
                    } else if create_explorer {
                        // Resources for explorer : 4 Sciences, 3 Ores, 2 Energies
                        if let Ok(mut s) = base_guard.science.lock() {
                            *s -= 4;
                        }
                        if let Ok(mut m) = base_guard.ore.lock() {
                            *m -= 3;
                        }
                        if let Ok(mut e) = base_guard.energy.lock() {
                            *e -= 2;
                        }

                        base_guard.add_robot(Box::new(Explorer::new(
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

    pub fn add_robot(&mut self, robot: Box<dyn Robot + Send>) {
        self.deployed_robots.lock().unwrap().push(robot);
    }

    pub fn update_map(&self, x: usize, y: usize, case: TypeCase) {
        let mut map = self.known_map.lock().unwrap();
        if x < map[0].len() && y < map.len() {
            map[y][x] = case;
        }
    }

    pub fn next_resource(&self) -> Option<(usize, usize)> {
        let known_map = self.known_map.lock().unwrap();
        let energy_count = *self.energy.lock().unwrap();
        let ore_count = *self.ore.lock().unwrap();
        let science_count = *self.science.lock().unwrap();
        let reserved = self.reserved_resources.lock().unwrap();

        let height = known_map.len();
        let width = known_map[0].len();

        // Find the highest resource counter
        let max_resource_count = energy_count.max(ore_count).max(science_count);

        let mut priority_queue: BinaryHeap<PrioritizedResource> = BinaryHeap::new();

        // Explore the entire map to find resources
        for y in 0..height {
            for x in 0..width {
                // Check if the case is not already reserved
                if reserved.contains(&(x, y)) {
                    continue;
                }

                let case = &known_map[y][x];
                match case {
                    TypeCase::Energy | TypeCase::Ore | TypeCase::Science => {
                        // Calculate the distance between the resource and the base
                        let distance =
                            Self::manhattan_distance(self.position_x, self.position_y, x, y);

                        // Calculate the priority level based on the difference with the highest counter
                        let priority_level = match case {
                            TypeCase::Energy => max_resource_count.saturating_sub(energy_count),
                            TypeCase::Ore => max_resource_count.saturating_sub(ore_count),
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

        // Take the most prioritized resource
        if let Some(resource) = priority_queue.pop() {
            drop(known_map);
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

    pub fn add_resource(&self, resource: TypeCase) {
        match resource {
            TypeCase::Energy => {
                if let Ok(mut energy) = self.energy.lock() {
                    *energy += 1;
                }
            }
            TypeCase::Ore => {
                if let Ok(mut ore) = self.ore.lock() {
                    *ore += 1;
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
