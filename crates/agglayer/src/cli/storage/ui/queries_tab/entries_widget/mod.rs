use std::sync::{Arc, RwLock};

use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize as _},
    text::Line,
    widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, Widget},
};

use super::DBBackend;

#[derive(Default)]
pub(crate) struct EntriesWidget {
    pub(crate) backend: Arc<RwLock<DBBackend>>,
    pub(crate) is_focused: bool,
}

impl EntriesWidget {
    pub(crate) fn new(backend: Arc<RwLock<DBBackend>>) -> Self {
        Self {
            backend,
            ..Default::default()
        }
    }

    pub(crate) fn on_down(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            backend.entries_table_state.select_next();
        }
    }

    pub(crate) fn on_up(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            backend.entries_table_state.select_previous();
        }
    }
}

impl Widget for &EntriesWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut block = Block::bordered().title("Entries");
        if self.is_focused {
            block = block
                .border_style(Style::default().yellow())
                .title(Line::from("h/l to move between database, ENTER to select").right_aligned());
        }

        let mut state = self.backend.write().unwrap();

        let rows = if let Some(_cf) = state.columns_table_state.selected() {
            state
                .entries
                .keys()
                .map(|key| Row::new(vec![key.clone()]))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let widths = [Constraint::Min(5)];
        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.entries_table_state);
    }
}
