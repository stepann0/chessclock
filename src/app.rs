use crate::event::{AppEvent, Event, EventHandler, TIMER_TICK};
use crate::tabs::TimeCtrl;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
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
    time_ctrl: (Duration, Duration),
}

impl Clock {
    pub fn burning(time: Duration) -> bool {
        time < Duration::from_secs(21)
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            increment: Duration::from_secs(2),
            player1: Duration::from_secs(40),
            player2: Duration::from_secs(65),
            turn: ClockTurn::NotStarted,
            time_ctrl: (Duration::from_secs(180), Duration::from_secs(2)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Screen {
    Main,
    PickTimeCtrl(TimeCtrl),
    TimeOut,
}

#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    pub timeout: bool,
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    pub clock: Clock,
    // pub time_ctrl_picker: SelectedTab,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::PickTimeCtrl(TimeCtrl::default()),
            timeout: false,
            clock: Clock::default(),
            running: true,
            events: EventHandler::new(),
            // time_ctrl_picker: SelectedTab::default(),
        }
    }
}

impl App {
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

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match &mut self.screen {
            Screen::Main => todo!(),
            Screen::PickTimeCtrl(s) => s.handle_key_events(key_event),
            Screen::TimeOut => todo!(),
        }
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char(' ') => {
                // panic!("from handle_key_events: {}", self.);
            }
            _ => {}
        }
        Ok(())
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn ui(&mut self, frame: &mut Frame) {
        self.pick_time_ctrl(frame);
        return;
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let l2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Max(3),
                Constraint::Fill(1),
            ])
            .split(layout[0]);
        let l3 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Max(3),
                Constraint::Fill(1),
            ])
            .split(layout[1]);
        let title = Line::from(" Chess clock ".bold());
        let instructions = Line::from(vec![
            " Press <space> to start "
                .fg(Color::LightGreen)
                .bold()
                .into(),
            " Quit ".bold().into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::default()
            .title(title.centered())
            .title_bottom(instructions.centered());
        let active_style = Style::default().bold().fg(Color::LightGreen); // TODO: make it a separate Style along with LightGray Style  below
        let inactive_style = Style::default().bold().fg(Color::from_u32(0x007a7a7a));
        let burning_clock_style = Style::default().bold().fg(Color::LightRed);
        let styles = match self.clock.turn {
            ClockTurn::NotStarted => vec![inactive_style, inactive_style],
            ClockTurn::Player1 => vec![
                if Clock::burning(self.clock.player1) {
                    burning_clock_style
                } else {
                    active_style
                },
                inactive_style,
            ],
            ClockTurn::Player2 => vec![
                inactive_style,
                if Clock::burning(self.clock.player2) {
                    burning_clock_style
                } else {
                    active_style
                },
            ],
        };

        let p1 = Span::styled(format!("{}", show_time(self.clock.player1)), styles[0]);
        let p2 = Span::styled(format!("{}", show_time(self.clock.player2)), styles[1]);
        frame.render_widget(Paragraph::new(p1).centered().block(Block::new()), l2[1]);
        frame.render_widget(Paragraph::new(p2).centered().block(Block::new()), l3[1]);
        frame.render_widget(block, frame.area());
    }

    pub fn pick_time_ctrl(&mut self, frame: &mut Frame) {
        let center = self.popup_area(frame.area(), 50, 3);
        // self.time_ctrl_picker.render(center, frame.buffer_mut());
        match &mut self.screen {
            Screen::PickTimeCtrl(s) => s.render(center, frame.buffer_mut()),
            Screen::Main => unreachable!(),
            Screen::TimeOut => unreachable!(),
        }
    }

    // helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(&self, area: Rect, percent_x: u16, height: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

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
