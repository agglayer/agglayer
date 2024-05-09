use std::{num::NonZeroU64, sync::Arc};

use agglayer_clock::{Clock, TimeClock};
use agglayer_config::{Config, Epoch};
use anyhow::Result;
use ethers::{
    middleware::MiddlewareBuilder as _,
    providers::{Http, Provider},
};

use crate::{kernel::Kernel, rpc::AgglayerImpl};

pub(crate) struct Node {}

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
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use agglayer_config::Config;
    /// # use agglayer_node::Node;
    /// # use anyhow::Result;
    /// #
    /// async fn start_node() -> Result<()> {
    ///    let config: Arc<Config> = Arc::new(Config::default());
    ///
    ///    Node::builder()
    ///      .config(config)
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
    /// - The time clock failed to start.
    #[builder(entry = "builder", exit = "start", visibility = "pub(crate)")]
    pub(crate) async fn start(config: Arc<Config>) -> Result<()> {
        // Create a new L1 RPC provider with the configured signer.
        let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?
            .with_signer(config.get_configured_signer().await?);

        // Construct the core.
        let core = Kernel::new(rpc, config.clone());

        // Spawn the time clock.
        let (_clock_ref, _epoch_ref) = match &config.epoch {
            Epoch::TimeClock(cfg) => {
                let duration =
                    NonZeroU64::new(cfg.epoch_duration.as_secs()).ok_or(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "EpochDuration is invalid",
                    ))?;
                let clock = TimeClock::new_now(duration);

                let epoch_ref = clock.epoch_ref();

                (clock.spawn().await?, epoch_ref)
            }
        };

        // Bind the core to the RPC server.
        let server_handle = AgglayerImpl::new(core).start(config).await?;

        server_handle.stopped().await;

        Ok(())
    }
}
