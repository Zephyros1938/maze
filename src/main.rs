use rand::rngs::ThreadRng;
use rand::{self, Rng, SeedableRng};
use std::io::{Write, stdin, stdout};
use std::thread::current;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let mut moves = Moveset {
        moves: vec![b'_'; 128],
    };

    let mut maze = Maze::new(9, 9);
    maze.generate();

    println!("{}\n", maze);

    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(
        stdout,
        r#"{}{}Esc to exit"#,
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
    .unwrap();
    stdout.flush().unwrap();

    //detecting keydown events
    for c in stdin.keys() {
        //clearing the screen and going to top left corner
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();

        //i reckon this speaks for itself
        match c.unwrap() {
            Key::Esc => {
                println!("break");
                break;
            }
            Key::Char('W') => moves.insert_front(b'W'),
            Key::Char('A') => moves.insert_front(b'A'),
            Key::Char('S') => moves.insert_front(b'S'),
            Key::Char('D') => moves.insert_front(b'D'),
            Key::Char('w') => moves.insert_back(b'w'),
            Key::Char('a') => moves.insert_back(b'a'),
            Key::Char('s') => moves.insert_back(b's'),
            Key::Char('d') => moves.insert_back(b'd'),
            Key::Char('e') => {
                println!("Moves: {0}", moves);
            }

            _ => (),
        }

        stdout.flush().unwrap();
    }
}

struct Moveset {
    moves: Vec<u8>,
}

impl Moveset {
    fn insert_front(&mut self, i: u8) {
        self.moves.rotate_right(1);
        self.moves[0] = i;
    }

    fn insert_back(&mut self, i: u8) {
        let len = self.moves.len() - 1;
        self.moves.rotate_left(1);
        self.moves[len] = i;
    }
}

impl std::fmt::Display for Moveset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match String::from_utf8(self.moves.to_vec()) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            }
        )
    }
}

struct Maze {
    x: usize,
    y: usize,
    grid: Vec<Vec<u8>>, // 0 for walls, 1 for passages
    walls: Vec<(usize, usize)>,
    rng: rand::rngs::ThreadRng,
}

impl Maze {
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
                        && self.grid[ny][nx] == 0
                    {
                        self.walls.push((nx, ny));
                    }
                }
            }
        }
    }

    fn InsertAt(&mut self, x: usize, y: usize, c: u8) {
        if x > self.x {
            panic!("x {} Was above maze max x {}", x, self.x);
        }
        if y > self.y {
            panic!("y {} Was above maze max y {}", y, self.y);
        }
        self.grid[x][y] = c;
    }

    fn SwapAt(&mut self, xa: usize, ya: usize, xb: usize, yb: usize) {
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
            for cell in row {
                print!("{}", {
                    match cell {
                        0 => "##",
                        1 => "  ",
                        2 => "[]",
                        _ => "??",
                    }
                });
            }
            println!();
        }
        Ok(())
    }
}
