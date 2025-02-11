use crate::generation::TypeCase;
use crate::robot::{Collecteur, Explorateur, Robot};

pub struct Base {
    pub carte_connue: Vec<Vec<TypeCase>>,
    pub robots_deployes: Vec<Box<dyn Robot>>,
    pub energie: usize,
    pub minerais: usize,
    pub science: usize,
    pub position_x: usize,
    pub position_y: usize,
}

impl Base {
    pub fn new(width: usize, height: usize, position_x: usize, position_y: usize) -> Self {
        let mut base = Base {
            carte_connue: vec![vec![TypeCase::Vide; width]; height],
            robots_deployes: Vec::new(),
            energie: 0,
            minerais: 0,
            science: 0,
            position_x,
            position_y,
        };
        base.ajouter_robot(Box::new(Explorateur::new(position_x, position_y)));
        base.ajouter_robot(Box::new(Collecteur::new(position_x, position_y)));
        base
    }

    pub fn ajouter_robot(&mut self, robot: Box<dyn Robot>) {
        self.robots_deployes.push(robot);
    }

    pub fn ajouter_ressource(&mut self, ressource: TypeCase) {
        match ressource {
            TypeCase::Energie => self.energie += 1,
            TypeCase::Minerais => self.minerais += 1,
            TypeCase::Science => self.science += 1,
            _ => (),
        }
    }

    pub fn mettre_a_jour_carte(&mut self, x: usize, y: usize, case: TypeCase) {
        if x < self.carte_connue[0].len() && y < self.carte_connue.len() {
            self.carte_connue[y][x] = case;
        }
    }
}
