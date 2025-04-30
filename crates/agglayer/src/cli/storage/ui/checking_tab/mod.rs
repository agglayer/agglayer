use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use agglayer_storage::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::CertificatePerNetworkColumn,
        epochs::{
            certificates::CertificatePerIndexColumn, end_checkpoint::EndCheckpointColumn,
            start_checkpoint::StartCheckpointColumn,
        },
        latest_settled_certificate_per_network::LatestSettledCertificatePerNetworkColumn,
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
        metadata::MetadataColumn,
        nullifier_tree_per_network::NullifierTreePerNetworkColumn,
    },
    storage::{epochs_db_cf_definitions, state_db_cf_definitions},
    types::{MetadataKey, MetadataValue},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{self, Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::cli::storage::read_column;

#[derive(Default)]
pub(crate) struct CheckingTab {
    storage_path: Arc<PathBuf>,
    state: CheckingTabState,
    checking_task: CheckingTask,
    checking_task_handle: Option<JoinHandle<Result<(), anyhow::Error>>>,
}

impl CheckingTab {
    pub(crate) fn new(storage_path: Arc<PathBuf>) -> Self {
        Self {
            storage_path,
            ..Default::default()
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>, area: Rect) {
        match self.state {
            CheckingTabState::Initialize => {
                let text = vec![
                    text::Line::from(vec![
                        Span::from("In order to start checking the storage, press "),
                        Span::styled("v", Style::default().fg(Color::Blue)),
                    ]),
                    text::Line::from(""),
                    text::Line::from(vec![
                        Span::from("The configured storage path is: "),
                        Span::styled(
                            self.storage_path.to_string_lossy(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    text::Line::from(vec![
                        Span::styled(
                            "The storage is opened as read-only, but it may affect node \
                             performance.",
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::ITALIC),
                        ),
                        Span::raw("."),
                    ]),
                ];
                let block = Block::bordered().title(Span::styled(
                    "",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                frame.render_widget(paragraph, area);
            }
            CheckingTabState::Analyzing | CheckingTabState::Finished => {
                let state = self.checking_task.state.write().unwrap();

                let mut text = vec![];
                // Status line for state opening DB
                self.add_status_line(
                    &mut text,
                    vec![
                        Span::raw("Opening state database at "),
                        Span::styled(
                            self.storage_path.join("state").display().to_string(),
                            Style::default().fg(Color::Blue),
                        ),
                        Span::raw("..."),
                    ],
                    &state.db_state_opened,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading metadata columns...")],
                    &state.metadata_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading certificate header columns...")],
                    &state.certificate_header_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading certificate per network columns...")],
                    &state.certificate_per_network_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw(
                        "Reading latest settled certificate per network columns...",
                    )],
                    &state.certificate_per_network_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading Local exit tree per network columns...")],
                    &state.local_exit_tree_per_network_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading balance tree per network columns...")],
                    &state.balance_tree_per_network_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading nullifier tree per network columns...")],
                    &state.nullifier_tree_per_network_column_read,
                );

                self.add_status_line(
                    &mut text,
                    vec![Span::raw("Reading the latest settled epoch...")],
                    &state.latest_settled_epoch,
                );

                if state.epoch_checking.1 != StepStatus::NotStarted {
                    self.add_status_line(
                        &mut text,
                        vec![
                            Span::raw("Checking epoch "),
                            Span::styled(
                                format!("{} ", state.epoch_checking.0),
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("..."),
                        ],
                        &state.epoch_checking.1,
                    );
                }

                let content = match self.state {
                    CheckingTabState::Initialize => "",
                    CheckingTabState::Analyzing => "Analyzing",
                    CheckingTabState::Finished => "Finished, reexecute using v",
                };
                let block = Block::bordered().title(Span::styled(
                    content,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                frame.render_widget(paragraph, area);
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: char) {
        match key {
            'v' if [CheckingTabState::Initialize, CheckingTabState::Finished]
                .contains(&self.state) =>
            {
                self.state = CheckingTabState::Analyzing;
                self.checking_task = CheckingTask::default();
                self.checking_task_handle = Some(self.checking_task.run(self.storage_path.clone()));
            }
            _ => {}
        }
    }

    fn add_status_line<'a>(
        &self,
        text: &mut Vec<Line<'a>>,
        mut line: Vec<Span<'a>>,
        status: &'a StepStatus,
    ) {
        match status {
            StepStatus::NotStarted => {
                let line = Line::from(line);
                text.push(line);
            }
            StepStatus::InProgress => {
                line.push(Span::styled(
                    "processing...",
                    Style::default().yellow().italic(),
                ));
                let line = Line::from(line);
                text.push(line);
            }
            StepStatus::Ok => {
                line.push(Span::styled("OK", Style::default().green()));
                let line = Line::from(line);
                text.push(line);
            }

            StepStatus::OkWithMessage(value) => {
                line.push(Span::styled(value, Style::default().green()));
                let line = Line::from(line);
                text.push(line);
            }

            StepStatus::Err(error) => {
                line.push(Span::styled("FAILED", Style::default().red().bold()));
                let line = Line::from(line);
                text.push(line);
                text.push(Line::from(Span::styled(error, Style::default().red())));
            }
        }
    }

    pub(crate) fn update_state(&mut self) {
        if let Some(handle) = self.checking_task_handle.as_ref() {
            if handle.is_finished() {
                self.state = CheckingTabState::Finished;
            }
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum CheckingTabState {
    #[default]
    Initialize,
    Analyzing,
    Finished,
}

#[derive(Default, Clone)]
pub(crate) struct CheckingTask {
    pub(crate) state: Arc<RwLock<CheckingTaskState>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum StepStatus {
    #[default]
    NotStarted,
    InProgress,
    Ok,
    OkWithMessage(String),
    Err(String),
}

#[derive(Debug, Default)]
pub(crate) struct CheckingTaskState {
    db_state_opened: StepStatus,
    metadata_column_read: StepStatus,
    certificate_header_column_read: StepStatus,
    certificate_per_network_column_read: StepStatus,
    latest_settled_certificate_per_network_column_read: StepStatus,
    local_exit_tree_per_network_column_read: StepStatus,
    balance_tree_per_network_column_read: StepStatus,
    nullifier_tree_per_network_column_read: StepStatus,
    latest_settled_epoch: StepStatus,
    epoch_checking: (u64, StepStatus),
}

impl CheckingTask {
    fn run(&self, storage_path: Arc<PathBuf>) -> JoinHandle<Result<(), anyhow::Error>> {
        let task = self.clone();
        std::thread::spawn(move || {
            let mut state = task.state.write().unwrap();
            state.db_state_opened = StepStatus::InProgress;
            drop(state);
            let db = agglayer_storage::storage::DB::open_cf_read_only(
                &storage_path.join("state"),
                state_db_cf_definitions(),
            )
            .map(|db| {
                let mut state = task.state.write().unwrap();
                state.db_state_opened = StepStatus::Ok;
                Arc::new(db)
            })
            .map_err(|err| {
                let mut state = task.state.write().unwrap();
                state.db_state_opened = StepStatus::Err(err.to_string());
                anyhow::anyhow!("failed to open state db")
            })?;

            let mut state = task.state.write().unwrap();
            state.metadata_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<MetadataColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.metadata_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.metadata_column_read = StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read metadata")
                })?;

            let mut state = task.state.write().unwrap();
            state.certificate_header_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<CertificateHeaderColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.certificate_header_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.certificate_header_column_read = StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read certificate_header")
                })?;

            let mut state = task.state.write().unwrap();
            state.certificate_per_network_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<CertificatePerNetworkColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.certificate_per_network_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.certificate_per_network_column_read = StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read certificate_per_network")
                })?;

            let mut state = task.state.write().unwrap();
            state.latest_settled_certificate_per_network_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<LatestSettledCertificatePerNetworkColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.latest_settled_certificate_per_network_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.latest_settled_certificate_per_network_column_read =
                        StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read latest_settled_certificate_per_network")
                })?;

            let mut state = task.state.write().unwrap();
            state.local_exit_tree_per_network_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<LocalExitTreePerNetworkColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.local_exit_tree_per_network_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.local_exit_tree_per_network_column_read =
                        StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read local_exit_tree_per_network")
                })?;

            let mut state = task.state.write().unwrap();
            state.balance_tree_per_network_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<BalanceTreePerNetworkColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.balance_tree_per_network_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.balance_tree_per_network_column_read = StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read balance_tree_per_network")
                })?;

            let mut state = task.state.write().unwrap();
            state.nullifier_tree_per_network_column_read = StepStatus::InProgress;
            drop(state);
            read_column::<NullifierTreePerNetworkColumn>(db.clone())
                .map(|_| {
                    let mut state = task.state.write().unwrap();
                    state.nullifier_tree_per_network_column_read = StepStatus::Ok;
                })
                .map_err(|err| {
                    let mut state = task.state.write().unwrap();
                    state.nullifier_tree_per_network_column_read = StepStatus::Err(err.to_string());
                    anyhow::anyhow!("failed to read nullifier_tree_per_network")
                })?;

            let mut state = task.state.write().unwrap();
            state.latest_settled_epoch = StepStatus::InProgress;
            drop(state);
            let latest_settled_epoch = db
                .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)?
                .map(|value| {
                    if let MetadataValue::LatestSettledEpoch(epoch) = value {
                        let mut state = task.state.write().unwrap();

                        state.latest_settled_epoch =
                            StepStatus::OkWithMessage(format!("epoch = {}", epoch));

                        Ok(epoch)
                    } else {
                        let mut state = task.state.write().unwrap();

                        let err = anyhow::anyhow!(
                            "Deserialized an unexpected field that is not a 'LatestSettledEpoch'"
                        );
                        state.latest_settled_epoch = StepStatus::Err(err.to_string());

                        Err(err)
                    }
                })
                .transpose()?;

            if let Some(latest_settled_epoch) = latest_settled_epoch {
                for i in 0..latest_settled_epoch + 1 {
                    let mut state = task.state.write().unwrap();
                    let epoch = i;
                    let status = StepStatus::InProgress;
                    state.epoch_checking = (epoch, status);
                    drop(state);

                    let db = agglayer_storage::storage::DB::open_cf_read_only(
                        &storage_path.join("epochs").join(format!("{}", i)),
                        epochs_db_cf_definitions(),
                    )
                    .map(Arc::new)
                    .map_err(|err| {
                        let mut state = task.state.write().unwrap();
                        state.epoch_checking = (epoch, StepStatus::Err(err.to_string()));
                        anyhow::anyhow!("failed to open epoch {} db", i)
                    })?;

                    read_column::<StartCheckpointColumn>(db.clone()).map_err(|err| {
                        let mut state = task.state.write().unwrap();
                        state.epoch_checking = (epoch, StepStatus::Err(err.to_string()));
                        anyhow::anyhow!("failed to read start_checkpoint for epoch {}", epoch)
                    })?;

                    read_column::<EndCheckpointColumn>(db.clone()).map_err(|err| {
                        let mut state = task.state.write().unwrap();
                        state.epoch_checking = (epoch, StepStatus::Err(err.to_string()));
                        anyhow::anyhow!("failed to read end_checkpoint for epoch {}", epoch)
                    })?;

                    read_column::<CertificatePerIndexColumn>(db.clone()).map_err(|err| {
                        let mut state = task.state.write().unwrap();
                        state.epoch_checking = (epoch, StepStatus::Err(err.to_string()));
                        anyhow::anyhow!("failed to read certificate_per_index for epoch {}", epoch)
                    })?;
                }

                let mut state = task.state.write().unwrap();
                let status = StepStatus::OkWithMessage("all epochs are readable".into());
                state.epoch_checking = (state.epoch_checking.0, status);
                drop(state);
            }

            Ok::<_, anyhow::Error>(())
        })
    }
}
