use std::io::stdout;

use anyhow::Ok;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode},
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

enum Mode {
    Insert,
    Noraml,
}

enum Action {
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    Print(char),
    Quite,
    EnterMode(Mode),
    NextLine,
}

fn handle_event(code: KeyCode, mode: &Mode) -> anyhow::Result<Option<Action>> {
    match mode {
        Mode::Noraml => handle_normal_event(code),
        Mode::Insert => handle_insert_event(code),
    }
}

fn handle_normal_event(code: KeyCode) -> anyhow::Result<Option<Action>> {
    match code {
        KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
        KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
        KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
        KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
        KeyCode::Char('q') => Ok(Some(Action::Quite)),
        KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
        _ => Ok(None),
    }
}

fn handle_insert_event(code: KeyCode) -> anyhow::Result<Option<Action>> {
    match code {
        KeyCode::Char(c) => Ok(Some(Action::Print(c))),
        KeyCode::Enter => Ok(Some(Action::NextLine)),
        KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Noraml))),
        _ => Ok(None),
    }
}

fn main() -> anyhow::Result<()> {
    let mut mode = Mode::Noraml;
    let mut cx = 0;
    let mut cy = 0;
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    stdout.execute(EnterAlternateScreen)?;

    loop {
        stdout.execute(MoveTo(cx, cy))?;
        match read()? {
            Event::Key(e) => match handle_event(e.code, &mode)? {
                Some(action) => match action {
                    Action::MoveUp => {
                        cy -= 1;
                    }
                    Action::MoveDown => {
                        cy += 1;
                    }
                    Action::MoveRight => {
                        cx += 1;
                    }
                    Action::MoveLeft => {
                        cx -= 1;
                    }
                    Action::Quite => {
                        break;
                    }
                    Action::Print(c) => {
                        cx += 1;
                        stdout.execute(Print(c))?;
                    }
                    Action::EnterMode(m) => match m {
                        Mode::Insert => mode = Mode::Insert,
                        Mode::Noraml => mode = Mode::Noraml,
                    },
                    Action::NextLine => {
                        cx = 0;
                        cy += 1;
                    }
                },
                None => (),
            },
            _ => (),
        }
    }

    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}
