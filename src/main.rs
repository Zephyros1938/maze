use std::{any::Any, rc::Rc, sync::Arc};

use rand::random_range;
use terminal_screen::{Action, ActionType, MClosure, ScreenAction, TerminalScreenTrait};
use utility::Number;

mod maze;
mod moveset;
mod terminal;
mod terminal_screen;
mod utility;

fn main() {
    unsafe {
        let mut screen = terminal_screen::TerminalScreen::new((0, 0));

        let mut p = Point::new(10, 10, 'c');

        let p_clone = p.clone(); // Create a clone of p to avoid borrow checker issues

        screen.add_action(terminal_screen::Action {
            t: ActionType::KEY(termion::event::Key::Char('w')),
            f: ScreenAction::FN(Arc::new(move || {
                // Use move closure to capture p_clone
                p_clone.y -= 1;
                p_clone.c = 'e';
                ScreenAction::SETPIXELCHAR((p_clone.x, p_clone.y, p_clone.c))
            })),
        });

        screen.run()
    }
}
#[derive(Clone, Copy)]
struct Point {
    pub x: usize,
    pub y: usize,
    pub c: char,
}

impl Point {
    pub fn new(x: usize, y: usize, c: char) -> Self {
        Self { x, y, c }
    }
}
