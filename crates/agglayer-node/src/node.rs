use std::{num::NonZeroU64, sync::Arc};

use agglayer_aggregator_notifier::{CertifierClient, RpcSettlementClient};
use agglayer_certificate_orchestrator::CertificateOrchestrator;
use agglayer_clock::{BlockClock, Clock, TimeClock};
use agglayer_config::{storage::backup::BackupConfig, Config, Epoch};
use agglayer_contracts::{contracts::PolygonRollupManager, L1RpcClient};
use agglayer_jsonrpc_api::{
    admin::AdminAgglayerImpl, kernel::Kernel, service::AgglayerService, AgglayerImpl,
};
use agglayer_signer::{ConfiguredSigner, ConfiguredSigners};
use agglayer_storage::{
    backup::{BackupClient, BackupEngine},
    stores::{
        debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore,
        PerEpochReader as _,
    },
};
use alloy::{
    network::EthereumWallet,
    providers::{ProviderBuilder, WalletProvider, WsConnect},
};
use eyre::Context as _;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tower::buffer::Buffer;
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
    /// #
    /// async fn start_node() -> eyre::Result<()> {
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
    ) -> eyre::Result<Self> {
        if config.mock_verifier {
            warn!(
                "Mock verifier is being used. This should only be used for testing purposes and \
                 not in production environments."
            );
        }

        // Initializing storage
        let pending_db = Arc::new(PendingStore::init_db(&config.storage.pending_db_path)?);
        let state_db = Arc::new(StateStore::init_db(&config.storage.state_db_path)?);

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
                    config.l1.connect_attempt_timeout,
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
                .await
                .context("Failed starting epoch synchronizer")?;

        info!(
            "Epoch synchronization completed, active epoch: {}.",
            current_epoch_store.get_epoch_number()
        );

        // Create RPC clients, note that they can be the same signer
        // or not depending on the configuration. If is the same signer
        // we share the nonce management too.

        let (rpc_pp_settlement, rpc_tx_settlement) = {
            // We will use the same parameterization to create both providers.
            let fn_build_provider = |signer: ConfiguredSigner| {
                Arc::new(
                    ProviderBuilder::new()
                        .with_simple_nonce_management()
                        .wallet(EthereumWallet::from(signer))
                        .connect_client(
                            alloy::rpc::client::RpcClient::builder()
                                .layer(crate::L1TraceLayer)
                                .http(config.l1.node_url.clone()),
                        ),
                )
            };

            let signers = ConfiguredSigners::new(&config).await?;
            let provider_cert = fn_build_provider(signers.pp_settlement);

            let provider_tx = if let Some(tx_settlement) = signers.tx_settlement {
                fn_build_provider(tx_settlement)
            } else {
                warn!("Using the same provider for certificate and tx settlement");
                provider_cert.clone()
            };

            tracing::info!(
                "Cert signer address: {:?}",
                // Note that because the signer always has at least one address,
                // this iterator will always have at least one element.
                provider_cert.signer_addresses().next().unwrap()
            );
            tracing::info!(
                "Tx signer address: {:?}",
                provider_tx.signer_addresses().next().unwrap()
            );

            (provider_cert, provider_tx)
        };

        tracing::debug!("RPC provider created");
        let rollup_manager = Arc::new(
            L1RpcClient::try_new(
                rpc_pp_settlement.clone(),
                PolygonRollupManager::new(
                    config.l1.rollup_manager_contract.into(),
                    (*rpc_pp_settlement).clone(),
                ),
                config.l1.polygon_zkevm_global_exit_root_v2_contract.into(),
                config.outbound.rpc.settle_cert.gas_multiplier_factor,
                {
                    let gas_config = &config.outbound.rpc.settle_cert.gas_price;
                    agglayer_contracts::GasPriceParams::new(
                        gas_config.multiplier.as_u64_per_1000(),
                        gas_config.floor..=gas_config.ceiling,
                    )?
                },
                config.l1.event_filter_block_range.get(),
            )
            .await?,
        );
        tracing::debug!("RollupManager created");

        let (_vkey, prover_executor) =
            prover_executor::Executor::create_prover(config.prover.clone(), pessimistic_proof::ELF)
                .await?;

        let prover_buffer = Buffer::new(prover_executor, config.prover_buffer_size);

        let certifier_client = CertifierClient::try_new(
            pending_store.clone(),
            Arc::clone(&rollup_manager),
            Arc::clone(&config),
            prover_buffer,
        )
        .await?;
        info!("Certifier client created.");

        // Construct the core.
        let core = Kernel::new(rpc_tx_settlement.clone(), config.clone()).unwrap();

        let current_epoch_store = Arc::new(arc_swap::ArcSwap::new(Arc::new(current_epoch_store)));
        let epoch_packing_aggregator_task = RpcSettlementClient::new(
            Arc::new(config.outbound.rpc.settle_cert.clone()),
            state_store.clone(),
            pending_store.clone(),
            Arc::clone(&rollup_manager),
            current_epoch_store.clone(),
        );

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
            .settlement_client(epoch_packing_aggregator_task)
            .pending_store(pending_store.clone())
            .epochs_store(epochs_store.clone())
            .current_epoch(current_epoch_store)
            .state_store(state_store.clone())
            .certifier_task_builder(certifier_client)
            .l1_rpc(Arc::clone(&rollup_manager))
            .start()
            .await
            .context("Failed starting certificate orchestrator")?;

        info!("Certificate orchestrator started.");

        // Set up the core service object.
        let service = Arc::new(AgglayerService::new(core));
        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            data_sender.clone(),
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            epochs_store.clone(),
            config.clone(),
            Arc::clone(&rollup_manager),
        ));

        let admin_router = AdminAgglayerImpl::new(
            data_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
        )
        .start()
        .await
        .context("Failed starting admin router")?;

        // Bind the core to the RPC server.
        let json_rpc_router = AgglayerImpl::new(service, rpc_service.clone())
            .start()
            .await
            .context("Failed starting JSON-RPC router")?;

        let public_grpc_router =
            agglayer_grpc_api::Server::with_config(config.clone(), rpc_service)
                .build()
                .inspect_err(|err| error!(?err, "Failed to build public gRPC router"))?;

        let health_router = api::rest::health_router();

        let readrpc_router = axum::Router::new()
            .merge(health_router)
            .merge(json_rpc_router);

        let readrpc_listener = tokio::net::TcpListener::bind(config.readrpc_addr()).await?;
        let public_grpc_listener = tokio::net::TcpListener::bind(config.public_grpc_addr()).await?;
        let admin_listener = tokio::net::TcpListener::bind(config.admin_rpc_addr()).await?;
        info!(on = %config.readrpc_addr(), "ReadRPC listening");
        info!(on = %config.public_grpc_addr(), "Public gRPC listening");
        info!(on = %config.admin_rpc_addr(), "AdminRPC listening");

        let readrpc_server = axum::serve(readrpc_listener, readrpc_router)
            .with_graceful_shutdown(cancellation_token.clone().cancelled_owned());

        let public_grpc_server = axum::serve(public_grpc_listener, public_grpc_router)
            .with_graceful_shutdown(cancellation_token.clone().cancelled_owned());

        let admin_server = axum::serve(admin_listener, admin_router)
            .with_graceful_shutdown(cancellation_token.clone().cancelled_owned());

        let rpc_handle = tokio::spawn(async move {
            tokio::select! {
                _ = readrpc_server => {},
                _ = public_grpc_server => {},
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
