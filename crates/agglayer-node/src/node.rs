use std::{num::NonZeroU64, sync::Arc};

use agglayer_aggregator_notifier::{CertifierClient, EpochPackerClient};
use agglayer_certificate_orchestrator::CertificateOrchestrator;
use agglayer_clock::{BlockClock, Clock, TimeClock};
use agglayer_config::{storage::backup::BackupConfig, Config, Epoch};
use agglayer_contracts::{
    polygon_rollup_manager::PolygonRollupManager,
    polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2, L1RpcClient,
};
use agglayer_jsonrpc_api::admin::AdminAgglayerImpl;
use agglayer_jsonrpc_api::service::AgglayerService;
use agglayer_jsonrpc_api::{kernel::Kernel, AgglayerImpl};
use agglayer_signer::ConfiguredSigner;
use agglayer_storage::{
    storage::{
        backup::{BackupClient, BackupEngine},
        DB,
    },
    stores::{
        debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore,
        PerEpochReader as _,
    },
};
use alloy::providers::WsConnect;
use anyhow::Result;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::Signer,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::epoch_synchronizer::EpochSynchronizer;

pub(crate) mod api;

pub(crate) struct Node {
    pub(crate) rpc_handle: JoinHandle<()>,
    pub(crate) certificate_orchestrator_handle: JoinHandle<()>,
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
        if config.mock_verifier {
            warn!(
                "Mock verifier is being used. This should only be used for testing purposes and \
                 not in production environments."
            );
        }

        // Initializing storage
        let pending_db = Arc::new(DB::open_cf(
            &config.storage.pending_db_path,
            agglayer_storage::storage::pending_db_cf_definitions(),
        )?);
        let state_db = Arc::new(DB::open_cf(
            &config.storage.state_db_path,
            agglayer_storage::storage::state_db_cf_definitions(),
        )?);

        // Initialize backup engine
        let backup_client = if let BackupConfig::Enabled {
            path,
            state_max_backup_count,
            pending_max_backup_count,
        } = &config.storage.backup
        {
            let (backup_engine, client) = BackupEngine::new(
                path,
                state_db.clone(),
                pending_db.clone(),
                *state_max_backup_count,
                *pending_max_backup_count,
                cancellation_token.clone(),
            )?;
            tokio::spawn(backup_engine.run());

            client
        } else {
            BackupClient::noop()
        };
        let state_store = Arc::new(StateStore::new(state_db.clone(), backup_client.clone()));
        let pending_store = Arc::new(PendingStore::new(pending_db.clone()));
        let debug_store = if config.debug_mode {
            Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path)?)
        } else {
            Arc::new(DebugStore::Disabled)
        };

        info!("Storage initialized.");

        // Spawn the TimeClock.
        let clock_ref = match &config.epoch {
            Epoch::BlockClock(cfg) => {
                info!(
                    "Starting BlockClock with provider: {}",
                    config.l1.ws_node_url
                );

                let clock = BlockClock::new_with_ws(
                    WsConnect::new(config.l1.ws_node_url.as_str()),
                    cfg.genesis_block,
                    cfg.epoch_duration,
                    config.l1.max_reconnection_elapsed_time,
                )
                .await
                .inspect_err(|e| {
                    error!("Failed to start BlockClock: {:?}", e);
                })?;

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
        info!("Clock started, current epoch {current_epoch}");

        let epochs_store = Arc::new(EpochsStore::new(
            config.clone(),
            current_epoch,
            pending_store.clone(),
            state_store.clone(),
            backup_client,
        )?);

        info!("Epoch synchronization started.");
        let current_epoch_store =
            EpochSynchronizer::start(state_store.clone(), epochs_store.clone(), clock_ref.clone())
                .await?;

        info!(
            "Epoch synchronization completed, active epoch: {}.",
            current_epoch_store.get_epoch_number()
        );

        let signer = ConfiguredSigner::new(config.clone()).await?;
        let address = signer.address();
        tracing::info!("Signer address: {:?}", address);

        // Create a new L1 RPC provider with the configured signer.
        let rpc = Arc::new(
            Provider::<Http>::try_from(config.l1.node_url.as_str())?
                .with_signer(signer)
                .nonce_manager(address),
        );

        tracing::debug!("RPC provider created");
        let rollup_manager = Arc::new(
            L1RpcClient::try_new(
                rpc.clone(),
                PolygonRollupManager::new(config.l1.rollup_manager_contract, rpc.clone()),
                PolygonZkEVMGlobalExitRootV2::new(
                    config.l1.polygon_zkevm_global_exit_root_v2_contract,
                    rpc.clone(),
                ),
            )
            .await?,
        );
        tracing::debug!("RollupManager created");

        let certifier_client = CertifierClient::try_new(
            config.prover_entrypoint.clone(),
            pending_store.clone(),
            Arc::clone(&rollup_manager),
            Arc::clone(&config),
        )
        .await?;
        info!("Certifier client created.");

        // Construct the core.
        let core = Kernel::new(rpc.clone(), config.clone());

        let current_epoch_store = Arc::new(arc_swap::ArcSwap::new(Arc::new(current_epoch_store)));
        let epoch_packing_aggregator_task = EpochPackerClient::try_new(
            Arc::new(config.outbound.rpc.settle.clone()),
            state_store.clone(),
            pending_store.clone(),
            Arc::clone(&rollup_manager),
            current_epoch_store.clone(),
        )?;

        info!("Epoch packing aggregator task created.");

        let (data_sender, data_receiver) = mpsc::channel(
            config
                .certificate_orchestrator
                .input_backpressure_buffer_size,
        );

        let certificate_orchestrator_handle = CertificateOrchestrator::builder()
            .clock(clock_ref)
            .data_receiver(data_receiver)
            .cancellation_token(cancellation_token.clone())
            .epoch_packing_task_builder(epoch_packing_aggregator_task)
            .pending_store(pending_store.clone())
            .epochs_store(epochs_store.clone())
            .current_epoch(current_epoch_store)
            .state_store(state_store.clone())
            .certifier_task_builder(certifier_client)
            .start()
            .await?;

        info!("Certificate orchestrator started.");

        // Set up the core service object.
        let service = Arc::new(AgglayerService::new(core));
        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            data_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            Arc::clone(&rollup_manager),
        ));

        let admin_router = AdminAgglayerImpl::new(
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
        )
        .start()
        .await?;

        // Bind the core to the RPC server.
        let json_rpc_router = AgglayerImpl::new(service, rpc_service).start().await?;

        let grpc_router = agglayer_grpc_api::Server::with_config(config.clone())
            .build()
            .map_err(|err| {
                error!("Failed to build gRPC router: {}", err);
                err
            })?;

        let health_router = api::rest::health_router();

        let router = axum::Router::new()
            .merge(health_router)
            .merge(json_rpc_router)
            .nest("/grpc", grpc_router);

        let listener = tokio::net::TcpListener::bind(config.rpc_addr()).await?;
        let admin_listener = tokio::net::TcpListener::bind(config.admin_rpc_addr()).await?;
        info!(on = %config.rpc_addr(), "API listening");

        let api_server = axum::serve(listener, router)
            .with_graceful_shutdown(cancellation_token.clone().cancelled_owned());

        let admin_server = axum::serve(admin_listener, admin_router)
            .with_graceful_shutdown(cancellation_token.clone().cancelled_owned());

        let rpc_handle = tokio::spawn(async move {
            tokio::select! {
                _ = api_server => {},
                _ = admin_server => {},
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
        tokio::select! {
            _ = self.rpc_handle => {
            }
            _ = self.certificate_orchestrator_handle => {
            }
        }
        debug!("Node shutdown completed.");
    }
}
