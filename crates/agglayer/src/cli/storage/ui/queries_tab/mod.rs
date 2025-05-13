use std::{
    ops::{AddAssign, SubAssign},
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub(crate) use backend::{BackendAction, BackendTask, DBBackend};
use columns_widget::ColumnsWidget;
use details_widget::DetailsWidget;
use entries_widget::EntriesWidget;
use epochs_widget::EpochsWidget;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize as _},
    text::{self, Line, Span},
    widgets::{Block, Tabs, Widget},
    Frame,
};

mod columns_widget;
mod details_widget;
mod entries_widget;
mod epochs_widget;

mod backend;

#[derive(Default)]
pub(crate) struct QueriesTab {
    storage_path: Arc<PathBuf>,
    database_selector: DatabaseSelectorWidget,
    columns: ColumnsWidget,
    details: DetailsWidget,
    entries: EntriesWidget,
    epochs: EpochsWidget,
    current_focus: CurrentFocus,
}

#[derive(Default)]
enum CurrentFocus {
    #[default]
    Database,
    Epochs,
    Columns,
    Entries,
}

impl QueriesTab {
    pub(crate) fn new(storage_path: Arc<PathBuf>) -> Self {
        let backend = Arc::new(RwLock::new(DBBackend::default()));
        let task = BackendTask::new(backend.clone());
        std::thread::spawn(|| task.run());

        Self {
            storage_path,
            database_selector: DatabaseSelectorWidget {
                is_focused: true,
                ..Default::default()
            },
            columns: ColumnsWidget::new(backend.clone()),
            entries: EntriesWidget::new(backend.clone()),
            details: DetailsWidget::new(backend.clone()),
            epochs: EpochsWidget::new(backend.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>, area: Rect) {
        let [database_selector_area, epochs_area, body_area] =
            if let CurrentFocus::Epochs = self.current_focus {
                let vertical = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ]);
                vertical.areas(area)
            } else {
                let vertical = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(0),
                    Constraint::Fill(1),
                ]);
                vertical.areas(area)
            };

        let [columns_area, entries_area, details_area] = match self.current_focus {
            CurrentFocus::Database | CurrentFocus::Epochs => Layout::horizontal([
                Constraint::Percentage(0),
                Constraint::Percentage(0),
                Constraint::Percentage(0),
            ])
            .areas(body_area),
            CurrentFocus::Columns => Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(0),
                Constraint::Percentage(0),
            ])
            .areas(body_area),
            CurrentFocus::Entries if !self.details.is_focused => Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Percentage(80),
                Constraint::Percentage(0),
            ])
            .areas(body_area),
            CurrentFocus::Entries => Layout::horizontal([
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(70),
            ])
            .areas(body_area),
        };

        let [columns_area] = Layout::horizontal([Constraint::Min(30)]).areas(columns_area);

        // Draw database selector
        frame.render_widget(&self.database_selector, database_selector_area);
        frame.render_widget(&self.epochs, epochs_area);
        frame.render_widget(&self.columns, columns_area);
        frame.render_widget(&self.entries, entries_area);
        frame.render_widget(&self.details, details_area);
    }

    pub(crate) fn on_down(&mut self) {
        match self.current_focus {
            CurrentFocus::Columns => {
                self.columns.on_down();
            }
            CurrentFocus::Entries => {
                self.entries.on_down();
            }
            _ => {}
        }
    }

    pub(crate) fn on_up(&mut self) {
        match self.current_focus {
            CurrentFocus::Columns => {
                self.columns.on_up();
            }
            CurrentFocus::Entries => {
                self.entries.on_up();
            }
            _ => {}
        }
    }

    pub(crate) fn on_left(&mut self) {
        if let CurrentFocus::Database = self.current_focus {
            self.database_selector.on_left();
        }
        if let CurrentFocus::Epochs = self.current_focus {
            self.epochs.on_left();
        }
    }

    pub(crate) fn on_right(&mut self) {
        if let CurrentFocus::Database = self.current_focus {
            self.database_selector.on_right();
        }

        if let CurrentFocus::Epochs = self.current_focus {
            self.epochs.on_right();
        }
    }

    pub(crate) fn on_key(&mut self, _key: char) {}

    pub(crate) fn on_enter(&mut self) {
        match self.current_focus {
            CurrentFocus::Database => {
                if let Some(database) = self.database_selector.on_enter() {
                    if let Database::Epoch(_) = database {
                        self.current_focus = CurrentFocus::Epochs;

                        let mut backend = self.columns.backend.write().unwrap();
                        backend
                            .action
                            .push_back(BackendAction::Open(self.storage_path.clone(), database));

                        self.database_selector.is_focused = false;
                        self.epochs.is_focused = true;
                        self.entries.is_focused = false;
                        self.columns.is_focused = false;

                        return;
                    }

                    self.current_focus = CurrentFocus::Columns;
                    let mut backend = self.columns.backend.write().unwrap();
                    backend
                        .action
                        .push_back(BackendAction::Open(self.storage_path.clone(), database));
                    self.database_selector.is_focused = false;
                    self.entries.is_focused = false;
                    self.columns.is_focused = true;
                }
            }
            CurrentFocus::Epochs => {}
            CurrentFocus::Columns => {
                let mut backend = self.columns.backend.write().unwrap();

                if let Some(index) = backend.columns_table_state.selected() {
                    let cf = backend.columns.get(index).cloned().unwrap();
                    backend.action.push_back(BackendAction::LoadCF(cf));
                    backend.action.push_back(BackendAction::LoadEntries(1000));
                    self.current_focus = CurrentFocus::Entries;
                    self.database_selector.is_focused = false;
                    self.columns.is_focused = false;
                    self.entries.is_focused = true;
                }
            }
            CurrentFocus::Entries => {
                let backend = self.entries.backend.write().unwrap();

                if let Some(_index) = backend.entries_table_state.selected() {
                    self.database_selector.is_focused = false;
                    self.columns.is_focused = false;
                    self.entries.is_focused = true;
                    self.details.is_focused = true;
                }
            }
        }
    }

    pub(crate) fn on_esc(&mut self) {
        match self.current_focus {
            CurrentFocus::Database => {
                self.database_selector.is_focused = true;
                self.columns.is_focused = false;
                self.entries.is_focused = false;
                self.details.is_focused = false;
            }
            CurrentFocus::Epochs => {
                self.database_selector.is_focused = true;
                self.epochs.is_focused = false;
                self.columns.is_focused = false;
                self.entries.is_focused = false;
                self.details.is_focused = false;
                self.current_focus = CurrentFocus::Database;
            }
            CurrentFocus::Columns => {
                if let Some(Database::Epoch(_)) = self.database_selector.on_enter() {
                    self.database_selector.is_focused = false;
                    self.epochs.is_focused = true;
                } else {
                    self.epochs.is_focused = false;
                    self.database_selector.is_focused = true;
                }
                self.columns.is_focused = false;
                self.entries.is_focused = false;
                self.details.is_focused = false;
                self.current_focus = CurrentFocus::Database;
            }
            CurrentFocus::Entries => {
                self.database_selector.is_focused = false;
                if self.details.is_focused {
                    self.details.is_focused = false;
                } else {
                    self.columns.is_focused = true;
                    self.entries.is_focused = false;
                    self.current_focus = CurrentFocus::Columns;
                }
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
pub(crate) enum Database {
    #[default]
    Unknown,
    State,
    Pending,
    Epoch(u64),
}

impl AddAssign<usize> for Database {
    fn add_assign(&mut self, rhs: usize) {
        debug_assert!(rhs == 1);

        *self = match *self {
            Database::Unknown => Database::Unknown,
            Database::State => Database::Pending,
            Database::Pending => Database::Epoch(0),
            Database::Epoch(_) => Database::State,
        };
    }
}

impl SubAssign<usize> for Database {
    fn sub_assign(&mut self, rhs: usize) {
        debug_assert!(rhs == 1);

        *self = match *self {
            Database::Unknown => Database::Unknown,
            Database::State => Database::Epoch(0),
            Database::Pending => Database::State,
            Database::Epoch(_) => Database::Pending,
        };
    }
}
impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::Unknown => write!(f, "Unknown"),
            Database::State => write!(f, "State"),
            Database::Pending => write!(f, "Pending"),
            Database::Epoch(epoch) => write!(f, "Epoch {}", epoch),
        }
    }
}

#[derive(Default)]
struct DatabaseSelector {
    epoch_database: u64,
    current_database: Option<usize>,
}

#[derive(Default)]
struct DatabaseSelectorWidget {
    state: Arc<RwLock<DatabaseSelector>>,
    is_focused: bool,
}
impl DatabaseSelectorWidget {
    fn on_left(&mut self) {
        if self.is_focused {
            let mut state = self.state.write().unwrap();
            if let Some(current_database) = state.current_database.as_mut() {
                let index = if *current_database == 0 {
                    2
                } else {
                    *current_database - 1
                };
                *current_database = index % 3;
            } else {
                state.current_database = Some(0);
            }
        }
    }
    fn on_right(&mut self) {
        if self.is_focused {
            let mut state = self.state.write().unwrap();
            if let Some(current_database) = state.current_database.as_mut() {
                *current_database = (*current_database + 1) % 3;
            } else {
                state.current_database = Some(0);
            }
        }
    }

    fn on_enter(&mut self) -> Option<Database> {
        if self.is_focused {
            let state = self.state.write().unwrap();
            if let Some(selected_database) = state.current_database {
                return match selected_database {
                    0 => Some(Database::State),
                    1 => Some(Database::Pending),
                    _ => None,
                };
            }
        }

        None
    }
}

impl Widget for &DatabaseSelectorWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let state = self.state.write().unwrap();

        let mut block = Block::bordered().title("Selected database");
        if self.is_focused {
            block = block
                .border_style(Style::default().yellow())
                .title(Line::from("h/l to move between database, ENTER to select").right_aligned());
        }
        let tabs = [
            "State",
            "Pending",
            &Database::Epoch(state.epoch_database).to_string(),
        ]
        .iter()
        .map(|t| {
            text::Line::from(Span::styled(
                t.to_string(),
                Style::default().fg(Color::Green),
            ))
        })
        .collect::<Tabs>()
        .block(block)
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(state.current_database);

        tabs.render(area, buf);
    }
}
