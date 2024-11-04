use std::{num::NonZeroU64, sync::Arc};

use agglayer_aggregator_notifier::{CertifierClient, EpochPackerClient};
use agglayer_certificate_orchestrator::CertificateOrchestrator;
use agglayer_clock::{BlockClock, Clock, TimeClock};
use agglayer_config::{Config, Epoch};
use agglayer_contracts::{polygon_rollup_manager::PolygonRollupManager, L1RpcClient};
use agglayer_signer::ConfiguredSigner;
use agglayer_storage::{
    storage::DB,
    stores::{epochs::EpochsStore, pending::PendingStore, state::StateStore},
};
use anyhow::Result;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider, Ws},
    signers::Signer,
};
use tokio::{join, sync::mpsc, task::JoinHandle};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::{epoch_synchronizer::EpochSynchronizer, kernel::Kernel, rpc::AgglayerImpl};

pub(crate) struct Node {
    rpc_handle: JoinHandle<()>,
    certificate_orchestrator_handle: JoinHandle<()>,
}

#[buildstructor::buildstructor]
impl Node {
    /// Function that setups and starts the Agglayer node.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `config`: Sets the configuration.
    /// - `start`: Starts the Agglayer node.
    ///
    /// # Examples
    /// ```no_compile
    /// # use std::sync::Arc;
    /// # use agglayer_config::Config;
    /// # use agglayer_node::Node;
    /// # use tokio_util::sync::CancellationToken;
    /// # use anyhow::Result;
    /// #
    /// async fn start_node() -> Result<()> {
    ///    let config: Arc<Config> = Arc::new(Config::default());
    ///
    ///    Node::builder()
    ///      .config(config)
    ///      .cancellation_token(CancellationToken::new())
    ///      .start()
    ///      .await?;
    ///
    ///    Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The L1 node URL is invalid.
    /// - The configured signer is invalid.
    /// - The RPC server failed to start.
    /// - The [`TimeClock`] failed to start.
    #[builder(entry = "builder", exit = "start", visibility = "pub(crate)")]
    pub(crate) async fn start(
        config: Arc<Config>,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        // Initializing storage
        let pending_db = Arc::new(DB::open_cf(
            &config.storage.pending_db_path,
            agglayer_storage::storage::pending_db_cf_definitions(),
        )?);
        let state_db = Arc::new(DB::open_cf(
            &config.storage.state_db_path,
            agglayer_storage::storage::state_db_cf_definitions(),
        )?);

        let state_store = Arc::new(StateStore::new(state_db.clone()));
        let pending_store = Arc::new(PendingStore::new(pending_db.clone()));

        // Spawn the TimeClock.
        let clock_ref = match &config.epoch {
            Epoch::BlockClock(cfg) => {
                let provider = Provider::<Ws>::connect(config.l1.ws_node_url.as_str()).await?;
                let clock = BlockClock::new(provider, cfg.genesis_block, cfg.epoch_duration);

                clock.spawn(cancellation_token.clone()).await?
            }
            Epoch::TimeClock(cfg) => {
                let duration =
                    NonZeroU64::new(cfg.epoch_duration.as_secs()).ok_or(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "EpochDuration is invalid",
                    ))?;
                let clock = TimeClock::new_now(duration);

                clock.spawn(cancellation_token.clone()).await?
            }
        };

        let current_epoch = clock_ref.current_epoch();

        let epochs_store = Arc::new(EpochsStore::new(
            config.clone(),
            current_epoch,
            pending_store.clone(),
            state_store.clone(),
        )?);

        let current_epoch_store =
            EpochSynchronizer::start(state_store.clone(), epochs_store.clone(), clock_ref.clone())
                .await?;

        let certifier_client =
            CertifierClient::try_new(config.prover_entrypoint.clone(), pending_store.clone())
                .await?;
        let signer = ConfiguredSigner::new(config.clone()).await?;
        let address = signer.address();
        tracing::info!("Signer address: {:?}", address);

        // Create a new L1 RPC provider with the configured signer.
        let rpc = Arc::new(
            Provider::<Http>::try_from(config.l1.node_url.as_str())?
                .with_signer(signer)
                .nonce_manager(address),
        );

        let rollup_manager = L1RpcClient::new(PolygonRollupManager::new(
            config.l1.rollup_manager_contract,
            rpc.clone(),
        ));

        // Construct the core.
        let core = Kernel::new(rpc, config.clone());

        let epoch_packing_aggregator_task = EpochPackerClient::try_new(
            Arc::new(config.outbound.rpc.settle.clone()),
            state_store.clone(),
            Arc::new(rollup_manager),
        )?;

        let (data_sender, data_receiver) = mpsc::channel(
            config
                .certificate_orchestrator
                .input_backpressure_buffer_size,
        );

        let clock_subscription =
            tokio_stream::wrappers::BroadcastStream::new(clock_ref.subscribe()?)
                .filter_map(|value| value.ok());

        let certificate_orchestrator_handle = CertificateOrchestrator::builder()
            .clock(clock_subscription)
            .data_receiver(data_receiver)
            .cancellation_token(cancellation_token.clone())
            .epoch_packing_task_builder(epoch_packing_aggregator_task)
            .pending_store(pending_store.clone())
            .epochs_store(epochs_store.clone())
            .current_epoch(arc_swap::ArcSwap::new(Arc::new(current_epoch_store)))
            .state_store(state_store.clone())
            .certifier_task_builder(certifier_client)
            .start()
            .await?;

        // Bind the core to the RPC server.
        let server_handle = AgglayerImpl::new(
            core,
            data_sender,
            pending_store.clone(),
            state_store.clone(),
        )
        .start(config)
        .await?;

        let rpc_handle = tokio::spawn(async move {
            tokio::select! {
                _ = server_handle.stopped() => {},
                _ = cancellation_token.cancelled() => {
                    debug!("Node RPC shutdown requested.");
                }
            }
        });

        let node = Self {
            rpc_handle,
            certificate_orchestrator_handle,
        };

        Ok(node)
    }

    pub(crate) async fn await_shutdown(self) {
        debug!("Node shutdown started.");
        _ = join!(self.rpc_handle, self.certificate_orchestrator_handle);
        debug!("Node shutdown completed.");
    }
}
