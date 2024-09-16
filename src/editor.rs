use std::{io::{stdout, Stdout}, u16};

use anyhow::Ok;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode},
    style::{Color, Print, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

const STATUS_LINE_HEIGHT: u16 = 2;

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

pub struct Editor {
    cx: u16,
    cy: u16,
    mode: Mode,
    stdout: Stdout,
    vwidth: u16,
    vheight: u16,
}

impl Drop for Editor {
    fn drop(&mut self) {
        _ = terminal::disable_raw_mode();
        _ = self.stdout.execute(LeaveAlternateScreen);
    }
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

    pub fn new() -> anyhow::Result<Self> {
        let size = terminal::size()?;
        Ok(Self {
            cy: 0,
            cx: 0,
            mode: Mode::Noraml,
            stdout: stdout(),
            vwidth: size.0,
            vheight: size.1,
        })
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_status_line()?;
        self.stdout.execute(MoveTo(self.cx, self.cy))?;
        Ok(())
    }

    fn draw_status_line(&mut self) -> anyhow::Result<()> {
        self.stdout.execute(MoveTo(1, self.vheight - 2))?;
        let mode = match self.mode {
            Mode::Insert => "NORMAL",
            Mode::Noraml => "INSERT",
        };

        let primary_color = Color::Rgb {
            r: 124,
            g: 245,
            b: 255,
        };
        let light_gray_color = Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        };
        let dark_gray_color = Color::Rgb {
            r: 30,
            g: 30,
            b: 30,
        };
        let black_color = Color::Black;
        let file_name = format!(" main.rs ");
        let mode = format!(" {mode} ");
        let chev_right = "";
        let chev_left = "";
        let cursor_position = format!(" {}:{} ", self.cx, self.cy);
        let span_len = self.vwidth
            - (file_name.len() as u16 + mode.len() as u16 + chev_right.len() as u16 * 2 + cursor_position.len() as u16);
        let empty_space = format!("{:width$}", "", width = span_len as usize);

        self.stdout
            .execute(Print(mode.with(black_color).on(primary_color)))?;
        self.stdout
            .execute(Print(chev_right.with(primary_color).on(light_gray_color)))?;
        self.stdout
            .execute(Print(file_name.with(primary_color).on(light_gray_color)))?;
        self.stdout
            .execute(Print(chev_right.with(light_gray_color).on(dark_gray_color)))?;
        self.stdout
            .execute(Print(empty_space.on(dark_gray_color)))?;
        self.stdout
            .execute(Print(chev_left.with(primary_color).on(dark_gray_color)))?;
        self.stdout
            .execute(Print(cursor_position.with(black_color).on(primary_color)))?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.execute(EnterAlternateScreen)?;

        loop {
            self.draw()?;
            match read()? {
                Event::Key(e) => match self.handle_event(e.code)? {
                    Some(action) => match action {
                        Action::MoveUp => {
                            self.cy = self.cy.saturating_sub(1);
                        }
                        Action::MoveDown => {
                            if self.cy < self.vheight - (1 + STATUS_LINE_HEIGHT) {
                                self.cy += 1;
                            }
                        }
                        Action::MoveRight => {
                            if self.cx < self.vwidth - 1 {
                                self.cx += 1;
                            }
                        }
                        Action::MoveLeft => {
                            self.cx = self.cx.saturating_sub(1);
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

        Ok(())
    }
}
