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
    text::{self, Span},
    widgets::{Block, Tabs},
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
}

impl StorageUI {
    pub fn new(storage_path: &Path) -> Self {
        let storage_path: Arc<PathBuf> = Arc::new(storage_path.into());

        Self {
            checking_tab: CheckingTab::new(storage_path.clone()),
            queries_tab: QueriesTab::new(storage_path.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
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

        match self.current_tab {
            Tab::Checking => self.draw_checking(frame, chunks[1]),
            // 1 => draw_second_tab(frame, app, chunks[1]),
            // 2 => draw_third_tab(frame, app, chunks[1]),
            _ => {}
        };
    }

    fn draw_checking(&mut self, frame: &mut Frame<'_>, area: ratatui::prelude::Rect) {
        self.checking_tab.update_state();
        self.checking_tab.draw(frame, area)
    }

    pub(crate) fn on_left(&mut self) {
        self.current_tab -= 1;
    }

    pub(crate) fn on_right(&mut self) {
        self.current_tab += 1;
    }
    pub(crate) fn on_down(&mut self) {
        todo!()
    }
    pub(crate) fn on_up(&mut self) {
        todo!()
    }

    pub(crate) fn on_key(&mut self, key: char) {
        match self.current_tab {
            Tab::Checking => self.checking_tab.on_key(key),
            Tab::Queries => todo!(),
        }
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
