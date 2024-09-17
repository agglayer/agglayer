use std::{num::NonZeroU64, sync::Arc};

use agglayer_clock::{Clock, TimeClock};
use agglayer_config::{Config, Epoch};
use agglayer_signer::ConfiguredSigner;
use anyhow::Result;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::Signer,
};
use tokio::{join, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::{kernel::Kernel, rpc::AgglayerImpl};

pub(crate) struct Node {
    rpc_handle: JoinHandle<()>,
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
        let signer = ConfiguredSigner::new(config.clone()).await?;
        let address = signer.address();
        // Create a new L1 RPC provider with the configured signer.
        let client = reqwest::Client::builder()
            .timeout(config.l1.rpc_timeout)
            .build()?;
        let rpc = Provider::new(Http::new_with_client(config.l1.node_url.clone(), client))
            .with_signer(signer)
            .nonce_manager(address);

        // Construct the core.
        let core = Kernel::new(rpc, config.clone());

        // Spawn the TimeClock.
        let _clock_ref = match &config.epoch {
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

        // Bind the core to the RPC server.
        let server_handle = AgglayerImpl::new(core).start(config).await?;

        let rpc_handle = tokio::spawn(async move {
            tokio::select! {
                _ = server_handle.stopped() => {},
                _ = cancellation_token.cancelled() => {
                    debug!("Node shutdown requested.");
                }
            }
        });

        let node = Self { rpc_handle };

        Ok(node)
    }

    pub(crate) async fn await_shutdown(self) {
        _ = join!(self.rpc_handle);
        debug!("Node shutdown complete.");
    }
}
