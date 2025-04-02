use std::{collections::HashMap, process, sync::Arc};
use terminal::TerminalScreenTrait;
use termion::event::Key;

mod maze;
mod moveset;
mod terminal;

fn main() {
    let mut screen = terminal::TerminalScreen::new(String::from("Press Esc to exit"), (0, 0));
    let mut key_actions: HashMap<Key, Arc<dyn Fn() + Send + Sync>> = HashMap::new();
    let mut run_actions: HashMap<(bool, u8), Arc<dyn Fn() + Send + Sync>> = HashMap::new();
    key_actions.insert(
        Key::Esc,
        Arc::new(|| {
            print!("\r\nExit");
            process::exit(0);
        }),
    );

    run_actions.insert(
        (true, 0),
        Arc::new(|| {
            print!("test");
        }),
    );

    screen.add_key_actions(key_actions);
    screen.add_run_actions(run_actions);

    unsafe { screen.run() };
}
