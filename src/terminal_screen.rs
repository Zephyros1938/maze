use std::{
    collections::HashMap,
    io::{Stdin, Stdout, Write, stdin, stdout},
    process,
    sync::{Arc, mpsc::TryRecvError},
    thread,
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

pub enum ActionType {
    KEY(Key),
    RUN,
}

pub enum TerminalScreenAction {
    EXIT(i32),
    SETPIXELCHAR((usize, usize, char)),
    PRINT((u16, u16, Vec<char>)),
}

pub struct TerminalScreen {
    pixel_front_buffer: Vec<Vec<char>>,
    pixel_back_buffer: Vec<Vec<char>>,
    dimensions: (usize, usize),
    pub startmessage: String,
    pub stdin: Stdin,
    pub stdout: RawTerminal<Stdout>,
    pub actions: ActionManager,
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

        let pixel_buffer = vec![vec![SCREEN_BLANK_CHAR; dimensions.0]; dimensions.1];
        let mut actions = ActionManager::new();
        actions.push((ActionType::KEY(Key::Esc), TerminalScreenAction::EXIT(0)));
        actions.push((
            ActionType::KEY(Key::Char('w')),
            TerminalScreenAction::SETPIXELCHAR((2, 2, 'D')),
        ));
        actions.push((
            ActionType::KEY(Key::Char('e')),
            TerminalScreenAction::PRINT((20, 20, vec!['F'])),
        ));

        Self {
            pixel_front_buffer: pixel_buffer.clone(),
            pixel_back_buffer: pixel_buffer,
            dimensions,
            startmessage,
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            actions,
        }
    }
}

pub trait TerminalScreenTrait {
    unsafe fn run(self);
}

impl TerminalScreenTrait for TerminalScreen {
    unsafe fn run(mut self) {
        let (tx, rx) = std::sync::mpsc::channel::<Arc<dyn Fn() + Send + Sync>>();
        let (pixel_tx, pixel_rx) = std::sync::mpsc::channel::<(usize, usize, char)>();
        let (write_tx, write_rx) = std::sync::mpsc::channel::<(u16, u16, String)>();
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
                // get input keys
                for action in self.actions.iter() {
                    match action.0 {
                        // depends on the action keytypes
                        ActionType::KEY(k) => {
                            // k is the key
                            if let Ok(key) = c {
                                // if the current inputted key (c) isnt an error, assign it to `key`
                                if key == k {
                                    // if `key` is the actions key (`k`)
                                    match action.1 {
                                        // all the possible actions
                                        TerminalScreenAction::EXIT(code) => process::exit(code),
                                        TerminalScreenAction::SETPIXELCHAR((x, y, chr)) => {
                                            pixel_tx.send((x, y, chr)).unwrap()
                                        }
                                        TerminalScreenAction::PRINT((x, y, ref chr)) => write_tx
                                            .send((
                                                x,
                                                y,
                                                String::from(chr.into_iter().collect::<String>()),
                                            ))
                                            .unwrap(),
                                        _ => unimplemented!(),
                                    }
                                }
                            }
                        }
                        ActionType::RUN => match action.1 {
                            // all the possible actions
                            TerminalScreenAction::EXIT(code) => process::exit(code),
                            _ => unimplemented!(),
                        },
                    }
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

            for y in 0..self.dimensions.1 {
                let yu16: u16 = (y + 1).try_into().unwrap();
                for x in 0..self.dimensions.0 {
                    let xu16: u16 = ((x) * SCREEN_PIXEL_WIDTH).try_into().unwrap();
                    write!(
                        self.stdout,
                        r#"{}{}"#,
                        String::from(self.pixel_front_buffer[y][x]).repeat(SCREEN_PIXEL_WIDTH),
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

            match pixel_rx.try_recv() {
                Ok(data) => self.pixel_back_buffer[data.1][data.0] = data.2,
                Err(TryRecvError::Disconnected) => panic!("Disconnected!"),
                Err(TryRecvError::Empty) => (),
            }

            match write_rx.try_recv() {
                Ok(data) => write!(
                    self.stdout,
                    r#"{}{}"#,
                    termion::cursor::Goto(data.0, data.1),
                    data.2,
                )
                .unwrap(),
                Err(TryRecvError::Disconnected) => panic!("Disconnected!"),
                Err(TryRecvError::Empty) => (),
            }

            for y in 0..self.dimensions.1 {
                for x in 0..self.dimensions.0 {
                    if self.pixel_front_buffer[y][x] != self.pixel_back_buffer[y][x] {
                        self.pixel_front_buffer[y][x] = self.pixel_back_buffer[y][x];
                    }
                }
            }

            self.pixel_back_buffer = self.pixel_front_buffer.clone();

            self.stdout.flush().unwrap();
            thread::sleep(Duration::from_secs_f32(1.0 / SCREEN_REFRESH_RATE));
        }
    }
}

pub struct ActionManager {
    actions: HashMap<u16, (ActionType, TerminalScreenAction)>,
}

impl std::ops::Index<&'_ u16> for ActionManager {
    type Output = (ActionType, TerminalScreenAction);

    fn index(&self, index: &'_ u16) -> &Self::Output {
        match self.actions.get(index) {
            Some(action) => action,
            None => panic!("Action not found at index {}", index),
        }
    }
}

impl ActionManager {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }
    pub fn insert(&mut self, data: (u16, (ActionType, TerminalScreenAction))) {
        self.actions.insert(data.0, data.1);
    }
    pub fn push(&mut self, data: (ActionType, TerminalScreenAction)) {
        let key = self.actions.len() as u16;
        self.actions.insert(key, data);
    }
    pub fn iter(
        &mut self,
    ) -> std::collections::hash_map::Values<'_, u16, (ActionType, TerminalScreenAction)> {
        self.actions.values()
    }
}
