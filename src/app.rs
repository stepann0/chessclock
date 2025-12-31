use crate::clock::Clock;
use crate::event::{AppEvent, Event, EventHandler};
use crate::tabs::TimeCtrl;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Widget,
};
use std::io;

#[derive(Debug, PartialEq)]
pub enum Screen {
    Clocks,
    PickTimeCtrl,
    TimeOut,
}

#[derive(Debug)]
pub struct App {
    // Event handler.
    pub events: EventHandler,
    pub running: bool,

    // Multi-screen logic goes here
    pub screen: Screen,
    pub clock: Clock,
    pub time_ctrl_picker: TimeCtrl,
}

impl Default for App {
    fn default() -> Self {
        Self {
            clock: Clock::default(),
            running: true,
            events: EventHandler::new(),
            screen: Screen::PickTimeCtrl,
            time_ctrl_picker: TimeCtrl::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|frame| self.ui(frame))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::TimerTick => {
                    if self.clock.is_time_out() && self.screen == Screen::Clocks {
                        self.events.send(AppEvent::Timeout);
                    }
                    self.clock.tick_timer();
                }
                Event::Crossterm(event) => match event {
                    ratatui::crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Timeout => self.screen = Screen::TimeOut,
                    AppEvent::HitClock => self.hit_clock(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            _ => {}
        }

        match self.screen {
            Screen::Clocks => match key_event.code {
                KeyCode::Char(' ') => {
                    self.events.send(AppEvent::HitClock);
                }
                _ => {}
            },
            Screen::PickTimeCtrl => match key_event.code {
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char(' ') | KeyCode::Enter => {
                    self.clock.set(self.time_ctrl_picker);
                    self.screen = Screen::Clocks;
                }
                _ => self.time_ctrl_picker.handle_key_events(key_event),
            },
            Screen::TimeOut => match key_event.code {
                KeyCode::Char('R') | KeyCode::Char('r') | KeyCode::Enter => {
                    self.screen = Screen::PickTimeCtrl;
                }
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                _ => {}
            },
        }
        Ok(())
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn ui(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Clocks => self.render_clocks(frame),
            Screen::PickTimeCtrl => self.render_pick_time_ctrl(frame),
            Screen::TimeOut => self.render_time_out(frame),
        }
    }

    pub fn render_clocks(&mut self, frame: &mut Frame) {
        self.clock.render(frame.area(), frame.buffer_mut());
    }

    pub fn render_pick_time_ctrl(&mut self, frame: &mut Frame) {
        let center = self.popup_area(frame.area(), 40, 3);
        self.time_ctrl_picker.render(center, frame.buffer_mut());
    }

    pub fn render_time_out(&mut self, frame: &mut Frame) {
        self.render_clocks(frame);
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
        self.clock.hit();
    }
}
