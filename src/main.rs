use terminal_screen::TerminalScreenTrait;

mod maze;
mod moveset;
mod terminal;
mod terminal_screen;

fn main() {
    let screen = terminal_screen::TerminalScreen::new(String::from("Press Esc to exit"), (0, 0));

    unsafe { screen.run() };
}
