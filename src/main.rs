use std::{collections::HashMap, process, sync::Arc};
use terminal::{TerminalScreen, TerminalScreenTrait, TerminalTrait};
use termion::event::Key;

mod maze;
mod moveset;
mod terminal;

fn main() {
    let mut screen = terminal::TerminalScreen::new(String::from("Press Esc to exit"), (0, 0));
    let mut key_actions: HashMap<Key, Arc<dyn Fn()>> = HashMap::new();
    let mut run_actions: HashMap<(bool, u8), Arc<dyn Fn()>> = HashMap::new();
    key_actions.insert(
        Key::Esc,
        Arc::new(|| {
            println!("Exit");
            process::exit(0);
        }),
    );

    screen.add_key_actions(key_actions);
    screen.add_run_actions(run_actions);

    screen.run();
}
