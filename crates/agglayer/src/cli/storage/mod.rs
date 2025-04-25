use std::{path::PathBuf, sync::Arc};

use agglayer_storage::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
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
        Codec, ColumnSchema,
    },
    storage::{
        backup::BackupClient, epochs_db_cf_definitions, state_db_cf_definitions, Direction,
        ReadOptions, DB,
    },
    stores::{state::StateStore, StateReader},
    types::{MetadataKey, MetadataValue},
};
use agglayer_types::{Address, Digest, LocalNetworkStateData, NetworkId, PessimisticRootInput};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Storage {
    Rebuild { from_path: PathBuf },
}

impl Storage {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Storage::Rebuild { from_path } => {
                print!("Opening state database...");
                let db = Arc::new(agglayer_storage::storage::DB::open_cf(
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
                    let db = Arc::new(agglayer_storage::storage::DB::open_cf(
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
                    println!("Target roots: {:?}", from_db);

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

                        let epoch = agglayer_storage::storage::DB::open_cf(
                            &from_path.join("epochs").join(format!("{}", epoch_number)),
                            epochs_db_cf_definitions(),
                        )?;

                        let certificate = epoch
                            .get::<CertificatePerIndexColumn>(&cert_index)
                            .unwrap()
                            .unwrap();
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
                            "Applying certificate ({}|{}). epoch: {}, idx: {} (#be:{}, #ibe:{})...",
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

                    println!("Network {} is connected ✅", network_id);

                    networks.next();
                }
            }
        }

        Ok(())
    }
}

fn read_column<T>(db: Arc<DB>) -> anyhow::Result<()>
where
    T: ColumnSchema,
    T::Key: std::fmt::Debug,
{
    let mut iterator =
        db.raw_iter_with_direction::<T>(ReadOptions::default(), Direction::Forward)?;

    while iterator.valid() {
        if let Some((bytes_key, bytes_value)) = iterator.item() {
            let key = T::Key::decode(&bytes_key)?;
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
