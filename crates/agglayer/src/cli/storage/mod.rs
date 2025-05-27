use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use agglayer_storage::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        debug_certificates::DebugCertificatesColumn,
        default_bincode_options,
        epochs::{
            certificates::CertificatePerIndexColumn, end_checkpoint::EndCheckpointColumn,
            start_checkpoint::StartCheckpointColumn,
        },
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
        metadata::MetadataColumn,
        nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        Codec, ColumnSchema, CERTIFICATE_HEADER_CF,
    },
    storage::{
        backup::BackupClient, debug_db_cf_definitions, epochs_db_cf_definitions,
        state_db_cf_definitions, Direction, ReadOptions, DB,
    },
    stores::{state::StateStore, StateReader},
    types::{MetadataKey, MetadataValue},
};
use agglayer_types::{
    Address, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Digest,
    EpochNumber, Height, LocalNetworkStateData, Metadata, NetworkId, PessimisticRootInput,
};
use clap::Subcommand;

mod app;
mod ui;

#[derive(Subcommand)]
pub enum Storage {
    Ui { storage_path: PathBuf },
    Rebuild { from_path: PathBuf },
    Test { storage_path: PathBuf },
}

impl Storage {
    pub fn ui(storage_path: &Path) -> anyhow::Result<()> {
        use std::io;
        use std::time::{Duration, Instant};

        use crossterm::event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        };
        use crossterm::execute;
        use crossterm::terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        };
        use ratatui::backend::CrosstermBackend;
        use ratatui::Terminal;

        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut ui = ui::StorageUI::new(storage_path);
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);
        loop {
            terminal.draw(|frame| ui.draw(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Left | KeyCode::Char('h') => ui.on_left(),
                            KeyCode::Up | KeyCode::Char('k') => ui.on_up(),
                            KeyCode::Right | KeyCode::Char('l') => ui.on_right(),
                            KeyCode::Down | KeyCode::Char('j') => ui.on_down(),
                            KeyCode::Tab => ui.on_tab(),
                            KeyCode::BackTab => ui.on_back_tab(),
                            KeyCode::Enter => ui.on_enter(),
                            KeyCode::Esc => ui.on_esc(),
                            KeyCode::Char('?') => ui.on_help(),
                            KeyCode::Char('q') => break,
                            KeyCode::Char(key) => ui.on_key(key),
                            _ => {}
                        }
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                // app.on_tick();
                last_tick = Instant::now();
            }
            // if app.should_quit {
            // return Ok(());
            // }
        }

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Storage::Ui { storage_path } => {
                _ = Self::ui(storage_path);
            }
            Storage::Test { storage_path } => {
                let db = Arc::new(agglayer_storage::storage::DB::open_cf_read_only(
                    &storage_path.join("state"),
                    state_db_cf_definitions(),
                )?);
                let key =
                    hex::decode("504cd18d93f6a80a877a1c8db881d559902a0a2edf294d526574e6a17ca7ee3b")
                        .unwrap();

                let cf = db.rocksdb.cf_handle(CERTIFICATE_HEADER_CF).unwrap();
                let bytes = db.rocksdb.get_cf(cf, &key);

                if let Ok(Some(bytes)) = bytes {
                    println!("{:?}", bytes);

                    use bincode::config::Options;
                    let parsed_length = (4 + 8 + 1 + 1 + 32 + 32 + 32 + 32);
                    let certificate: Result<CertificateHeader, _> =
                        // default_bincode_options().deserialize(&bytes[..parsed_length]);
                    default_bincode_options().deserialize(&bytes);
                    println!("{:?}", certificate);
                    println!("{:?}", &bytes[parsed_length..]);
                    #[derive(serde::Deserialize, Debug)]
                    pub struct CertificateHeader {
                        // 32
                        pub network_id: NetworkId,
                        pub height: Height,
                        pub epoch_number: Option<EpochNumber>,
                        pub certificate_index: Option<CertificateIndex>,
                        pub certificate_id: CertificateId,
                        pub prev_local_exit_root: Digest,
                        pub new_local_exit_root: Digest,
                        pub metadata: Metadata,
                        pub status: CertificateStatus,
                        pub settlement_tx_hash: Option<Digest>,
                    }
                }
            }
            Storage::Rebuild { from_path } => {
                print!("Opening state database...");
                let db = Arc::new(agglayer_storage::storage::DB::open_cf_read_only(
                    &from_path.join("state"),
                    state_db_cf_definitions(),
                )?);
                println!("OK");

                print!("Reading metadata columns...");
                read_column::<MetadataColumn>(db.clone())?;
                println!("OK");

                print!("Reading certificate header columns...");
                read_column::<CertificateHeaderColumn>(db.clone())?;
                println!("OK");

                print!("Reading certificate per network columns...");
                read_column::<CertificatePerNetworkColumn>(db.clone())?;
                println!("OK");

                print!("Reading latest settled certificate per network columns...");
                read_column::<LatestSettledCertificatePerNetworkColumn>(db.clone())?;
                println!("OK");

                print!("Reading Local exit tree per network columns...");
                read_column::<LocalExitTreePerNetworkColumn>(db.clone())?;
                println!("OK");

                print!("Reading balance tree per network columns...");
                read_column::<BalanceTreePerNetworkColumn>(db.clone())?;
                println!("OK");

                print!("Reading nullifier tree per network columns...");
                read_column::<NullifierTreePerNetworkColumn>(db.clone())?;
                println!("OK");

                let latest_settled_epoch = db
                    .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)?
                    .map(|value| {
                        if let MetadataValue::LatestSettledEpoch(epoch) = value {
                            epoch
                        } else {
                            panic!("Invalid latest settled epoch");
                        }
                    })
                    .unwrap_or(0);

                for i in 0..latest_settled_epoch + 1 {
                    print!("Opening epoch {}...", i);
                    let db = Arc::new(agglayer_storage::storage::DB::open_cf_read_only(
                        &from_path.join("epochs").join(format!("{}", i)),
                        epochs_db_cf_definitions(),
                    )?);
                    println!("OK");

                    print!("Epoch {} => Reading StartCheckpoint columns...", i);
                    read_column::<StartCheckpointColumn>(db.clone())?;
                    println!("OK");

                    print!("Epoch {} => Reading EndCheckpoint columns...", i);
                    read_column::<EndCheckpointColumn>(db.clone())?;
                    println!("OK");

                    print!("Epoch {} => Reading Certificate per index columns...", i);
                    read_column::<CertificatePerIndexColumn>(db.clone())?;
                    println!("OK");
                }

                println!("Rebuilding network states");
                let mut networks = db
                    .raw_iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
                        ReadOptions::default(),
                        Direction::Forward,
                    )?;

                let state_store = StateStore::new(db.clone(), BackupClient::noop());

                while networks.valid() {
                    let network_id = NetworkId::decode(networks.key().unwrap()).unwrap();
                    println!(
                        "Network {} => Reading settled certificates and rebuilding network \
                         state...",
                        network_id
                    );

                    let from_db = {
                        let lns = state_store
                            .read_local_network_state(network_id)
                            .unwrap()
                            .unwrap();

                        lns.get_roots()
                    };

                    let value = SettledCertificate::decode(networks.value().unwrap()).unwrap();
                    let max_settled_height = value.1 + 1;

                    let mut network_state = LocalNetworkStateData::default();
                    let mut current_height = 0;

                    while current_height < max_settled_height {
                        let certificate_id =
                            db.get::<CertificatePerNetworkColumn>(
                                &certificate_per_network::Key::new(*network_id, current_height),
                            )
                            .unwrap()
                            .unwrap();

                        let header = db
                            .get::<CertificateHeaderColumn>(&certificate_id)
                            .unwrap()
                            .unwrap();

                        assert!(header.epoch_number.is_some());

                        let (cert_index, epoch_number) = (
                            header.certificate_index.unwrap(),
                            header.epoch_number.unwrap(),
                        );

                        let epoch = agglayer_storage::storage::DB::open_cf_read_only(
                            &from_path.join("epochs").join(format!("{}", epoch_number)),
                            epochs_db_cf_definitions(),
                        )?;

                        let mut certificate = epoch
                            .get::<CertificatePerIndexColumn>(&cert_index)
                            .unwrap()
                            .unwrap();

                        if certificate.network_id != network_id {
                            let debug = agglayer_storage::storage::DB::open_cf_read_only(
                                &from_path.join("debug"),
                                debug_db_cf_definitions(),
                            )?;

                            let debug_certificate = debug
                                .get::<DebugCertificatesColumn>(&certificate_id)?
                                .unwrap();

                            let end_checkpoint = epoch
                                .iter_with_direction::<EndCheckpointColumn>(
                                    ReadOptions::default(),
                                    Direction::Forward,
                                )?
                                .filter_map(|v| v.ok())
                                .collect::<BTreeMap<NetworkId, Height>>();

                            let end_height = end_checkpoint.get(&network_id).unwrap();
                            if *end_height == current_height {
                                println!(
                                    "Certificate mismatch in epoch {} at index {}, we can correct \
                                     the certificate {} in the epoch",
                                    epoch_number, cert_index, certificate_id
                                );
                                certificate = debug_certificate;
                            } else {
                                panic!(
                                    "Certificate mismatch in epoch {} at index {}, we cannot \
                                     correct the certificate {} in the epoch",
                                    epoch_number, cert_index, certificate_id
                                );
                            }
                        }
                        assert_eq!(
                            certificate.network_id, network_id,
                            "wrong network lookup for ({network_id}|{current_height}), \
                             epoch:{:?}, idx: {:?}. got: {}, expected: {}",
                            epoch_number, cert_index, certificate.network_id, network_id
                        );
                        assert_eq!(
                            certificate.height, current_height,
                            "wrong height lookup for ({network_id}|{current_height}), epoch: \
                             {:?}, idx: {:?}. got: {}, expected: {}",
                            epoch_number, cert_index, certificate.height, current_height
                        );

                        print!(
                            "Applying certificate (network:height => {}:{}). epoch: {}, idx in \
                             epoch: {} (#be:{}, #ibe:{})...",
                            certificate.network_id,
                            certificate.height,
                            epoch_number,
                            cert_index,
                            certificate.bridge_exits.len(),
                            certificate.imported_bridge_exits.len(),
                        );

                        network_state.apply_certificate(
                            &certificate,
                            Address::ZERO,
                            Digest::ZERO,
                            PessimisticRootInput::Fetched(Digest::ZERO),
                            None,
                        )?;

                        println!("OK");
                        current_height += 1;
                    }

                    assert!(
                        current_height == max_settled_height,
                        "Current height {} does not match max settled height {}",
                        current_height,
                        max_settled_height
                    );

                    let roots: pessimistic_proof::local_state::StateCommitment =
                        network_state.get_roots();

                    assert_eq!(
                        from_db, roots,
                        "target mismatch. from db: {:?}, re-computed: {:?}",
                        from_db, roots
                    );

                    println!("Network {} is in sync ✅", network_id);

                    networks.next();
                }
            }
        }

        Ok(())
    }
}

pub fn read_column<T>(db: Arc<DB>) -> anyhow::Result<()>
where
    T: ColumnSchema,
    T::Key: std::fmt::Debug,
{
    let mut iterator =
        db.raw_iter_with_direction::<T>(ReadOptions::default(), Direction::Forward)?;

    while iterator.valid() {
        if let Some((bytes_key, bytes_value)) = iterator.item() {
            let key = T::Key::decode(bytes_key)?;
            let value = T::Value::decode(bytes_value);

            if let Err(err) = value {
                println!("Error decoding column key: {:?}", key);
                println!("bytes representation of the value:");
                println!("{:?}", bytes_value);
                println!("bytes representation of the key:");
                println!("{:?}", bytes_key);

                panic!("Error reading column: {:?}", err);
            }

            iterator.next();
        }
    }

    Ok(())
}
