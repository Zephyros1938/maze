use std::{
    char,
    collections::HashMap,
    fmt,
    io::{Stdin, Stdout, Write, stdin, stdout},
    ops::DerefMut,
    sync::Arc,
};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

pub struct Terminal {
    pub startmessage: String,
    pub stdin: Stdin,
    pub stdout: RawTerminal<Stdout>,
    key_actions: HashMap<Key, Box<dyn FnMut()>>,
    run_actions: HashMap<(bool, u8), Box<dyn FnMut()>>,
}

pub trait TerminalTrait {
    fn new(startmessage: String) -> Self;
    fn run(self);

    // Visual

    // Key Actions

    fn add_key_action<F: FnMut() + 'static>(&mut self, k: char, func: F);
    fn add_key_actions(&mut self, h: HashMap<Key, impl FnMut() + 'static>);
    fn rem_key_action(&mut self, k: char);
    fn rem_key_actions(&mut self, h: Vec<char>);

    // Run Actions

    fn add_run_action<F: FnMut() + 'static>(&mut self, enabled: bool, id: u8, func: F);
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), impl FnMut() + 'static>);
    fn rem_run_action(&mut self, id: u8);
    fn rem_run_actions(&mut self, h: Vec<char>);
}

impl TerminalTrait for Terminal {
    fn new(startmessage: String) -> Self {
        Self {
            startmessage,
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            key_actions: HashMap::new(),
            run_actions: HashMap::new(),
        }
    }
    fn run(mut self) {
        write!(
            self.stdout,
            r#"{}{}{}"#,
            self.startmessage,
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
        self.stdout.flush().unwrap();

        //detecting keydown events
        for c in self.stdin.keys() {
            //clearing the screen and going to top left corner
            write!(
                self.stdout,
                "{:}{:}",
                termion::cursor::Goto(1, 1),
                termion::clear::All
            )
            .unwrap();

            //i reckon this speaks for itself
            if let Ok(key) = c {
                if let Some(action) = self.key_actions.get_mut(&key) {
                    action();
                }
            }

            for (enabled, action) in self.run_actions.iter_mut() {
                if enabled.0 == true {
                    action()
                };
            }

            self.stdout.flush().unwrap();
        }
    }
    // Visual

    // Key Actions

    fn add_key_action<F: FnMut() + 'static>(&mut self, k: char, func: F) {
        self.key_actions.insert(Key::Char(k), Box::new(func));
    }
    fn add_key_actions(&mut self, h: HashMap<Key, impl FnMut() + 'static>) {
        for k in h {
            self.key_actions.insert(k.0, Box::new(k.1));
        }
    }
    fn rem_key_action(&mut self, k: char) {
        self.key_actions.remove(&Key::Char(k));
    }
    fn rem_key_actions(&mut self, h: Vec<char>) {
        for k in h {
            self.rem_key_action(k);
        }
    }

    // Run Actions

    fn add_run_action<F: FnMut() + 'static>(&mut self, enabled: bool, id: u8, func: F) {
        self.run_actions.insert((enabled, id), Box::new(func));
    }
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), impl FnMut() + 'static>) {
        for k in h {
            self.run_actions.insert(k.0, Box::new(k.1));
        }
    }
    fn rem_run_action(&mut self, id: u8) {
        self.run_actions.remove(&(true || false, id));
    }
    fn rem_run_actions(&mut self, h: Vec<char>) {
        for k in h {
            self.rem_key_action(k);
        }
    }
}

const SCREEN_BLANK_CHAR: char = '_';
const SCREEN_PIXEL_WIDTH: usize = 2;

pub struct TerminalScreen {
    pixel_front_buffer: Vec<Vec<String>>,
    pixel_back_buffer: Vec<Vec<String>>,
    dimensions: (usize, usize),
    pub startmessage: String,
    pub stdin: Stdin,
    pub stdout: RawTerminal<Stdout>,
    key_actions: HashMap<Key, Arc<dyn Fn()>>,
    run_actions: HashMap<(bool, u8), Arc<dyn Fn()>>,
}

impl TerminalScreen {
    pub fn new(startmessage: String, dimensions: (usize, usize)) -> Self {
        let (cols, rows) = termion::terminal_size().unwrap();
        let cols: usize = cols.into();
        let rows: usize = rows.into();
        let cols = cols / SCREEN_PIXEL_WIDTH;

        if dimensions.0 > cols || dimensions.1 > rows {
            panic!(
                "Dimensions {}x{} exceed terminal size {}x{}",
                dimensions.0, dimensions.1, cols, rows
            );
        };

        let pixel_buffer = if dimensions == (0, 0) {
            vec![vec![String::from(SCREEN_BLANK_CHAR).repeat(SCREEN_PIXEL_WIDTH); cols]; rows]
        } else {
            vec![
                vec![String::from(SCREEN_BLANK_CHAR).repeat(SCREEN_PIXEL_WIDTH); dimensions.0];
                dimensions.1
            ]
        };
        Self {
            pixel_front_buffer: pixel_buffer.clone(),
            pixel_back_buffer: pixel_buffer,
            dimensions,
            startmessage,
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            key_actions: HashMap::new(),
            run_actions: HashMap::new(),
        }
    }

    pub fn set_pixel_at(mut self, x: usize, y: usize, c: char) {
        self.pixel_back_buffer[x][y] = String::from(c).repeat(SCREEN_PIXEL_WIDTH);
    }

    pub fn draw(mut self) {
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                write!(self.stdout, r#"{}{}"#, termion::cursor::Goto(x, y), x).unwrap();
            }
        }
    }
}

impl fmt::Display for TerminalScreen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in &self.pixel_front_buffer {
            write!(f, "\r\n")?;
            for x in y {
                write!(f, "{}", x)?;
            }
        }
        Ok(())
    }
}

pub trait TerminalScreenTrait {
    fn run(self);

    fn add_key_action(&mut self, k: char, func: Arc<dyn Fn()>);
    fn add_key_actions(&mut self, h: HashMap<Key, Arc<dyn Fn()>>);
    fn rem_key_action(&mut self, k: char);
    fn rem_key_actions(&mut self, h: Vec<char>);

    // Run Actions

    fn add_run_action(&mut self, enabled: bool, id: u8, func: Arc<dyn Fn()>);
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), Arc<dyn Fn()>>);
    fn rem_run_action(&mut self, id: u8);
    fn rem_run_actions(&mut self, h: Vec<char>);
}

impl TerminalScreenTrait for TerminalScreen {
    fn run(mut self) {
        write!(
            self.stdout,
            r#"{}{}{}"#,
            self.startmessage,
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
        self.stdout.flush().unwrap();

        //detecting keydown events
        for c in self.stdin.keys() {
            //clearing the screen and going to top left corner
            write!(
                self.stdout,
                "{:}{:}",
                termion::cursor::Goto(1, 1),
                termion::clear::All
            )
            .unwrap();

            //i reckon this speaks for itself
            if let Ok(key) = c {
                if let Some(action) = self.key_actions.get(&key) {
                    action()
                }
            }

            for (enabled, action) in self.run_actions.iter() {
                if enabled.0 == true {
                    action()
                };
            }

            self.pixel_front_buffer = self.pixel_back_buffer.clone();

            self.stdout.flush().unwrap();
        }
    }

    fn add_key_action(&mut self, k: char, func: Arc<dyn Fn()>) {
        self.key_actions.insert(Key::Char(k), func);
    }
    fn add_key_actions(&mut self, h: HashMap<Key, Arc<dyn Fn()>>) {
        for k in h {
            self.key_actions.insert(k.0, k.1);
        }
    }
    fn rem_key_action(&mut self, k: char) {
        self.key_actions.remove(&Key::Char(k));
    }
    fn rem_key_actions(&mut self, h: Vec<char>) {
        for k in h {
            self.rem_key_action(k);
        }
    }

    // Run Actions

    fn add_run_action(&mut self, enabled: bool, id: u8, func: Arc<dyn Fn()>) {
        self.run_actions.insert((enabled, id), func);
    }
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), Arc<dyn Fn()>>) {
        for k in h {
            self.run_actions.insert(k.0, k.1);
        }
    }
    fn rem_run_action(&mut self, id: u8) {
        self.run_actions.remove(&(true || false, id));
    }
    fn rem_run_actions(&mut self, h: Vec<char>) {
        for k in h {
            self.rem_key_action(k);
        }
    }
}
