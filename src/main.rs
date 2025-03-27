use std::{collections::HashMap, process};
use termion::event::Key;

mod maze;
mod moveset;
mod terminal;

fn main() {
    let mut key_actions: HashMap<Key, Box<dyn FnMut() + Send>> = HashMap::new();
    key_actions.insert(Key::Char('w'), Box::new(|| println!("2")));
    key_actions.insert(
        Key::Esc,
        Box::new(|| {
            println!("Exit");
            process::exit(0);
        }),
    );
    let mut term = terminal::Terminal::new(String::from("Press Esc to exit"));
    term.add_key_actions(key_actions);

    term.run();
}
