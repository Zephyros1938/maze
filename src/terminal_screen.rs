use std::{
    collections::HashMap,
    io::{Stdin, Stdout, Write, stdin, stdout},
    ops::DerefMut,
    process::{self, Output},
    rc::Rc,
    sync::{Arc, Mutex, mpsc::TryRecvError},
    thread,
    time::Duration,
};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

// MARK: constants

const SCREEN_BLANK_CHAR: char = ' ';
const SCREEN_PIXEL_WIDTH: usize = 2;
const SCREEN_REFRESH_RATE: f32 = 15.0;

pub struct MClosure<T> {
    data: Arc<Mutex<Box<T>>>,
}
impl<T> MClosure<T> {
    pub fn new(c: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(Box::new(c))),
        }
    }
}

pub enum ActionType {
    KEY(Key),
    RUN,
}

pub enum ScreenAction {
    EXIT(i32),
    SETPIXELCHAR((usize, usize, char)),
    PRINTC((u16, u16, Vec<char>)),
    PRINT((u16, u16, String)),
    CUSTOMFN_ARC(Arc<dyn Fn() + Send + Sync>),

    FN_SETPIXELCHAR(
        (
            Arc<dyn Fn() -> usize + Send + Sync>,
            Arc<dyn Fn() -> usize + Send + Sync>,
            Arc<dyn Fn() -> char + Send + Sync>,
        ),
    ),

    FN_SA(
        MClosure<ScreenAction>, /*<dyn FnMut() -> ScreenAction + Send + Sync>*/
    ),

    FN(Arc<dyn Fn() -> ScreenAction + Send + Sync>),
}

pub struct TerminalScreen {
    pixel_front_buffer: Vec<Vec<char>>,
    pixel_back_buffer: Vec<Vec<char>>,
    dimensions: (usize, usize),
    pub stdin: Stdin,
    pub stdout: RawTerminal<Stdout>,
    pub actions: ActionManager,
    master_channel: (
        std::sync::mpsc::Sender<ScreenAction>,
        std::sync::mpsc::Receiver<ScreenAction>,
    ),
}

impl TerminalScreen {
    pub fn new(dimensions: (usize, usize)) -> Self {
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
        let actions = ActionManager::from(vec![
            Action::new(ActionType::KEY(Key::Ctrl('c')), ScreenAction::EXIT(130)),
            Action::new(ActionType::KEY(Key::Esc), ScreenAction::EXIT(0)),
        ]);

        Self {
            pixel_front_buffer: pixel_buffer.clone(),
            pixel_back_buffer: pixel_buffer,
            dimensions,
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            actions,
            master_channel: std::sync::mpsc::channel(),
        }
    }
}

pub unsafe trait TerminalScreenTrait {
    unsafe fn run(self);
    unsafe fn add_action(&mut self, action: Action);
    unsafe fn do_action(&mut self, action: ScreenAction);
}

unsafe impl TerminalScreenTrait for TerminalScreen {
    unsafe fn run(mut self) {
        let (event_tx, event_rx) = std::sync::mpsc::channel::<Arc<dyn Fn() + Send + Sync>>();
        let (pixel_tx, pixel_rx) = std::sync::mpsc::channel::<(usize, usize, char)>();
        let (write_tx, write_rx) = std::sync::mpsc::channel::<(u16, u16, String)>();
        self.stdout.flush().unwrap();
        let event_thread = thread::Builder::new()
            .name(String::from("Event Thread"))
            .spawn(move || {
                for _c in self.stdin.keys() {
                    // get input keys
                    for _action in self.actions.iter() {
                        match _action.t {
                            // depends on the action keytypes
                            ActionType::KEY(_k) => {
                                // k is the key
                                if let Ok(_key) = _c {
                                    // if the current inputted key (c) isnt an error, assign it to `key`
                                    if _key == _k {
                                        // if `key` is the actions key (`k`)
                                        match _action.f {
                                            // all the possible actions
                                            ScreenAction::EXIT(code) => process::exit(code),
                                            ScreenAction::SETPIXELCHAR((x, y, chr)) => {
                                                pixel_tx.send((x, y, chr)).unwrap()
                                            }
                                            ScreenAction::FN_SETPIXELCHAR((
                                                ref x,
                                                ref y,
                                                ref chr,
                                            )) => pixel_tx.send((x(), y(), chr())).unwrap(),
                                            ScreenAction::PRINTC((x, y, ref chr)) => write_tx
                                                .send((
                                                    x,
                                                    y,
                                                    String::from(
                                                        chr.into_iter().collect::<String>(),
                                                    ),
                                                ))
                                                .unwrap(),
                                            ScreenAction::PRINT((x, y, ref st)) => {
                                                write_tx.send((x, y, st.to_owned())).unwrap()
                                            }
                                            ScreenAction::CUSTOMFN_ARC(ref f) => {
                                                event_tx.send(f.to_owned()).unwrap()
                                            }
                                            ScreenAction::FN_SA(ref f) => match **f.data.lock().unwrap() {
                                                ScreenAction::FN_SA(ref _f) => panic!("ScreenAction::FN_SA cannot lead to another ScreenAction::FN_SA!"),
                                                ScreenAction::FN(ref _f) => panic!("ScreenAction::FN_SA cannot lead to a ScreenAction::FN!"),
                                                ScreenAction::SETPIXELCHAR((x, y, chr)) => {
                                                    pixel_tx.send((x, y, chr)).unwrap()
                                                }
                                                _ => unimplemented!(),
                                            },
                                            ScreenAction::FN(ref f) => match f()
                                            {
                                                ScreenAction::FN_SA( _f) => panic!("ScreenAction::FN cannot lead to another ScreenAction::FN!"),
                                                    ScreenAction::FN( _f) => panic!("ScreenAction::FN cannot lead to a ScreenAction::FN_SA!"),
                                                    ScreenAction::SETPIXELCHAR(( x, y, chr)) => {
                                                        pixel_tx.send((x, y, chr)).unwrap()
                                                    }
                                                    _ => unimplemented!(),
                                            }
                                            _ => unimplemented!(),
                                        }
                                    }
                                }
                            }
                            ActionType::RUN => match _action.f {
                                // all the possible actions
                                ScreenAction::EXIT(code) => process::exit(code),
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
                r#"{}{}"#,
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
                        termion::cursor::Goto(xu16, yu16),
                        String::from(self.pixel_front_buffer[y][x]).repeat(SCREEN_PIXEL_WIDTH),
                    )
                    .unwrap();
                }
            }

            match event_rx.try_recv() {
                Ok(resp) => resp(),
                Err(TryRecvError::Disconnected) => panic!("event_rx disconnected!"),
                Err(TryRecvError::Empty) => (),
            }

            match pixel_rx.try_recv() {
                Ok(data) => self.pixel_back_buffer[data.1][data.0] = data.2,
                Err(TryRecvError::Disconnected) => panic!("pixel_rx disconnected!"),
                Err(TryRecvError::Empty) => (),
            }

            match self.master_channel.1.try_recv() {
                Ok(data) => match data {
                    ScreenAction::SETPIXELCHAR((x, y, c)) => self.pixel_back_buffer[y][x] = c,
                    ScreenAction::CUSTOMFN_ARC(f) => f(),
                    ScreenAction::EXIT(code) => process::exit(code),
                    _ => unimplemented!(),
                },
                Err(TryRecvError::Disconnected) => panic!("pixel_rx disconnected!"),
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
                Err(TryRecvError::Disconnected) => panic!("write_rx disconnected!"),
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

    unsafe fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    unsafe fn do_action(&mut self, action: ScreenAction) {
        (*self).master_channel.0.send(action).unwrap();
    }
}

pub struct Action {
    pub t: ActionType,
    pub f: ScreenAction,
}

impl Action {
    pub fn new(t: ActionType, f: ScreenAction) -> Self {
        Self { t, f }
    }
}

pub struct ActionManager {
    actions: HashMap<u16, Action>,
}

impl std::ops::Index<&'_ u16> for ActionManager {
    type Output = Action;

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
    fn from(value: Vec<Action>) -> Self {
        let mut action_manager = ActionManager::new();
        for item in value {
            action_manager.push(item);
        }
        action_manager
    }
    pub fn insert(&mut self, data: (u16, Action)) {
        self.actions.insert(data.0, data.1);
    }
    pub fn push(&mut self, data: Action) {
        let key = self.actions.len() as u16;
        self.actions.insert(key, data);
    }
    pub fn rem(&mut self, key: u16) {
        self.actions.remove(&key);
    }
    pub fn iter(&mut self) -> std::collections::hash_map::Values<'_, u16, Action> {
        self.actions.values()
    }
}
