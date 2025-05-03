use std::sync::{Arc, RwLock};

use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize as _},
    text::Line,
    widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, Widget},
};

use super::DBBackend;

#[derive(Default)]
pub(crate) struct ColumnsWidget {
    pub(crate) backend: Arc<RwLock<DBBackend>>,
    pub(crate) is_focused: bool,
}

impl ColumnsWidget {
    pub(crate) fn new(backend: Arc<RwLock<DBBackend>>) -> Self {
        Self {
            backend,
            is_focused: false,
        }
    }

    pub(crate) fn on_down(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            backend.columns_table_state.select_next();
        }
    }

    pub(crate) fn on_up(&self) {
        if self.is_focused {
            let mut backend = self.backend.write().unwrap();
            backend.columns_table_state.select_previous();
        }
    }
}

impl Widget for &ColumnsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut block = Block::bordered().title("Columns");
        if self.is_focused {
            block = block
                .border_style(Style::default().yellow())
                .title(Line::from("h/l to move between columns, ENTER to select").right_aligned());
        }
        let mut state = self.backend.write().unwrap();
        let rows = state
            .columns
            .iter()
            .map(|cf| Row::new(vec![cf.clone()]))
            .collect::<Vec<_>>();

        let widths = [Constraint::Min(5)];
        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.columns_table_state);
    }
}
