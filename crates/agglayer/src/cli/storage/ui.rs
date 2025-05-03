use std::{
    fmt::Display,
    ops::{AddAssign, SubAssign},
    path::{Path, PathBuf},
    sync::Arc,
};

use checking_tab::CheckingTab;
use queries_tab::QueriesTab;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{self, Line, Span},
    widgets::{Block, Paragraph, Tabs},
    Frame,
};

mod checking_tab;
mod queries_tab;

#[derive(Default, Copy, Clone)]
enum Tab {
    #[default]
    Checking = 0,
    Queries = 1,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Checking => write!(f, "Checking"),
            Tab::Queries => write!(f, "Queries"),
        }
    }
}

#[derive(Default)]
pub(crate) struct StorageUI {
    current_tab: Tab,
    pub(crate) checking_tab: CheckingTab,
    pub(crate) queries_tab: QueriesTab,
    show_help: bool,
}

impl StorageUI {
    pub fn new(storage_path: &Path) -> Self {
        let storage_path: Arc<PathBuf> = Arc::new(storage_path.into());

        Self {
            checking_tab: CheckingTab::new(storage_path.clone()),
            queries_tab: QueriesTab::new(storage_path.clone()),
            show_help: true,
            ..Default::default()
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let chunks = if self.show_help {
            Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Min(2),
            ])
            .split(frame.area())
        } else {
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area())
        };
        let tabs = [Tab::Checking, Tab::Queries]
            .iter()
            .map(|t| {
                text::Line::from(Span::styled(
                    t.to_string(),
                    Style::default().fg(Color::Green),
                ))
            })
            .collect::<Tabs>()
            .block(Block::bordered().title("Agglayer Storage UI"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(self.current_tab as usize);

        frame.render_widget(tabs, chunks[0]);
        if self.show_help {
            let block = Block::bordered().title("Help");

            let mut text = vec![
                Line::from("Press `TAB` to move between tabs"),
                Line::from("Press `?` to show/hide help"),
                Line::from("Press `q` to quit"),
            ];
            if let Tab::Queries = self.current_tab {
                text.push(Line::from(""));
                text.push(Line::from("Press  `j`, `k` to move in lists"));
                text.push(Line::from("Press `Esc` close latest horizontal block"));
            }
            let paragraph = Paragraph::new(text).block(block);

            frame.render_widget(paragraph, chunks[2]);
        }

        match self.current_tab {
            Tab::Checking => self.draw_checking(frame, chunks[1]),
            Tab::Queries => self.draw_queries(frame, chunks[1]),
        };
    }

    fn draw_checking(&mut self, frame: &mut Frame<'_>, area: ratatui::prelude::Rect) {
        self.checking_tab.update_state();
        self.checking_tab.draw(frame, area)
    }

    fn draw_queries(&mut self, frame: &mut Frame<'_>, area: ratatui::prelude::Rect) {
        self.queries_tab.draw(frame, area)
    }

    pub(crate) fn on_left(&mut self) {
        match self.current_tab {
            Tab::Checking => {}
            Tab::Queries => self.queries_tab.on_left(),
        }
    }
    pub(crate) fn on_right(&mut self) {
        match self.current_tab {
            Tab::Checking => {}
            Tab::Queries => self.queries_tab.on_right(),
        }
    }
    pub(crate) fn on_down(&mut self) {
        match self.current_tab {
            Tab::Checking => {}
            Tab::Queries => self.queries_tab.on_down(),
        }
    }
    pub(crate) fn on_up(&mut self) {
        match self.current_tab {
            Tab::Checking => {}
            Tab::Queries => self.queries_tab.on_up(),
        }
    }

    pub(crate) fn on_key(&mut self, key: char) {
        match self.current_tab {
            Tab::Checking => self.checking_tab.on_key(key),
            Tab::Queries => self.queries_tab.on_key(key),
        }
    }

    pub(crate) fn on_tab(&mut self) {
        self.current_tab += 1;
    }
    pub(crate) fn on_back_tab(&mut self) {
        self.current_tab -= 1;
    }

    pub(crate) fn on_enter(&mut self) {
        match self.current_tab {
            Tab::Checking => self.checking_tab.on_enter(),
            Tab::Queries => self.queries_tab.on_enter(),
        }
    }

    pub(crate) fn on_esc(&mut self) {
        match self.current_tab {
            Tab::Checking => self.checking_tab.on_esc(),
            Tab::Queries => self.queries_tab.on_esc(),
        }
    }

    pub(crate) fn on_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

impl AddAssign<usize> for Tab {
    fn add_assign(&mut self, rhs: usize) {
        debug_assert!(rhs == 1);

        *self = match *self {
            Tab::Checking => Tab::Queries,
            Tab::Queries => Tab::Checking,
        };
    }
}

impl SubAssign<usize> for Tab {
    fn sub_assign(&mut self, rhs: usize) {
        debug_assert!(rhs == 1);

        *self = match *self {
            Tab::Checking => Tab::Queries,
            Tab::Queries => Tab::Checking,
        };
    }
}
