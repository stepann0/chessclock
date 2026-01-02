use std::{fmt::Display, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{Fill, Length, Min, Percentage},
        Direction, Layout, Rect,
    },
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

        let letter_height = split_vec[0].len();
        let lines: Vec<_> = (0..letter_height)
            .map(|i| {
                split_vec
                    .iter()
                    .map(move |s| s[i])
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        lines.join("\n")
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
pub enum Player {
    Player1,
    Player2,
}

impl Default for Player {
    fn default() -> Self {
        Player::Player1
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClockState {
    NotStarted,
    Pause,
    Player(Player),
}

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub player1: Time,
    pub player2: Time,
    pub state: ClockState,
    resume_player: Player, // player turn before pause
    first_to_move: Player,
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
        self.state = ClockState::NotStarted;
    }

    pub fn curr_player(&self) -> Option<Player> {
        match self.state {
            ClockState::Player(p) => Some(p),
            _ => None,
        }
    }

    pub fn hit(&mut self) {
        match self.state {
            ClockState::NotStarted => self.state = ClockState::Player(self.first_to_move),
            ClockState::Pause => (),
            ClockState::Player(p) => match p {
                Player::Player1 => {
                    self.state = ClockState::Player(Player::Player2);
                    self.player1.0 += self.increment;
                }
                Player::Player2 => {
                    self.state = ClockState::Player(Player::Player1);
                    self.player2.0 += self.increment;
                }
            },
        }
    }

    pub fn tick_timer(&mut self) {
        let millisec = Duration::from_millis(TIMER_TICK);
        match self.state {
            ClockState::NotStarted | ClockState::Pause => (),
            ClockState::Player(p) => match p {
                Player::Player1 => {
                    self.player1.0 = self.player1.0.saturating_sub(millisec);
                }
                Player::Player2 => {
                    self.player2.0 = self.player2.0.saturating_sub(millisec);
                }
            },
        }
    }

    pub fn is_time_out(&self) -> bool {
        if self.player1.0 == Duration::ZERO || self.player2.0 == Duration::ZERO {
            true
        } else {
            false
        }
    }

    pub fn pause(&mut self, resume_player: Player) {
        match self.state {
            ClockState::Pause => self.state = ClockState::Player(self.resume_player),
            _ => {
                self.resume_player = resume_player;
                self.state = ClockState::Pause;
            }
        }
    }

    fn state_to_style_pure(
        state: ClockState,
        resume: Player,
        time1: Duration,
        time2: Duration,
    ) -> [Style; 2] {
        let active_style = Style::default().fg(Color::LightGreen);
        let inactive_style = Style::default().fg(Color::from_u32(0x003f3f3f));
        let burning_clock_style = Style::default().fg(Color::LightRed);
        match state {
            ClockState::Player(p) => match p {
                Player::Player1 => [
                    if Clock::burning(time1) {
                        burning_clock_style
                    } else {
                        active_style
                    },
                    inactive_style,
                ],
                Player::Player2 => [
                    inactive_style,
                    if Clock::burning(time2) {
                        burning_clock_style
                    } else {
                        active_style
                    },
                ],
            },
            ClockState::NotStarted => [inactive_style, inactive_style],
            ClockState::Pause => {
                Clock::state_to_style_pure(ClockState::Player(resume), resume, time1, time2)
            }
        }
    }

    pub fn flip_first_to_move(&mut self) {
        self.first_to_move = match self.first_to_move {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            increment: Duration::from_secs(1),
            player1: Time(Duration::from_secs(1)),
            player2: Time(Duration::from_secs(1)),
            state: ClockState::NotStarted,
            resume_player: Player::Player1,
            time_ctrl: TimeCtrl::Tab1,
            first_to_move: Player::default(),
        }
    }
}

impl Widget for Clock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Percentage(50), Percentage(50)])
            .split(area);

        let l2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Fill(3), Min(10), Fill(1)])
            .split(layout[0]);
        let l3 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Fill(3), Min(10), Fill(1)])
            .split(layout[1]);

        let bottom_text = if matches!(self.state, ClockState::NotStarted) {
            " Hit <space> to start ".to_string()
        } else if self.is_time_out() {
            " Time out. Hit <enter> to continue ".to_string()
        } else if matches!(self.state, ClockState::Pause) {
            " Pause. Hit 'p' to resume ".to_string()
        } else {
            self.time_ctrl.to_string()
        };
        let instructions = Line::from(bottom_text.fg(Color::LightGreen).bold());
        let block = Block::default().title_bottom(instructions.centered());

        if matches!(self.state, ClockState::NotStarted) {
            let [left, right] =
                Layout::horizontal([Percentage(50), Percentage(50)]).areas(*buf.area());
            let [_, left] = Layout::vertical([Fill(1), Length(1)]).areas(left);
            let [_, right] = Layout::vertical([Fill(1), Length(1)]).areas(right);
            let mark = Line::from(" first to move ".fg(Color::Yellow).bold()).centered();
            mark.render(
                match self.first_to_move {
                    Player::Player1 => left,
                    Player::Player2 => right,
                },
                buf,
            );
        }

        let styles = Clock::state_to_style_pure(
            self.state,
            self.resume_player,
            self.player1.0,
            self.player2.0,
        );
        let p1 = Text::styled(self.player1.with_font(), styles[0]);
        let p2 = Text::styled(self.player2.with_font(), styles[1]);
        Paragraph::new(p1).centered().render(l2[1], buf);
        Paragraph::new(p2).centered().render(l3[1], buf);
        block.render(area, buf);
    }
}
