use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Padding, Tabs, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub type CtrlOption = (Duration, Duration);

#[derive(Debug, PartialEq, Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum TimeCtrl {
    #[default]
    #[strum(to_string = "3 +2")]
    Tab1,
    #[strum(to_string = "1 +0")]
    Tab2,
    #[strum(to_string = "5 +3")]
    Tab3,
    #[strum(to_string = "10 +0")]
    Tab4,
}

impl TimeCtrl {
    pub fn previous(&mut self) {
        let current_index: usize = *self as usize;
        let previous_index = current_index.saturating_sub(1);
        *self = Self::from_repr(previous_index).unwrap_or(*self);
    }

    pub fn next(&mut self) {
        let current_index = *self as usize;
        let next_index = current_index.saturating_add(1);
        *self = Self::from_repr(next_index).unwrap_or(*self);
    }

    pub fn handle_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Right => self.next(),
            KeyCode::Left => self.previous(),
            // KeyCode::Enter => self.app,
            _ => {}
        }
    }

    pub fn to_duration(&self) -> CtrlOption {
        match self {
            TimeCtrl::Tab1 => (Duration::from_secs(180), Duration::from_secs(2)),
            TimeCtrl::Tab2 => (Duration::from_secs(60), Duration::from_secs(0)),
            TimeCtrl::Tab3 => (Duration::from_secs(300), Duration::from_secs(2)),
            TimeCtrl::Tab4 => (Duration::from_secs(600), Duration::from_secs(0)),
        }
    }
}

impl Widget for TimeCtrl {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Fill, Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0)]);
        let [_, tabs_area] = vertical.areas(area);
        let horizontal = Layout::horizontal([Fill(1), Min(0), Fill(1)]);
        let [_, tabs_area, _] = horizontal.areas(tabs_area);

        let titles = TimeCtrl::iter().map(TimeCtrl::title);
        let selected_tab_index = self as usize;
        Tabs::new(titles)
            .highlight_style(Style::default().fg(Color::LightGreen).bold().underlined())
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(tabs_area, buf);
        self.block().render(area, buf);
    }
}

impl TimeCtrl {
    /// Return tab's name as a styled `Line`
    pub fn title(self) -> Line<'static> {
        format!(" {self} ").fg(tailwind::SLATE.c200).into()
    }

    // fn render_tab0(self, area: Rect, buf: &mut Buffer) {
    //     Paragraph::new("Hello, World!")
    //         .block(self.block())
    //         .render(area, buf);
    // }

    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .padding(Padding::horizontal(1))
            .border_style(Color::LightGreen)
            .title(Line::from(" Choose time control ").centered())
    }

    // pub const fn palette(self) -> tailwind::Palette {
    //     match self {
    //         Self::Tab1 => tailwind::BLUE,
    //         Self::Tab2 => tailwind::EMERALD,
    //         Self::Tab3 => tailwind::INDIGO,
    //         Self::Tab4 => tailwind::RED,
    //     }
    // }
}
