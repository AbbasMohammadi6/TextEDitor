use std::io::{stdout, Stdout};

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

struct Editor {
    cx: u16,
    cy: u16,
    mode: Mode,
    stdout: Stdout,
}

impl Editor {
    fn handle_normal_event(&self, code: KeyCode) -> anyhow::Result<Option<Action>> {
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

    fn handle_insert_event(&self, code: KeyCode) -> anyhow::Result<Option<Action>> {
        match code {
            KeyCode::Char(c) => Ok(Some(Action::Print(c))),
            KeyCode::Enter => Ok(Some(Action::NextLine)),
            KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Noraml))),
            _ => Ok(None),
        }
    }

    fn handle_event(&self, code: KeyCode) -> anyhow::Result<Option<Action>> {
        match self.mode {
            Mode::Noraml => self.handle_normal_event(code),
            Mode::Insert => self.handle_insert_event(code),
        }
    }

    fn new() -> Self {
        Self {
            cy: 0,
            cx: 0,
            mode: Mode::Noraml,
            stdout: stdout(),
        }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;

        self.stdout.execute(EnterAlternateScreen)?;

        loop {
            self.stdout.execute(MoveTo(self.cx, self.cy))?;
            match read()? {
                Event::Key(e) => match self.handle_event(e.code)? {
                    Some(action) => match action {
                        Action::MoveUp => {
                            self.cy -= 1;
                        }
                        Action::MoveDown => {
                            self.cy += 1;
                        }
                        Action::MoveRight => {
                            self.cx += 1;
                        }
                        Action::MoveLeft => {
                            self.cx -= 1;
                        }
                        Action::Quite => {
                            break;
                        }
                        Action::Print(c) => {
                            self.cx += 1;
                            self.stdout.execute(Print(c))?;
                        }
                        Action::EnterMode(m) => match m {
                            Mode::Insert => self.mode = Mode::Insert,
                            Mode::Noraml => self.mode = Mode::Noraml,
                        },
                        Action::NextLine => {
                            self.cx = 0;
                            self.cy += 1;
                        }
                    },
                    None => (),
                },
                _ => (),
            }
        }

        terminal::disable_raw_mode()?;
        self.stdout.execute(LeaveAlternateScreen)?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    editor.run()?;
    Ok(())
}
