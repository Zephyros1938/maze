use std::{
    char,
    collections::HashMap,
    io::{Stdin, Stdout, Write, stdin, stdout},
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
    pub key_actions: HashMap<Key, Box<dyn FnMut()>>,
}
impl Terminal {
    pub fn new(startmessage: String) -> Self {
        Self {
            startmessage,
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            key_actions: HashMap::new(),
        }
    }
    pub fn run(mut self) {
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
            if let Some(action) = self.key_actions.get_mut(&c.unwrap()) {
                action();
            }

            self.stdout.flush().unwrap();
        }
    }

    // Screen

    // Key Actions

    pub fn add_key_action<F: FnMut() + 'static>(&mut self, k: char, func: F) {
        self.key_actions.insert(Key::Char(k), Box::new(func));
    }
    pub fn add_key_actions(&mut self, h: HashMap<Key, impl FnMut() + 'static>) {
        for k in h {
            self.key_actions.insert(k.0, Box::new(k.1));
        }
    }
    pub fn rem_key_action(&mut self, k: char) {
        self.key_actions.remove(&Key::Char(k));
    }
    pub fn rem_key_actions(&mut self, h: Vec<char>) {
        for k in h {
            self.rem_key_action(k);
        }
    }
}

pub struct Screen {
    base: Terminal,
    pixel_buffer: Vec<Vec<char>>,
    dimensions: (usize, usize),
}

impl Screen {
    pub fn new(terminal: Terminal, dimensions: (usize, usize)) -> Self {
        let cols = std::env::var("COLUMNS")
            .unwrap_or(String::from("80"))
            .parse::<usize>()
            .unwrap();
        let rows = std::env::var("LINES")
            .unwrap_or(String::from("24"))
            .parse::<usize>()
            .unwrap();

        if dimensions.0 > cols || dimensions.1 > rows {
            panic!(
                "Dimensions {}x{} exceed terminal size {}x{}",
                dimensions.0, dimensions.1, cols, rows
            );
        }

        Self {
            base: terminal,
            pixel_buffer: vec![vec![' '; dimensions.0]; dimensions.1],
            dimensions,
        }
    }
}
