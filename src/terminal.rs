use std::{
    char,
    collections::HashMap,
    fmt,
    io::{Stdin, Stdout, Write, stdin, stdout},
    sync::{Arc, mpsc::TryRecvError},
    thread::{self},
    time::Duration,
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
