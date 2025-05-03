use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use agglayer_storage::{
    columns::{
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::CertificatePerNetworkColumn,
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
        metadata::MetadataColumn,
        Codec, ColumnSchema, CERTIFICATE_HEADER_CF, CERTIFICATE_PER_NETWORK_CF,
        LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF, LOCAL_EXIT_TREE_PER_NETWORK_CF, METADATA_CF,
    },
    storage::state_db_cf_definitions,
};
use ratatui::widgets::TableState;
use rocksdb::Options;
use serde_json::json;

use super::Database;

pub(crate) enum BackendAction {
    Open(Arc<PathBuf>, Database),
    LoadCF(String),
    LoadEntries(usize),
}

#[derive(Default)]
pub(crate) struct DBBackend {
    pub(crate) action: VecDeque<BackendAction>,
    pub(crate) columns: Vec<String>,
    pub(crate) columns_table_state: TableState,
    pub(crate) entries: BTreeMap<String, String>,
    pub(crate) entries_table_state: TableState,
}

#[derive(Default)]
pub(crate) struct BackendTask {
    backend: Arc<RwLock<DBBackend>>,
    db: Option<rocksdb::DB>,
    current_cf: String,
}

impl BackendTask {
    pub(crate) fn new(backend: Arc<RwLock<DBBackend>>) -> Self {
        Self {
            backend,
            ..Default::default()
        }
    }

    fn iter_column<T: rocksdb::DBAccess>(
        iterator: &mut rocksdb::DBRawIteratorWithThreadMode<'_, T>,
        max: usize,
        cf: &str,
    ) -> BTreeMap<String, String> {
        let mut entries = BTreeMap::new();

        while iterator.valid() {
            if entries.len() >= max {
                break;
            }

            if let Some((bytes_key, bytes_value)) = iterator.item() {
                let (key, value) = match cf {
                    CERTIFICATE_HEADER_CF => (
                        <CertificateHeaderColumn as ColumnSchema>::Key::decode(bytes_key)
                            .unwrap()
                            .to_string(),
                        serde_json::to_string_pretty(
                            &<CertificateHeaderColumn as ColumnSchema>::Value::decode(bytes_value)
                                .unwrap(),
                        )
                        .unwrap(),
                    ),
                    CERTIFICATE_PER_NETWORK_CF => (
                        <CertificatePerNetworkColumn as ColumnSchema>::Key::decode(bytes_key)
                            .unwrap()
                            .to_string(),
                        serde_json::to_string_pretty(
                            &<CertificatePerNetworkColumn as ColumnSchema>::Value::decode(
                                bytes_value,
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    ),
                    LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF => {
                        let value: SettledCertificate = <LatestSettledCertificatePerNetworkColumn as ColumnSchema>::Value::decode(
                                                        bytes_value,
                                                    )
                                                    .unwrap();
                        let value = json!({
                            "certificateId": value.0,
                            "height": value.1,
                            "epoch": value.2,
                            "indexInEpoch": value.3,
                        });
                        (
                        <LatestSettledCertificatePerNetworkColumn as ColumnSchema>::Key::decode(bytes_key)
                            .unwrap()
                            .to_string(),
                        serde_json::to_string_pretty(
                            &value,
                        )
                        .unwrap(),
                    )
                    }
                    METADATA_CF => (
                        <MetadataColumn as ColumnSchema>::Key::decode(bytes_key)
                            .unwrap()
                            .to_string(),
                        serde_json::to_string_pretty(
                            &<MetadataColumn as ColumnSchema>::Value::decode(bytes_value).unwrap(),
                        )
                        .unwrap(),
                    ),
                    LOCAL_EXIT_TREE_PER_NETWORK_CF => (
                        <LocalExitTreePerNetworkColumn as ColumnSchema>::Key::decode(bytes_key)
                            .unwrap()
                            .to_string(),
                        serde_json::to_string_pretty(
                            &<LocalExitTreePerNetworkColumn as ColumnSchema>::Value::decode(
                                bytes_value,
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    ),
                    _ => (String::new(), String::new()),
                };
                entries.insert(key, value);
            }
            iterator.next();
        }

        entries
    }

    pub(crate) fn run(mut self) {
        loop {
            let mut backend = self.backend.write().unwrap();
            let action = backend.action.pop_front();
            drop(backend);

            match action {
                Some(BackendAction::Open(path, database)) => {
                    match database {
                        Database::Unknown => {}
                        Database::State => {
                            let mut options = Options::default();
                            options.create_if_missing(true);
                            options.create_missing_column_families(true);

                            self.db = Some(
                                rocksdb::DB::open_cf_descriptors_read_only(
                                    &options,
                                    path.join("state"),
                                    state_db_cf_definitions(),
                                    false,
                                )
                                .unwrap(),
                            );
                            let mut backend = self.backend.write().unwrap();
                            backend.columns = state_db_cf_definitions()
                                .iter()
                                .map(|cf| cf.name().to_string())
                                .collect::<Vec<_>>();
                            drop(backend);
                        }
                        Database::Pending => todo!(),
                        Database::Epoch(_) => todo!(),
                    };
                }

                Some(BackendAction::LoadCF(cf)) => {
                    self.current_cf = cf;
                }

                Some(BackendAction::LoadEntries(max)) => {
                    if let Some(db) = self.db.as_mut() {
                        let cf = db.cf_handle(&self.current_cf).unwrap();
                        let mut iterator = db.raw_iterator_cf(&cf);
                        iterator.seek_to_first();

                        let entries = Self::iter_column(&mut iterator, max, &self.current_cf);
                        let mut backend = self.backend.write().unwrap();

                        backend.entries = entries;
                        drop(backend);
                    }
                }

                None => {}
            }
        }
    }
}
