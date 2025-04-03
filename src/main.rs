use std::{any::Any, rc::Rc, sync::Arc};

use rand::random_range;
use terminal_screen::{Action, ActionType, MClosure, ScreenAction, TerminalScreenTrait};

mod maze;
mod moveset;
mod terminal;
mod terminal_screen;

fn main() {
    let mut screen = terminal_screen::TerminalScreen::new((0, 0));

    screen.add_action(terminal_screen::Action {
        t: ActionType::KEY(termion::event::Key::Char('f')),
        f: ScreenAction::CUSTOMFN(MClosure::new(|| {
            1;
        })),
    });

    unsafe { screen.run() };
}
