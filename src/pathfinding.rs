use crate::generation::TypeCase;
use seastar::{astar, Grid, Point};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub usize, pub usize);

pub fn find_path(
    start: (usize, usize),
    goal: (usize, usize),
    map: &Vec<Vec<TypeCase>>,
) -> Option<Vec<(usize, usize)>> {
    let width = map[0].len();
    let height = map.len();

    // Create the grid for seastar (true = wall, false = possible path)
    let mut grid_data = vec![vec![false; width]; height];
    for y in 0..height {
        for x in 0..width {
            grid_data[y][x] = map[y][x] == TypeCase::Wall || map[y][x] == TypeCase::Unknown;
        }
    }

    let grid = Grid::from_2d(grid_data);
    let start_point = Point::new(start.0 as isize, start.1 as isize);
    let goal_point = Point::new(goal.0 as isize, goal.1 as isize);

    // Use of the A* algorithm
    astar(&grid, start_point, goal_point).map(|path| {
        path.into_iter()
            .map(|point| (point.x as usize, point.y as usize))
            .collect()
    })
}
