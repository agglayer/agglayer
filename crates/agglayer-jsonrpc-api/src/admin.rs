use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter, UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
};
use agglayer_tries::smt::SmtPath;
use agglayer_types::{
    Address, Certificate, CertificateHeader, CertificateId, CertificateStatus,
    CertificateStatusError, Digest, Height, NetworkId, SettlementTxHash, U256,
};
use jsonrpsee::{core::async_trait, proc_macros::rpc, server::ServerBuilder};
use pessimistic_proof::local_balance_tree::BalanceTree;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::{error, info, instrument, warn};
use unified_bridge::TokenInfo;

use super::error::RpcResult;
use crate::{error::Error, rpc_middleware, JsonRpcService};

#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
pub enum ProcessNow {
    #[serde(rename = "process-now=true")]
    True,

    #[serde(rename = "process-now=false")]
    False,
}

use serde_with::{serde_as, DisplayFromStr};

/// Token balance entry structure in order to display the balance tree values.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceEntry {
    pub origin_network: NetworkId,
    pub origin_token_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: U256,
}

impl From<(SmtPath<192>, Digest)> for TokenBalanceEntry {
    fn from((path, leaf_value): (SmtPath<192>, Digest)) -> Self {
        let TokenInfo {
            origin_network,
            origin_token_address,
        } = TokenInfo::from_bits(&path.as_bits());

        Self {
            origin_network,
            origin_token_address,
            amount: U256::from_be_bytes(*leaf_value.as_bytes()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenBalanceResponse {
    pub balances: Vec<TokenBalanceEntry>,
}

#[rpc(server, namespace = "admin")]
pub(crate) trait AdminAgglayer {
    #[method(name = "getTokenBalance")]
    async fn get_token_balance(
        &self,
        network_id: NetworkId,
        token_info: Option<TokenInfo>,
    ) -> RpcResult<GetTokenBalanceResponse>;

    #[method(name = "getCertificate")]
    async fn get_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<(Certificate, Option<CertificateHeader>)>;

    #[method(name = "forcePushPendingCertificate")]
    async fn force_push_pending_certificate(
        &self,
        certificate: Certificate,
        status: CertificateStatus,
    ) -> RpcResult<()>;

    #[method(name = "forceEditCertificate")]
    async fn force_edit_certificate(
        &self,
        certificate_id: CertificateId,
        process_now: ProcessNow,
        operation_1: Option<String>,
        operation_2: Option<String>,
        // Add one more operation for each allowed operation, so we can do all the needed changes
        // "atomically". For now, we have:
        // * set status
        // * (un)set settlement tx hash.
    ) -> RpcResult<()>;

    #[method(name = "setLatestPendingCertificate")]
    async fn set_latest_pending_certificate(&self, certificate_id: CertificateId) -> RpcResult<()>;

    #[method(name = "setLatestProvenCertificate")]
    async fn set_latest_proven_certificate(&self, certificate_id: CertificateId) -> RpcResult<()>;

    #[method(name = "removePendingCertificate")]
    async fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        remove_proof: bool,
    ) -> RpcResult<()>;

    #[method(name = "removePendingProof")]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()>;

    #[method(name = "getDisabledNetworks")]
    async fn get_disabled_networks(&self) -> RpcResult<Vec<NetworkId>>;
    #[method(name = "disableNetwork")]
    async fn disable_network(&self, network_id: NetworkId) -> RpcResult<()>;
    #[method(name = "enableNetwork")]
    async fn enable_network(&self, network_id: NetworkId) -> RpcResult<()>;
}

/// The Admin RPC agglayer service implementation.
pub struct AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
}

impl<PendingStore, StateStore, DebugStore> AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    /// Create an instance of the admin RPC agglayer service.
    pub fn new(
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            certificate_sender,
            pending_store,
            state,
            debug_store,
            config,
        }
    }
}

impl<PendingStore, StateStore, DebugStore> AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    pub async fn start(self) -> eyre::Result<axum::Router> {
        // Create the RPC service
        let config = self.config.clone();

        // Create the RPC server.
        let mut server_builder = ServerBuilder::new()
            // Set the maximum request body size. The default is 10MB.
            .max_request_body_size(config.rpc.max_request_body_size)
            // Set the maximum response body size. The default is 10MB.
            .max_response_body_size(config.rpc.max_response_body_size)
            // Set the maximum number of connections. The default is 100.
            .max_connections(config.rpc.max_connections)
            // Set the batch request limit. The default is unlimited.
            .set_batch_request_config(match config.rpc.batch_request_limit {
                None => jsonrpsee::server::BatchRequestConfig::Unlimited,
                Some(0) => jsonrpsee::server::BatchRequestConfig::Disabled,
                Some(n) => jsonrpsee::server::BatchRequestConfig::Limit(n),
            });

        // Enable WebSocket ping/pong with the configured interval.
        // By default, pings are disabled.
        if let Some(duration) = config.rpc.ping_interval {
            server_builder = server_builder
                .enable_ws_ping(jsonrpsee::server::PingConfig::default().ping_interval(duration));
        }

        // Create a CORS middleware to allow cross-origin requests.
        let cors = CorsLayer::new()
            .allow_methods([
                hyper::Method::POST,
                hyper::Method::GET,
                hyper::Method::OPTIONS,
            ])
            .allow_origin(tower_http::cors::Any)
            .allow_headers([hyper::header::CONTENT_TYPE]);

        // Create a middleware stack with the CORS middleware and a proxy layer for
        // health checks.
        let middleware = tower::ServiceBuilder::new()
            .layer(CompressionLayer::new())
            .layer(cors);

        let service_builder =
            server_builder.set_rpc_middleware(rpc_middleware::from_config(&config));
        let (stop_handle, server_handle) = jsonrpsee::server::stop_channel();
        std::mem::forget(server_handle);

        let service = self.into_rpc();
        let service = JsonRpcService {
            service: service_builder
                .to_service_builder()
                .build(service, stop_handle),
        };

        Ok(axum::Router::new()
            .route("/", axum::routing::get_service(service.clone()))
            .route("/", axum::routing::post_service(service.clone()))
            .layer(middleware))
    }
}

impl<PendingStore, StateStore, DebugStore> Drop
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer service");
    }
}

#[async_trait]
impl<PendingStore, StateStore, DebugStore> AdminAgglayerServer
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    #[instrument(skip(self), level = "debug")]
    async fn get_token_balance(
        &self,
        network_id: NetworkId,
        token_info: Option<TokenInfo>,
    ) -> RpcResult<GetTokenBalanceResponse> {
        let Some(balance_tree) = self
            .state
            .read_local_network_state(network_id)
            .map_err(|error| {
                error!(?error, "Failed to read the balance tree");
                Error::internal("Unable to read the balance tree")
            })?
            .map(|s| BalanceTree(s.balance_tree))
        else {
            return Ok(GetTokenBalanceResponse { balances: vec![] }); // empty balances, not an error
        };

        // get the balance of a given token, or return all of them
        let balances: Vec<TokenBalanceEntry> = if let Some(token_info) = token_info {
            let amount = balance_tree.get_balance(token_info);
            vec![TokenBalanceEntry {
                origin_network: token_info.origin_network,
                origin_token_address: token_info.origin_token_address,
                amount,
            }]
        } else {
            balance_tree
                .get_all_balances()
                .map_err(|error| {
                    error!(?error, "Failed to get all balances");
                    Error::internal("Unable to get all balances")
                })?
                .map(TokenBalanceEntry::from)
                .collect()
        };

        Ok(GetTokenBalanceResponse { balances })
    }

    #[instrument(skip(self), fields(hash), level = "debug")]
    async fn get_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<(Certificate, Option<CertificateHeader>)> {
        match self.debug_store.get_certificate(&certificate_id) {
            Ok(Some(cert)) => match self
                .state
                .get_certificate_header(&certificate_id)
                .map(|header| (cert, header))
            {
                Ok(result) => Ok(result),
                Err(error) => {
                    error!("Failed to get certificate header: {}", error);
                    Err(Error::internal("Unable to get certificate header"))
                }
            },
            Ok(None) => Err(Error::ResourceNotFound(format!(
                "Certificate({certificate_id})"
            ))),
            Err(error) => {
                error!("Failed to get certificate: {}", error);

                Err(Error::internal("Unable to get certificate"))
            }
        }
    }

    #[instrument(skip(self, certificate), level = "debug")]
    async fn force_push_pending_certificate(
        &self,
        certificate: Certificate,
        status: CertificateStatus,
    ) -> RpcResult<()> {
        warn!(
            hash = certificate.hash().to_string(),
            ?certificate,
            "(ADMIN) Forcing push of pending certificate: {}",
            certificate.hash()
        );
        let header = self
            .state
            .get_certificate_header(&certificate.hash())
            .map_err(|error| {
                error!(?error, "Failed to get certificate header");
                Error::internal("Unable to get certificate header")
            })?;
        if let Some(header) = header {
            if header.status == CertificateStatus::Settled {
                return Err(Error::InvalidArgument(
                    "Cannot change status of a settled certificate".to_string(),
                ));
            }
        }
        match self.pending_store.insert_pending_certificate(
            certificate.network_id,
            certificate.height,
            &certificate,
        ) {
            Ok(_) => match self
                .state
                .update_certificate_header_status(&certificate.hash(), &status)
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    error!("Failed to insert certificate header: {}", error);
                    Err(Error::internal("Unable to insert certificate header"))
                }
            },
            Err(error) => {
                error!("Failed to insert pending certificate: {}", error);
                Err(Error::internal("Unable to insert pending certificate"))
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn force_edit_certificate(
        &self,
        certificate_id: CertificateId,
        process_now: ProcessNow,
        operation_1: Option<String>,
        operation_2: Option<String>,
    ) -> RpcResult<()> {
        warn!(
            %certificate_id,
            ?process_now,
            ?operation_1,
            ?operation_2,
            "(ADMIN) Editing certificate"
        );

        enum Operation {
            SetStatus {
                from: CertificateStatus,
                to: CertificateStatus,
            },
            SetSettlementTxHash {
                from: Option<SettlementTxHash>,
                to: Option<SettlementTxHash>,
            },
        }

        impl Operation {
            fn parse(operation: &str) -> Result<Self, Error> {
                if let Some(operation) = operation.strip_prefix("set-status,from=") {
                    let parts = operation.split(",to=").collect::<Vec<_>>();
                    let [from_status, to_status] = parts[..] else {
                        return Err(Error::InvalidArgument(
                            "Invalid set status operation format".to_string(),
                        ));
                    };
                    fn parse_status(status_str: &str) -> Result<CertificateStatus, Error> {
                        if status_str == "InError" {
                            Ok(CertificateStatus::error(
                                CertificateStatusError::InternalError(
                                    "Set to InError by administrator".to_string(),
                                ),
                            ))
                        } else {
                            CertificateStatus::deserialize(
                                serde::de::value::BorrowedStrDeserializer::new(status_str),
                            )
                            .map_err(|e: serde::de::value::Error| {
                                Error::InvalidArgument(format!(
                                    "Invalid status {status_str}: {e:?}"
                                ))
                            })
                        }
                    }
                    Ok(Operation::SetStatus {
                        from: parse_status(from_status)?,
                        to: parse_status(to_status)?,
                    })
                } else if let Some(operation) =
                    operation.strip_prefix("set-settlement-tx-hash,from=")
                {
                    let parts = operation.split(",to=").collect::<Vec<_>>();
                    let [from_tx_hash, to_tx_hash] = parts[..] else {
                        return Err(Error::InvalidArgument(
                            "Invalid set settlement tx hash operation format".to_string(),
                        ));
                    };
                    fn parse_tx_hash(tx_hash_str: &str) -> Result<Option<SettlementTxHash>, Error> {
                        if tx_hash_str == "null" {
                            Ok(None)
                        } else {
                            SettlementTxHash::deserialize(
                                serde::de::value::BorrowedStrDeserializer::new(tx_hash_str),
                            )
                            .map(Some)
                            .map_err(|e: serde::de::value::Error| {
                                Error::InvalidArgument(format!(
                                    "Invalid settlement tx hash {tx_hash_str}: {e:?}"
                                ))
                            })
                        }
                    }
                    Ok(Operation::SetSettlementTxHash {
                        from: parse_tx_hash(from_tx_hash)?,
                        to: parse_tx_hash(to_tx_hash)?,
                    })
                } else {
                    Err(Error::InvalidArgument(format!(
                        "Unknown operation: {operation:?}"
                    )))
                }
            }
        }

        let operations = [operation_1, operation_2]
            .into_iter()
            .flatten()
            .map(|op_str| Operation::parse(&op_str))
            .collect::<Result<Vec<_>, _>>()?;

        let header = self
            .state
            .get_certificate_header(&certificate_id)
            .map_err(|error| {
                error!(?error, "Failed to get certificate header");
                Error::internal("Unable to get certificate header")
            })?
            .ok_or_else(|| {
                error!("Certificate header not found");
                Error::ResourceNotFound(format!("CertificateHeader({certificate_id})"))
            })?;

        if header.status == CertificateStatus::Settled {
            return Err(Error::InvalidArgument(
                "Cannot edit a settled certificate".to_string(),
            ));
        }

        // Check that the current values match the "from" value
        for operation in operations.iter() {
            match operation {
                Operation::SetStatus { from, to: _ } => {
                    // Ensure that the original status is the one described in `from=`.
                    // However, for InError status, the `from=` does not contain the error message.
                    // So, we match it separately, and we do not verify the current error message if
                    // we had `set-status,from=InError,to=*`
                    if &header.status != from
                        && !matches!(
                            (&header.status, &from),
                            (
                                &CertificateStatus::InError { .. },
                                &CertificateStatus::InError { .. }
                            )
                        )
                    {
                        return Err(Error::InvalidArgument(format!(
                            "Current status ({:?}) does not match expected 'from' status ({:?})",
                            header.status, from
                        )));
                    }
                }
                Operation::SetSettlementTxHash { from, to: _ } => {
                    if &header.settlement_tx_hash != from {
                        return Err(Error::InvalidArgument(format!(
                            "Current settlement_tx_hash ({:?}) does not match expected 'from' \
                             settlement_tx_hash ({:?})",
                            header.settlement_tx_hash, from
                        )));
                    }
                }
            }
        }

        // Now, actually apply the operations
        for operation in operations {
            match operation {
                Operation::SetStatus { from: _, to } => {
                    self.state
                        .update_certificate_header_status(&certificate_id, &to)
                        .map_err(|error| {
                            error!(?error, ?to, "Failed to update certificate status");
                            Error::internal("Unable to update certificate status")
                        })?;
                }
                Operation::SetSettlementTxHash {
                    from: _,
                    to: Some(to),
                } => {
                    self.state
                        .update_settlement_tx_hash(
                            &certificate_id,
                            to,
                            UpdateEvenIfAlreadyPresent::Yes,
                            UpdateStatusToCandidate::No,
                        )
                        .map_err(|error| {
                            error!(?error, ?to, "Failed to update settlement_tx_hash");
                            Error::internal("Unable to update settlement_tx_hash")
                        })?;
                }
                Operation::SetSettlementTxHash { from: _, to: None } => {
                    self.state
                        .remove_settlement_tx_hash(&certificate_id)
                        .map_err(|error| {
                            error!(?error, "Failed to remove settlement_tx_hash");
                            Error::internal("Unable to remove settlement_tx_hash")
                        })?;
                }
            }
        }

        // Finally, if requested, reprocess the certificate
        if process_now == ProcessNow::True {
            self.certificate_sender
                .send((header.network_id, header.height, certificate_id))
                .await
                .map_err(|error| {
                    error!(?error, "Failed to send certificate to orchestrator");
                    Error::internal("Unable to send certificate to orchestrator")
                })?;
        }

        Ok(())
    }

    #[instrument(skip(self, certificate_id), level = "debug")]
    async fn set_latest_pending_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Setting latest pending certificate: {}", certificate_id
        );
        let certificate = if let Some(certificate) = self
            .state
            .get_certificate_header(&certificate_id)
            .map_err(|error| {
                error!("Failed to get certificate header: {}", error);
                Error::internal("Unable to get certificate header")
            })? {
            certificate
        } else {
            return Err(Error::ResourceNotFound(format!(
                "CertificateHeader({certificate_id})"
            )));
        };

        match self
            .pending_store
            .set_latest_pending_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.certificate_id,
            ) {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("Failed to update latest pending certificate: {}", error);
                Err(Error::internal(
                    "Unable to update latest pending certificate",
                ))
            }
        }
    }

    #[instrument(skip(self, certificate_id), level = "debug")]
    async fn set_latest_proven_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Setting latest proven certificate: {}", certificate_id
        );
        let certificate = if let Some(certificate) = self
            .state
            .get_certificate_header(&certificate_id)
            .map_err(|error| {
                error!("Failed to get certificate header: {}", error);
                Error::internal("Unable to get certificate header")
            })? {
            certificate
        } else {
            return Err(Error::ResourceNotFound(format!(
                "CertificateHeader({certificate_id})"
            )));
        };

        match self
            .pending_store
            .set_latest_proven_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.certificate_id,
            ) {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("Failed to update latest proven certificate: {}", error);
                Err(Error::internal(
                    "Unable to update latest proven certificate",
                ))
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Removing pending proof: {}", certificate_id
        );

        self.pending_store
            .remove_generated_proof(&certificate_id)
            .map_err(|error| {
                error!("Failed to remove generated proof: {}", error);
                Error::internal("Unable to remove generated proof")
            })
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        remove_proof: bool,
    ) -> RpcResult<()> {
        warn!(
            "(ADMIN) Removing pending certificate for network {} at height {}",
            network_id, height
        );
        let certificate_id = if let Some(certificate) = self
            .pending_store
            .get_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to get pending certificate: {}", error);
                Error::internal("Unable to get pending certificate")
            })? {
            certificate.hash()
        } else {
            return Err(Error::ResourceNotFound(format!(
                "PendingCertificate({network_id:?}, {height:?})",
            )));
        };

        self.pending_store
            .remove_pending_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to remove pending certificate: {error}");
                Error::internal("Unable to remove pending certificate")
            })?;

        // Update certificate status to InError in the state store
        let error_status = CertificateStatus::error(CertificateStatusError::InternalError(
            "Certificate removed from pending store by administrator".to_string(),
        ));
        self.state
            .update_certificate_header_status(&certificate_id, &error_status)
            .map_err(|error| {
                error!(
                    %certificate_id,
                    ?error,
                    "Failed to update certificate status in the state store on pending removal"
                );
                Error::internal(format!(
                    "Unable to update certificate_id: {certificate_id} status in the state store \
                     on pending removal"
                ))
            })?;

        if remove_proof {
            self.pending_store
                .remove_generated_proof(&certificate_id)
                .map_err(|error| {
                    error!( %certificate_id, ?error, "Failed to remove generated proof");
                    Error::internal(format!(
                        "Failed to remove generated proof for certificate_id: {certificate_id}"
                    ))
                })?;
        }

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_disabled_networks(&self) -> RpcResult<Vec<NetworkId>> {
        self.state.get_disabled_networks().map_err(|error| {
            error!(?error, "Failed to get disabled networks");
            Error::internal("Unable to get disabled networks")
        })
    }

    #[instrument(skip(self), level = "debug")]
    async fn disable_network(&self, network_id: NetworkId) -> RpcResult<()> {
        self.state
            .disable_network(&network_id, agglayer_types::network_info::DisabledBy::Admin)
            .map_err(|error| {
                error!(?error, "Failed to disable network {network_id}");
                Error::internal(format!("Unable to disable network {network_id}"))
            })
    }

    #[instrument(skip(self), level = "debug")]
    async fn enable_network(&self, network_id: NetworkId) -> RpcResult<()> {
        self.state.enable_network(&network_id).map_err(|error| {
            error!(?error, "Failed to enable network {network_id}");
            Error::internal(format!("Unable to enable network {network_id}"))
        })
    }
}
