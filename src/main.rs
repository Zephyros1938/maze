use std::{collections::HashMap, process};
use terminal::Screen;
use termion::event::Key;

mod maze;
mod moveset;
mod terminal;

fn main() {
    let mut term = terminal::Terminal::new(String::from("Press Esc to exit"));
    let mut screen = terminal::Screen::new((20, 20));
    let mut key_actions: HashMap<Key, Box<dyn FnMut() + Send + 'static>> = HashMap::new();
    key_actions.insert(
        Key::Esc,
        Box::new(|| {
            println!("Exit");
            process::exit(0);
        }),
    );
    key_actions.insert(
        Key::Char('w'),
        Box::new(|| {
            screen.print();
        }),
    );
    term.add_key_actions(key_actions);

    term.run();
}
