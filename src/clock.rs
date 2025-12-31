use std::{fmt::Display, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::{event::TIMER_TICK, tabs::TimeCtrl};

macro_rules! font {
    ($d:expr) => {
        match $d {
            '0' => include_str!("font/0.txt"),
            '1' => include_str!("font/1.txt"),
            '2' => include_str!("font/2.txt"),
            '3' => include_str!("font/3.txt"),
            '4' => include_str!("font/4.txt"),
            '5' => include_str!("font/5.txt"),
            '6' => include_str!("font/6.txt"),
            '7' => include_str!("font/7.txt"),
            '8' => include_str!("font/8.txt"),
            '9' => include_str!("font/9.txt"),
            '.' => include_str!("font/dot.txt"),
            ':' => include_str!("font/colon.txt"),
            _ => unreachable!(),
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Time(pub Duration);

impl Time {
    fn with_font(&self) -> String {
        let mut split_vec: Vec<Vec<&str>> = vec![];
        for d in self.to_string().chars() {
            split_vec.push(font!(d).split('\n').collect());
        }
        let mut line: Vec<&str> = Vec::new();
        let letter_height = split_vec[0].len();

        for i in 0..letter_height {
            for n in &split_vec {
                // skip empty str
                if n[i].len() > 0 {
                    line.push(n[i]);
                    line.push(" ");
                }
            }
            line.push("\n");
        }
        line.join("")
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_s = self.0.as_secs();
        let hh = total_s / 3_600;
        let mm = (total_s % 3_600) / 60;
        let ss = total_s % 60;
        let ms = (self.0.as_millis() % 1000) / 100;

        if hh > 0 {
            write!(f, "{:02}:{:02}:{:02}", hh, mm, ss)
        } else if mm == 0 && ss <= 20 {
            write!(f, "{:02}:{:02}.{}", mm, ss, ms)
        } else {
            write!(f, "{:02}:{:02}", mm, ss)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClockTurn {
    NotStarted,
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub player1: Time,
    pub player2: Time,
    pub turn: ClockTurn,
    pub increment: Duration,
    pub time_ctrl: TimeCtrl,
}

impl Clock {
    pub fn burning(time: Duration) -> bool {
        time < Duration::from_secs(21)
    }

    pub fn set(&mut self, ctrl: TimeCtrl) {
        self.time_ctrl = ctrl;
        self.player1.0 = ctrl.to_duration().0;
        self.player2.0 = ctrl.to_duration().0;
        self.increment = ctrl.to_duration().1;
        self.turn = ClockTurn::NotStarted;
    }

    pub fn hit(&mut self) {
        match self.turn {
            ClockTurn::NotStarted => self.turn = ClockTurn::Player1,
            ClockTurn::Player1 => {
                self.turn = ClockTurn::Player2;
                self.player1.0 += self.increment;
            }
            ClockTurn::Player2 => {
                self.turn = ClockTurn::Player1;
                self.player2.0 += self.increment;
            }
        }
    }

    pub fn tick_timer(&mut self) {
        let millisec = Duration::from_millis(TIMER_TICK);
        match self.turn {
            ClockTurn::NotStarted => (),
            ClockTurn::Player1 => {
                self.player1.0 = self.player1.0.saturating_sub(millisec);
            }
            ClockTurn::Player2 => {
                self.player2.0 = self.player2.0.saturating_sub(millisec);
            }
        }
    }

    pub fn is_time_out_player(&self) -> (bool, usize) {
        if self.player1.0 == Duration::ZERO {
            (true, 1)
        } else if self.player2.0 == Duration::ZERO {
            (true, 2)
        } else {
            (false, 0)
        }
    }

    pub fn is_time_out(&self) -> bool {
        if self.player1.0 == Duration::ZERO || self.player2.0 == Duration::ZERO {
            true
        } else {
            false
        }
    }

    pub fn render_time_out(self, area: Rect, buf: &mut Buffer) {
        let (is_time_out, player) = self.is_time_out_player();
        if !is_time_out {
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(33),
                Constraint::Fill(1),
            ])
            .split(area);

        let l = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(3),
                Constraint::Min(10),
                Constraint::Fill(1),
            ])
            .split(layout[1]);
        let p = Text::styled(
            format!("PLAYER {player} LOST ON TIME\nHit <space> to continue "),
            Style::default().bold().fg(Color::LightGreen),
        );
        Paragraph::new(p).render(l[1], buf);
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            increment: Duration::from_secs(1),
            player1: Time(Duration::from_secs(1)),
            player2: Time(Duration::from_secs(1)),
            turn: ClockTurn::NotStarted,
            time_ctrl: TimeCtrl::Tab1,
        }
    }
}

impl Widget for Clock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let l2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(3),
                Constraint::Min(10),
                Constraint::Fill(1),
            ])
            .split(layout[0]);
        let l3 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(3),
                Constraint::Min(10),
                Constraint::Fill(1),
            ])
            .split(layout[1]);
        let instructions = Line::from(vec![if matches!(self.turn, ClockTurn::NotStarted) {
            " Hit <space> to start ".fg(Color::LightGreen).bold().into()
        } else {
            // Show time control when clock has started
            self.time_ctrl
                .to_string()
                .fg(Color::LightGreen)
                .bold()
                .into()
        }]);
        let block = Block::default().title_bottom(instructions.centered());

        let active_style = Style::default().bold().fg(Color::LightGreen); // TODO: make it a separate Style along with LightGray Style  below
        let inactive_style = Style::default().bold().fg(Color::from_u32(0x007a7a7a));
        let burning_clock_style = Style::default().bold().fg(Color::LightRed);
        let styles = match self.turn {
            ClockTurn::NotStarted => vec![inactive_style, inactive_style],
            ClockTurn::Player1 => vec![
                if Clock::burning(self.player1.0) {
                    burning_clock_style
                } else {
                    active_style
                },
                inactive_style,
            ],
            ClockTurn::Player2 => vec![
                inactive_style,
                if Clock::burning(self.player2.0) {
                    burning_clock_style
                } else {
                    active_style
                },
            ],
        };

        let p1 = Text::styled(self.player1.with_font(), styles[0]);
        let p2 = Text::styled(self.player2.with_font(), styles[1]);
        Paragraph::new(p1).centered().render(l2[1], buf);
        Paragraph::new(p2).centered().render(l3[1], buf);
        block.render(area, buf);
    }
}
