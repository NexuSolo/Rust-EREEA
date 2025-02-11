use crate::generation::TypeCase;

pub trait Robot {
    fn next_move(&self);
    fn get_type(&self) -> TypeCase;
    fn get_position_x(&self) -> usize;
    fn get_position_y(&self) -> usize;
    fn is_at_base(&self) -> bool;
    fn communicate(&self);
}

pub struct Explorateur {
    position_x: usize,
    position_y: usize,
    at_base: bool,
}

impl Explorateur {
    pub fn new(x: usize, y: usize) -> Self {
        Explorateur {
            position_x: x,
            position_y: y,
            at_base: true,
        }
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
        self.position_x
    }

    fn get_position_y(&self) -> usize {
        self.position_y
    }

    fn is_at_base(&self) -> bool {
        self.at_base
    }

    fn communicate(&self) {
        // todo
    }
}

pub struct Collecteur {
    position_x: usize,
    position_y: usize,
    at_base: bool,
}

impl Collecteur {
    pub fn new(x: usize, y: usize) -> Self {
        Collecteur {
            position_x: x,
            position_y: y,
            at_base: true,
        }
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
        self.position_x
    }

    fn get_position_y(&self) -> usize {
        self.position_y
    }

    fn is_at_base(&self) -> bool {
        self.at_base
    }

    fn communicate(&self) {
        //todo
    }
}