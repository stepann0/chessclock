use crate::event::{AppEvent, Event, EventHandler, TIMER_TICK};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::{io, time::Duration};

#[derive(Debug)]
pub enum ClockTurn {
    NotStarted,
    Player1,
    Player2,
}

#[derive(Debug)]
pub struct Clock {
    player1: Duration,
    player2: Duration,
    turn: ClockTurn,
    increment: Duration,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            increment: Duration::from_secs(2),
            player1: Duration::from_secs(40),
            player2: Duration::from_secs(65),
            turn: ClockTurn::NotStarted,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub timeout: bool,
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    pub clock: Clock,
}

impl Default for App {
    fn default() -> Self {
        Self {
            timeout: false,
            clock: Clock::default(),
            running: true,
            events: EventHandler::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| self.ui(frame))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::TimerTick => self.tick_timer(),
                Event::Crossterm(event) => match event {
                    ratatui::crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Timeout => self.quit(),
                    AppEvent::HitClock => self.hit_clock(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char(' ') => self.hit_clock(),
            _ => {}
        }
        Ok(())
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn ui(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let title = Line::from(" Chess clock ".bold());
        let instructions = Line::from(vec![
            " Press <space> to start ".green().into(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::default()
            .title(title.centered())
            .title_bottom(instructions.centered());
        let styles = match self.clock.turn {
            ClockTurn::NotStarted => vec![Style::default(), Style::default()],
            ClockTurn::Player1 => vec![
                Style::default().fg(Color::Black).bg(Color::White),
                Style::default(),
            ],
            ClockTurn::Player2 => vec![
                Style::default(),
                Style::default().fg(Color::Black).bg(Color::White),
            ],
        };

        let p1 = Span::styled(format!("{}", show_time(self.clock.player1)), styles[0]);
        let p2 = Span::styled(format!("{}", show_time(self.clock.player2)), styles[1]);
        frame.render_widget(
            Paragraph::new(p1).centered().block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(Line::from(" PLAYER1 ").centered()),
            ),
            layout[0],
        );
        frame.render_widget(
            Paragraph::new(p2).centered().block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(Line::from(" PLAYER2 ").centered()),
            ),
            layout[1],
        );
        frame.render_widget(block, frame.area());
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn hit_clock(&mut self) {
        match self.clock.turn {
            ClockTurn::NotStarted => self.clock.turn = ClockTurn::Player1,
            ClockTurn::Player1 => {
                self.clock.turn = ClockTurn::Player2;
                self.clock.player1 += self.clock.increment;
            }
            ClockTurn::Player2 => {
                self.clock.turn = ClockTurn::Player1;
                self.clock.player2 += self.clock.increment;
            }
        }
    }

    pub fn tick_timer(&mut self) {
        let millisec = Duration::from_millis(TIMER_TICK);
        if self.clock.player1 == Duration::ZERO || self.clock.player2 == Duration::ZERO {
            self.events.send(AppEvent::Timeout);
            return;
        }
        match self.clock.turn {
            ClockTurn::NotStarted => return,
            ClockTurn::Player1 => {
                self.clock.player1 = self.clock.player1.saturating_sub(millisec);
                return;
            }
            ClockTurn::Player2 => {
                self.clock.player2 = self.clock.player2.saturating_sub(millisec);
                return;
            }
        }
    }
}

fn show_time(time: Duration) -> String {
    let total_s = time.as_secs();
    let hh = total_s / 3_600;
    let mm = (total_s % 3_600) / 60;
    let ss = total_s % 60;
    let ms = (time.as_millis() % 1000) / 100;

    if hh > 0 {
        format!("{:02}:{:02}:{:02}", hh, mm, ss)
    } else if mm == 0 && ss <= 20 {
        format!("{:02}:{:02}.{}", mm, ss, ms)
    } else {
        format!("{:02}:{:02}", mm, ss)
    }
}
