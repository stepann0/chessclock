use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::{event::TIMER_TICK, tabs::CtrlOption};

#[derive(Debug, Clone, Copy)]
pub enum ClockTurn {
    NotStarted,
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    player1: Duration,
    player2: Duration,
    turn: ClockTurn,
    increment: Duration,
    time_ctrl: CtrlOption,
}

impl Clock {
    pub fn burning(time: Duration) -> bool {
        time < Duration::from_secs(21)
    }

    pub fn set(&mut self, ctrl: CtrlOption) {
        self.time_ctrl = ctrl;
        self.player1 = ctrl.0;
        self.player2 = ctrl.0;
        self.increment = ctrl.1;
    }

    pub fn hit(&mut self) {
        match self.turn {
            ClockTurn::NotStarted => self.turn = ClockTurn::Player1,
            ClockTurn::Player1 => {
                self.turn = ClockTurn::Player2;
                self.player1 += self.increment;
            }
            ClockTurn::Player2 => {
                self.turn = ClockTurn::Player1;
                self.player2 += self.increment;
            }
        }
    }

    pub fn tick_timer(&mut self) {
        let millisec = Duration::from_millis(TIMER_TICK);
        match self.turn {
            ClockTurn::NotStarted => return,
            ClockTurn::Player1 => {
                self.player1 = self.player1.saturating_sub(millisec);
                return;
            }
            ClockTurn::Player2 => {
                self.player2 = self.player2.saturating_sub(millisec);
                return;
            }
        }
    }

    pub fn is_time_out(&self) -> bool {
        if self.player1 == Duration::ZERO || self.player2 == Duration::ZERO {
            return true;
        }
        false
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            increment: Duration::from_secs(1),
            player1: Duration::from_secs(1),
            player2: Duration::from_secs(1),
            turn: ClockTurn::NotStarted,
            time_ctrl: (Duration::from_secs(180), Duration::from_secs(2)),
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
        let instructions = Line::from(vec![if matches!(self.turn, ClockTurn::NotStarted) {
            " Hit <space> to start ".fg(Color::LightGreen).bold().into()
        } else {
            // Do not show anything when clock has started
            "".into()
        }]);
        let block = Block::default().title_bottom(instructions.centered());

        let active_style = Style::default().bold().fg(Color::LightGreen); // TODO: make it a separate Style along with LightGray Style  below
        let inactive_style = Style::default().bold().fg(Color::from_u32(0x007a7a7a));
        let burning_clock_style = Style::default().bold().fg(Color::LightRed);
        let styles = match self.turn {
            ClockTurn::NotStarted => vec![inactive_style, inactive_style],
            ClockTurn::Player1 => vec![
                if Clock::burning(self.player1) {
                    burning_clock_style
                } else {
                    active_style
                },
                inactive_style,
            ],
            ClockTurn::Player2 => vec![
                inactive_style,
                if Clock::burning(self.player2) {
                    burning_clock_style
                } else {
                    active_style
                },
            ],
        };

        let p1 = Span::styled(format!("{}", show_time(self.player1)), styles[0]);
        let p2 = Span::styled(format!("{}", show_time(self.player2)), styles[1]);
        Paragraph::new(p1)
            .centered()
            .block(Block::new())
            .render(l2[1], buf);
        Paragraph::new(p2)
            .centered()
            .block(Block::new())
            .render(l3[1], buf);
        block.render(area, buf);
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
