use std::sync::{Arc, RwLock};

use ratatui::{
    layout::Rect,
    style::{Style, Stylize as _},
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use super::DBBackend;

#[derive(Default)]
pub(crate) struct DetailsWidget {
    pub(crate) backend: Arc<RwLock<DBBackend>>,
    pub(crate) is_focused: bool,
}

impl DetailsWidget {
    pub(crate) fn new(backend: Arc<RwLock<DBBackend>>) -> Self {
        Self {
            backend,
            is_focused: false,
        }
    }
}

impl Widget for &DetailsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut block = Block::bordered().title("Details");
        block = block
            .border_style(Style::default().yellow())
            .title(Line::from("h/l to move between database, ENTER to select").right_aligned());

        let state = self.backend.write().unwrap();
        if let Some(index) = state.entries_table_state.selected() {
            let text = state
                .entries
                .iter()
                .nth(index)
                .map(|(_, value)| value.clone())
                .unwrap_or_default();

            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
            paragraph.render(area, buf);
        }
    }
}
