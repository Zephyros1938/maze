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

const SCREEN_BLANK_CHAR: char = ' ';
const SCREEN_PIXEL_WIDTH: usize = 2;
const SCREEN_REFRESH_RATE: f32 = 15.0;

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

enum TerminalScreenAction {
    KILL(u8),
    PRINT(u8),
    SETPIXEL(u8),
}

pub struct TerminalScreen {
    pixel_front_buffer: Vec<Vec<String>>,
    pixel_back_buffer: Vec<Vec<String>>,
    dimensions: (usize, usize),
    pub startmessage: String,
    pub stdin: Stdin,
    pub stdout: RawTerminal<Stdout>,
    key_actions: HashMap<Key, Arc<dyn Fn() + Send + Sync>>,
    run_actions: HashMap<(bool, u8), Arc<dyn Fn() + Send + Sync>>,
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

        let dimensions = if dimensions == (0, 0) {
            (cols, rows)
        } else {
            dimensions
        };

        let pixel_buffer =
            vec![
                vec![String::from(SCREEN_BLANK_CHAR).repeat(SCREEN_PIXEL_WIDTH); dimensions.0];
                dimensions.1
            ];

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

    pub fn draw(&self) -> RawTerminal<Stdout> {
        let mut stdout = stdout();
        for y in 0..self.dimensions.1 {
            let yu16: u16 = y.try_into().unwrap();
            for x in 0..self.dimensions.0 {
                let xu16: u16 = x.try_into().unwrap();
                write!(stdout, r#"{}{}"#, termion::cursor::Goto(xu16, yu16), x).unwrap();
            }
        }
        stdout.into_raw_mode().unwrap()
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
    unsafe fn run(self);

    fn add_key_action(&mut self, k: char, func: Arc<dyn Fn() + Send + Sync>);
    fn add_key_actions(&mut self, h: HashMap<Key, Arc<dyn Fn() + Send + Sync>>);
    fn rem_key_action(&mut self, k: char);
    fn rem_key_actions(&mut self, h: Vec<char>);

    // Run Actions

    fn add_run_action(&mut self, enabled: bool, id: u8, func: Arc<dyn Fn() + Send + Sync>);
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), Arc<dyn Fn() + Send + Sync>>);
    fn rem_run_action(&mut self, id: u8);
    fn rem_run_actions(&mut self, h: Vec<char>);
}

impl TerminalScreenTrait for TerminalScreen {
    unsafe fn run(mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        write!(
            self.stdout,
            r#"{}{}{}"#,
            self.startmessage,
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
        self.stdout.flush().unwrap();
        thread::spawn(move || {
            for c in self.stdin.keys() {
                if let Ok(key) = c {
                    if let Some(action) = self.key_actions.get(&key) {
                        // self.queued_actions.lock().unwrap().push(action.clone());
                        tx.send(action.clone()).unwrap();
                    }
                }

                for (enabled, action) in self.run_actions.iter() {
                    if enabled.0 == true {
                        // self.queued_actions.lock().unwrap().push(action.clone());
                        tx.send(action.clone()).unwrap();
                    };
                }
            }
        });

        loop {
            //clearing the screen and going to top left corner
            write!(
                self.stdout,
                "{:}{:}",
                termion::cursor::Goto(1, 1),
                termion::clear::All
            )
            .unwrap();

            self.pixel_front_buffer = self.pixel_back_buffer.clone();
            write!(self.stdout, "{}x{}", self.dimensions.0, self.dimensions.1).unwrap();
            for y in 0..self.dimensions.1 {
                let yu16: u16 = (y + 1).try_into().unwrap();
                for x in 0..self.dimensions.0 {
                    let xu16: u16 = ((x + 1) * SCREEN_PIXEL_WIDTH).try_into().unwrap();
                    write!(
                        self.stdout,
                        "{}{}",
                        self.pixel_front_buffer[y][x],
                        termion::cursor::Goto(xu16, yu16)
                    )
                    .unwrap();
                }
            }

            match rx.try_recv() {
                Ok(resp) => resp(),
                Err(TryRecvError::Disconnected) => panic!("Disconnected!"),
                Err(TryRecvError::Empty) => (),
            }

            for y in 0..self.dimensions.1 {
                for x in 0..self.dimensions.0 {
                    self.pixel_back_buffer[y][x] = String::from(
                        if self.pixel_back_buffer[y][x]
                            == String::from(SCREEN_BLANK_CHAR).repeat(SCREEN_PIXEL_WIDTH)
                        {
                            String::from('#')
                        } else {
                            String::from(SCREEN_BLANK_CHAR)
                        },
                    )
                    .repeat(SCREEN_PIXEL_WIDTH);
                }
            }

            self.stdout.flush().unwrap();
            thread::sleep(Duration::from_secs_f32(1.0 / SCREEN_REFRESH_RATE));
        }
    }

    fn add_key_action(&mut self, k: char, func: Arc<dyn Fn() + Send + Sync>) {
        self.key_actions.insert(Key::Char(k), func);
    }
    fn add_key_actions(&mut self, h: HashMap<Key, Arc<dyn Fn() + Send + Sync>>) {
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

    fn add_run_action(&mut self, enabled: bool, id: u8, func: Arc<dyn Fn() + Send + Sync>) {
        self.run_actions.insert((enabled, id), func);
    }
    fn add_run_actions(&mut self, h: HashMap<(bool, u8), Arc<dyn Fn() + Send + Sync>>) {
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
