use std::sync::{Arc, RwLock};

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize as _},
    text::{self, Line, Span},
    widgets::{Block, Tabs, Widget},
};

use super::DBBackend;

#[derive(Default)]
pub(crate) struct EpochsWidget {
    pub(crate) backend: Arc<RwLock<DBBackend>>,
    pub(crate) is_focused: bool,
}

impl EpochsWidget {
    pub(crate) fn new(backend: Arc<RwLock<DBBackend>>) -> Self {
        Self {
            backend,
            ..Default::default()
        }
    }

    pub(crate) fn on_right(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            let index = backend
                .open_epoch
                .map(|index| index.saturating_add(1))
                .unwrap_or_default();
            if backend.epochs.contains_key(&index) {
                backend.open_epoch.as_mut().map(|_| index);
            }
        }
    }
    pub(crate) fn on_left(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            let index = backend
                .open_epoch
                .map(|index| index.saturating_sub(1))
                .unwrap_or_default();

            if backend.epochs.contains_key(&index) {
                backend.open_epoch.as_mut().map(|_| index);
            }
        }
    }
}

impl Widget for &EpochsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut block = Block::bordered().title("Entries");
        if self.is_focused {
            block = block
                .border_style(Style::default().yellow())
                .title(Line::from("h/l to move between epochs, ENTER to select").right_aligned());
        }

        let state = self.backend.write().unwrap();

        let table: Tabs = state
            .epochs
            .values()
            .map(|t| {
                text::Line::from(Span::styled(
                    t.to_string(),
                    Style::default().fg(Color::Green),
                ))
            })
            .collect::<Tabs>()
            .block(block)
            .highlight_style(Style::default().on_blue())
            .select(state.open_epoch);

        table.render(area, buf);
    }
}
