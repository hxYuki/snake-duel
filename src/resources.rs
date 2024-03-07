use std::time::Duration;

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Cell {
    Empty,
    Food,
    Head(Entity),
    Body(Entity, IVec2),
    Wall,
}

#[derive(Resource, Debug)]
pub(crate) struct Grid {
    pub(crate) height: usize,
    pub(crate) width: usize,
    pub(crate) grid: Vec<Cell>,
}

impl Grid {
    pub fn new(height: usize, width: usize) -> Self {
        let grid = vec![Cell::Empty; height * width];
        Self {
            height,
            width,
            grid,
        }
    }

    pub fn get(&self, IVec2 { x, y }: IVec2) -> Option<Cell> {
        let (x, y) = (x as usize, y as usize);
        if y < self.height && x < self.width {
            Some(self.grid[y * self.width + x])
        } else {
            None
        }
    }
    pub fn find_around(&self, IVec2 { x, y }: IVec2, cell: Cell) -> Option<IVec2> {
        let (x, y) = (x as usize, y as usize);
        let mut result = None;
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                let (i, j) = (i + x as i32, j + y as i32);
                if i < 0 || j < 0 || i >= self.width as i32 || j >= self.height as i32 {
                    continue;
                }
                if self.get((i, j).into()) == Some(cell) {
                    result = Some((i, j).into());
                    break;
                }
            }
        }
        result
    }

    pub fn set(&mut self, IVec2 { x, y }: IVec2, cell: Cell) {
        let (y, x) = (y as usize, x as usize);
        if y < self.height && x < self.width {
            self.grid[y * self.width + x] = cell;
        }
    }

    pub fn build_wall(&mut self) {
        for i in 0..self.width {
            for j in 0..self.height {
                if i == 0 || j == 0 || i == self.width - 1 || j == self.height - 1 {
                    let (i, j) = (i as i32, j as i32);
                    self.set((i, j).into(), Cell::Wall);
                }
            }
        }
    }
    pub fn print_grid(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                let c = self.get((j as i32, i as i32).into()).unwrap();
                let c = match c {
                    Cell::Empty => " ",
                    Cell::Food => "F",
                    Cell::Head(_) => "H",
                    Cell::Body(_, _) => "B",
                    Cell::Wall => "W",
                };
                print!("{}", c);
            }
            println!();
        }
    }
}

#[derive(Resource, Debug)]
pub(crate) struct FoodPlaceTime {
    many_food_time: Duration,
    no_food_time: Duration,
    pub timer: Timer,
}
impl FoodPlaceTime {
    pub fn reset_long(&mut self) {
        self.timer.set_duration(self.many_food_time);
        self.timer.reset();
    }
    pub fn reset_short(&mut self) {
        self.timer.set_duration(self.no_food_time);
        self.timer.reset();
    }
}

impl Default for FoodPlaceTime {
    fn default() -> Self {
        Self {
            many_food_time: Duration::from_secs(5),
            no_food_time: Duration::from_secs(3),
            timer: Timer::from_seconds(3., TimerMode::Repeating),
        }
    }
}
