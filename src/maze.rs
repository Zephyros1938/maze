use rand::Rng as _;

pub struct Maze {
    x: usize,
    y: usize,
    grid: Vec<Vec<u8>>, // 0 for walls, 1 for passages
    walls: Vec<(usize, usize)>,
    start: (usize, usize),
    rng: rand::rngs::ThreadRng,
}

pub trait MazeTrait {
    fn new(x: usize, y: usize) -> Self;

    fn generate(&mut self);

    fn insert_at(&mut self, x: usize, y: usize, c: u8);

    fn get_at(&self, nx: usize, ny: usize) -> u8;

    fn swap_at(&mut self, xa: usize, ya: usize, xb: usize, yb: usize);
}

impl MazeTrait for Maze {
    fn new(x: usize, y: usize) -> Self {
        let mut grid = vec![vec![0; x]; y];
        let mut rng = rand::rng();
        let start_x = rng.random_range(1..x - 1);
        let start_y = rng.random_range(1..y - 1);

        grid[start_y][start_x] = 1;
        let mut walls = vec![];
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = (start_x as isize + dx) as usize;
            let ny = (start_y as isize + dy) as usize;
            if nx > 0 && ny > 0 && nx < x - 1 && ny < y - 1 {
                walls.push((nx, ny));
            }
        }

        Self {
            x,
            y,
            grid,
            walls,
            start: (start_x, start_y),
            rng,
        }
    }

    fn generate(&mut self) {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        while !self.walls.is_empty() {
            // Select a random wall from the list
            let idx = self.rng.random_range(0..self.walls.len());
            let (wx, wy) = self.walls.swap_remove(idx);

            // Count how many adjacent cells are passages
            let mut adjacent = 0;
            for &(dx, dy) in directions.iter() {
                let nx = (wx as isize + dx) as usize;
                let ny = (wy as isize + dy) as usize;
                if nx < self.x && ny < self.y && self.grid[ny][nx] == 1 {
                    adjacent += 1;
                }
            }

            // If the wall divides one passage cell from an unvisited cell, convert it
            if adjacent == 1 {
                self.grid[wy][wx] = 1;

                // Add neighboring walls of the newly made passage cell
                for &(dx, dy) in directions.iter() {
                    let nx = (wx as isize + dx) as usize;
                    let ny = (wy as isize + dy) as usize;
                    if nx > 0
                        && ny > 0
                        && nx < self.x - 1
                        && ny < self.y - 1
                        && self.get_at(nx, ny) == 0
                    {
                        self.walls.push((nx, ny));
                    }
                }
            }
        }
        self.insert_at(self.start.1, self.start.0, 2);
    }

    fn insert_at(&mut self, x: usize, y: usize, c: u8) {
        if x > self.x {
            panic!("x {} Was above maze max x {}", x, self.x);
        }
        if y > self.y {
            panic!("y {} Was above maze max y {}", y, self.y);
        }
        self.grid[x][y] = c;
    }

    fn get_at(&self, nx: usize, ny: usize) -> u8 {
        return self.grid[ny][nx];
    }

    fn swap_at(&mut self, xa: usize, ya: usize, xb: usize, yb: usize) {
        if xa > self.x {
            panic!("xa {} Was above maze max x {}", xa, self.x);
        }
        if ya > self.y {
            panic!("ya {} Was above maze max y {}", ya, self.y);
        }
        if xb > self.x {
            panic!("xb {} Was above maze max x {}", xb, self.x);
        }
        if yb > self.y {
            panic!("yb {} Was above maze max y {}", yb, self.y);
        }
        let a: u8 = self.grid[xa][ya];
        let b: u8 = self.grid[xb][yb];

        self.grid[xa][ya] = b;
        self.grid[xb][yb] = a;
    }
}

impl std::fmt::Display for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for row in &self.grid {
            for &cell in row {
                write!(
                    f,
                    "{}",
                    match cell {
                        0 => "##",
                        1 => "__",
                        2 => "[]",
                        _ => "??",
                    }
                )?;
            }
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}
