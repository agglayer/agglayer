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
    storage::{epochs_db_cf_definitions, state_db_cf_definitions, Direction, ReadOptions, DB},
    types::{MetadataKey, MetadataValue, SmtKey, SmtKeyType, SmtValue},
};
use agglayer_types::{Address, Digest, LocalNetworkStateData, NetworkId, PessimisticRootInput};
use clap::Subcommand;
use pessimistic_proof::keccak::keccak256_combine;

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

                while networks.valid() {
                    let network_id = NetworkId::decode(networks.key().unwrap()).unwrap();
                    println!(
                        "Network {} => Reading settled certificates and rebuilding network \
                         state...",
                        network_id
                    );
                    let value = SettledCertificate::decode(networks.value().unwrap()).unwrap();
                    let max_settled_height = value.1;

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
                        let epoch = agglayer_storage::storage::DB::open_cf(
                            &from_path
                                .join("epochs")
                                .join(format!("{}", header.epoch_number.unwrap())),
                            epochs_db_cf_definitions(),
                        )?;

                        let certificate = epoch
                            .get::<CertificatePerIndexColumn>(&header.certificate_index.unwrap())
                            .unwrap()
                            .unwrap();

                        print!("Applying certificate {}...", certificate_id);
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
                    let balance_root = db
                        .get::<BalanceTreePerNetworkColumn>(&SmtKey::new(
                            *network_id,
                            SmtKeyType::Root,
                        ))
                        .unwrap()
                        .map(|value| {
                            if let SmtValue::Node(left, right) = value {
                                keccak256_combine([left.as_ref(), right.as_ref()])
                            } else {
                                panic!("Unexpected SmtValue type");
                            }
                        })
                        .unwrap();
                    let roots = network_state.get_roots();
                    assert!(
                        roots.balance_root == balance_root,
                        "Balance root mismatch expected {}, calculated {}",
                        roots.balance_root,
                        balance_root,
                    );

                    println!("Network {} is connected", network_id);

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
