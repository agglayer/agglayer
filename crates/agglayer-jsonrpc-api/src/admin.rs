use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use agglayer_config::Config;
use agglayer_settlement_service::{LiveTaskNotification, NewSettlementAttempt, SettlementService};
use agglayer_storage::stores::{
    DebugReader, DebugWriter, EditEvenIfCompleted, PendingCertificateReader,
    PendingCertificateWriter, SettlementReader, SettlementWriter, StateReader, StateWriter,
    UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
};
use agglayer_tries::smt::SmtPath;
use agglayer_types::{
    Address, Certificate, CertificateHeader, CertificateId, CertificateStatus,
    CertificateStatusError, Digest, Height, NetworkId, Nonce, SettlementJobId, SettlementTxHash,
    U256,
};
use alloy::providers::{Provider, WalletProvider};
use jsonrpsee::{core::async_trait, proc_macros::rpc, server::ServerBuilder};
use pessimistic_proof::local_balance_tree::BalanceTree;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::{error, info, instrument, warn};
use unified_bridge::TokenInfo;

use super::error::RpcResult;
use crate::{error::Error, rpc_middleware, JsonRpcService};

/// Controls whether the certificate is immediately submitted for reprocessing
/// after edits are applied.
///
/// Passed as the `process_now` parameter of `admin_forceEditCertificate`.
#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
pub enum ProcessNow {
    /// Reprocess the certificate immediately after edits.
    ///
    /// Sends the certificate to the orchestrator as if it had just been
    /// submitted. Use this to recover a certificate that is stuck in an
    /// error state after correcting its status or settlement tx hash.
    #[serde(rename = "process-now=true")]
    True,

    /// Apply edits without triggering reprocessing.
    #[serde(rename = "process-now=false")]
    False,
}

/// A settlement attempt as accepted by `admin_insertSettlementAttempt`.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertAttemptParams {
    /// Hash of the settlement transaction. The only mandatory field: the
    /// others are resolved from the transaction when omitted.
    pub tx_hash: SettlementTxHash,

    /// Wallet the settlement transaction was sent from. Resolved by fetching
    /// the transaction from L1 when omitted.
    #[serde(default)]
    pub sender_wallet: Option<Address>,

    /// L1 nonce of the settlement transaction. Resolved by fetching the
    /// transaction from L1 when omitted.
    #[serde(default)]
    pub nonce: Option<u64>,

    /// Unix timestamp (in seconds) at which the transaction was submitted to
    /// L1. Defaults to now. The settlement task uses it as the base of its
    /// retry backoff for this attempt.
    #[serde(default)]
    pub submission_time_unix_secs: Option<u64>,

    /// `max_fee_per_gas` (wei) of the transaction. A fee-bumping retry
    /// outbids this baseline. When omitted, taken from the L1 transaction if
    /// it was fetched, else 0 (which makes a retry start over from freshly
    /// estimated fees).
    #[serde(default)]
    pub max_fee_per_gas: Option<u128>,

    /// `max_priority_fee_per_gas` (wei) of the transaction. Defaulted like
    /// `maxFeePerGas`.
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<u128>,
}

impl From<InsertAttemptParams> for NewSettlementAttempt {
    fn from(params: InsertAttemptParams) -> Self {
        Self {
            tx_hash: params.tx_hash,
            sender_wallet: params.sender_wallet,
            nonce: params.nonce.map(Nonce),
            submission_time: params
                .submission_time_unix_secs
                .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
            max_fee_per_gas: params.max_fee_per_gas,
            max_priority_fee_per_gas: params.max_priority_fee_per_gas,
        }
    }
}

/// Controls whether a settlement attempt mutation may touch a job that
/// already has a terminal result.
///
/// Passed as the optional trailing `force` parameter of the attempt
/// mutations; omitting it is equivalent to `"force=false"`.
///
/// Editing a completed job's attempts is refused by default. Forcing exists
/// to prepare `admin_forceRemoveSettlementJobResult`: attempt-result
/// corrections must land while the job still has its terminal result,
/// because removing the result immediately respawns the task, which could
/// re-derive and re-record the job result from the uncorrected attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum Force {
    /// Apply the mutation even if the job already has a terminal result.
    #[serde(rename = "force=true")]
    True,

    /// Refuse the mutation on a job that already has a terminal result.
    #[serde(rename = "force=false")]
    False,
}

fn edit_even_if_completed(force: Option<Force>) -> EditEvenIfCompleted {
    match force {
        Some(Force::True) => EditEvenIfCompleted::Yes,
        Some(Force::False) | None => EditEvenIfCompleted::No,
    }
}

/// Outcome of a settlement admin mutation.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResponse {
    /// The attempt number the mutation landed on. For
    /// `admin_insertSettlementAttempt` this is the newly assigned number.
    pub attempt_number: u64,

    /// Whether the live task observed the edit. Anything but `notified`
    /// means the task acts on stale state until it reloads
    /// (`admin_reloadSettlementTask` is the manual escape hatch).
    pub live_task: LiveTaskNotification,
}

fn settlement_internal_error(error: eyre::Report) -> Error {
    error!(?error, "Settlement admin command failed");
    Error::internal(format!("{error:#}"))
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

    /// Edit a certificate's mutable fields as an administrative override.
    ///
    /// **JSON-RPC method:** `admin_forceEditCertificate`
    ///
    /// Up to two operations are applied atomically: all `from=` preconditions
    /// are checked before any write is performed, so the certificate is never
    /// left in a partially-updated state.
    ///
    /// # Parameters
    ///
    /// - `certificate_id` — the certificate to edit.
    /// - `process_now` — whether to resubmit the certificate to the
    ///   orchestrator for reprocessing once edits are applied.  Pass
    ///   [`ProcessNow::True`] (`"process-now=true"`) to recover a stuck
    ///   certificate after fixing its state.
    /// - `operation_1`, `operation_2` — optional operation strings (see
    ///   [Operation format](#operation-format) below).  Omit or pass `null` for
    ///   unused slots.
    ///
    /// # Operation format
    ///
    /// Each operation is an ASCII string with a `from=` precondition and a
    /// `to=` target value.  The precondition is checked against the live
    /// certificate header **before** any write; a mismatch returns an error
    /// and leaves the certificate unchanged.
    ///
    /// ## `set-status`
    ///
    /// ```text
    /// set-status,from=<STATUS>,to=<STATUS>
    /// ```
    ///
    /// Changes the certificate's [`CertificateStatus`].
    ///
    /// Valid `<STATUS>` values: `Pending`, `Proven`, `Candidate`, `InError`,
    /// `Settled`.
    ///
    /// Special case for `InError`: when `from=InError` the inner error message
    /// is **not** compared — any `InError` variant will satisfy the
    /// precondition.  When `to=InError` the error is recorded as
    /// `"Set to InError by administrator"`.
    ///
    /// ## `set-settlement-tx-hash`
    ///
    /// ```text
    /// set-settlement-tx-hash,from=<HASH_OR_null>,to=<HASH_OR_null>
    /// ```
    ///
    /// Sets or clears the settlement transaction hash.  Use the literal
    /// `null` to represent the absence of a hash.  Any other value must be a
    /// hex-encoded [`SettlementTxHash`] (a 32-byte keccak digest).
    ///
    /// # Guards
    ///
    /// - A [`CertificateStatus::Settled`] certificate cannot be edited.
    /// - The `from=` value of every operation must exactly match the current
    ///   state of the certificate header (with the `InError` relaxation
    ///   described above).
    ///
    /// # Errors
    ///
    /// | Error | When |
    /// |---|---|
    /// | `INVALID_PARAMS` | Malformed operation string; `from=` mismatch; attempting to edit a `Settled` certificate |
    /// | `ResourceNotFound` (`-10008`) | No certificate header found for `certificate_id` |
    /// | `INTERNAL_ERROR` | Storage read/write failure; orchestrator channel failure |
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

    // ----- Settlement admin mutations -----
    //
    // Thin adapters over the settlement service's `admin_*` methods: no
    // handler touches settlement storage directly, so the service stays the
    // single choke point for settlement admin mutations.

    /// Append one new settlement attempt to a settlement job.
    ///
    /// **JSON-RPC method:** `admin_insertSettlementAttempt`
    ///
    /// Registers a settlement transaction the service does not know about
    /// (e.g. one submitted out-of-band, or ported from the legacy settlement
    /// path) so the settlement task tracks it like its own attempts. Only the
    /// transaction hash is mandatory: a missing sender or nonce is resolved
    /// by fetching the transaction from the L1 RPC (an unknown transaction is
    /// then rejected).
    ///
    /// This always adds one new attempt under the next unused attempt number
    /// — it never overwrites an existing attempt — and returns the assigned
    /// number. It fails if the job does not exist, or if it already has a
    /// terminal result and `force` is not `"force=true"` (see [`Force`]).
    #[method(name = "insertSettlementAttempt")]
    async fn insert_settlement_attempt(
        &self,
        job_id: SettlementJobId,
        attempt: InsertAttemptParams,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse>;

    /// Record that this attempt will never land on L1.
    ///
    /// **JSON-RPC method:** `admin_markSettlementAttemptDefinitelyFailed`
    ///
    /// Overwrites the attempt's result with a client error carrying the
    /// mandatory `reason`, bypassing the usual upgrade-only rule for attempt
    /// results. Terminal for the attempt, never for the job: the reloaded
    /// task stops waiting on the attempt and drives the settlement elsewhere
    /// (a wallet still under the service's control re-drives the same nonce;
    /// a rotated-away wallet falls through to a fresh nonce).
    ///
    /// This is a trusted operator assertion: if the transaction can still
    /// land, double settlement is only prevented by the settlement contract
    /// itself. Real on-chain evidence observed later supersedes the
    /// assertion. It fails if the attempt does not exist, or if the job
    /// already has a terminal result and `force` is not `"force=true"`
    /// (see [`Force`]).
    #[method(name = "markSettlementAttemptDefinitelyFailed")]
    async fn mark_settlement_attempt_definitely_failed(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        reason: String,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse>;

    /// Remove the recorded result of a settlement attempt.
    ///
    /// **JSON-RPC method:** `admin_removeSettlementAttemptResult`
    ///
    /// Hands the attempt back to the settlement task as pending, so the task
    /// re-derives its outcome from L1. This is the undo of
    /// `admin_markSettlementAttemptDefinitelyFailed` (and of any wrongly
    /// recorded client error). It fails if the attempt does not exist, no
    /// result is recorded, or if the job already has a terminal result and
    /// `force` is not `"force=true"` (see [`Force`]).
    #[method(name = "removeSettlementAttemptResult")]
    async fn remove_settlement_attempt_result(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse>;

    /// Remove the terminal result of a settlement job, turning it back into
    /// a pending job that is immediately re-driven.
    ///
    /// **JSON-RPC method:** `admin_forceRemoveSettlementJobResult`
    ///
    /// The job's stale completed-result watcher is dropped and a fresh
    /// settlement task is spawned from the stored job state. Attempts and
    /// their results are untouched: correct them **first**, with the attempt
    /// mutations' `"force=true"` parameter, while the terminal result still
    /// blocks the job from being re-driven — the fresh task spawned here
    /// could otherwise re-derive and re-record the removed result from the
    /// uncorrected attempts.
    ///
    /// **Force operation**: it un-completes a job. If the removed result was
    /// real, only the settlement contract's replay protection stands between
    /// the re-driven job and a double settlement. It fails if the job does
    /// not exist, has no terminal result, or still has a live task.
    #[method(name = "forceRemoveSettlementJobResult")]
    async fn force_remove_settlement_job_result(&self, job_id: SettlementJobId) -> RpcResult<()>;

    /// Stop the in-memory settlement task of a job.
    ///
    /// **JSON-RPC method:** `admin_abortSettlementTask`
    ///
    /// Runtime-only: the stored job is untouched and gets a fresh task on the
    /// next startup recovery. This is not a terminal job state.
    #[method(name = "abortSettlementTask")]
    async fn abort_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()>;

    /// Make the live settlement task of a job drop its in-memory state and
    /// reload from storage.
    ///
    /// **JSON-RPC method:** `admin_reloadSettlementTask`
    ///
    /// Escape hatch when a mutation reported a `live_task` status other than
    /// `notified`.
    #[method(name = "reloadSettlementTask")]
    async fn reload_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()>;
}

/// The Admin RPC agglayer service implementation.
pub struct AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
    settlement_service: SettlementService<L1Provider, StateStore>,
}

impl<PendingStore, StateStore, DebugStore, L1Provider>
    AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
{
    /// Create an instance of the admin RPC agglayer service.
    pub fn new(
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
        settlement_service: SettlementService<L1Provider, StateStore>,
    ) -> Self {
        Self {
            certificate_sender,
            pending_store,
            state,
            debug_store,
            config,
            settlement_service,
        }
    }
}

impl<PendingStore, StateStore, DebugStore, L1Provider>
    AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + SettlementReader + SettlementWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Provider: Provider + WalletProvider + 'static,
{
    /// Starts the admin JSON-RPC router.
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

impl<PendingStore, StateStore, DebugStore, L1Provider> Drop
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer service");
    }
}

#[async_trait]
impl<PendingStore, StateStore, DebugStore, L1Provider> AdminAgglayerServer
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + SettlementReader + SettlementWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Provider: Provider + WalletProvider + 'static,
{
    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self), fields(certificate_id = %certificate.hash()))]
    async fn force_push_pending_certificate(
        &self,
        certificate: Certificate,
        status: CertificateStatus,
    ) -> RpcResult<()> {
        warn!(
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

    #[instrument(skip(self))]
    async fn force_edit_certificate(
        &self,
        certificate_id: CertificateId,
        process_now: ProcessNow,
        operation_1: Option<String>,
        operation_2: Option<String>,
    ) -> RpcResult<()> {
        warn!("(ADMIN) Editing certificate");

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

    #[instrument(skip(self))]
    async fn set_latest_pending_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            "(ADMIN) Setting latest pending certificate: {}",
            certificate_id
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

    #[instrument(skip(self))]
    async fn set_latest_proven_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            "(ADMIN) Setting latest proven certificate: {}",
            certificate_id
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

    #[instrument(skip(self))]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!("(ADMIN) Removing pending proof: {}", certificate_id);

        self.pending_store
            .remove_generated_proof(&certificate_id)
            .map_err(|error| {
                error!("Failed to remove generated proof: {}", error);
                Error::internal("Unable to remove generated proof")
            })
    }

    #[instrument(skip(self), fields(certificate_id))]
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
        tracing::Span::current().record("certificate_id", certificate_id.to_string());

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
                    error!(?error, "Failed to remove generated proof");
                    Error::internal(format!(
                        "Failed to remove generated proof for certificate_id: {certificate_id}"
                    ))
                })?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_disabled_networks(&self) -> RpcResult<Vec<NetworkId>> {
        self.state.get_disabled_networks().map_err(|error| {
            error!(?error, "Failed to get disabled networks");
            Error::internal("Unable to get disabled networks")
        })
    }

    #[instrument(skip(self))]
    async fn disable_network(&self, network_id: NetworkId) -> RpcResult<()> {
        self.state
            .disable_network(&network_id, agglayer_types::network_info::DisabledBy::Admin)
            .map_err(|error| {
                error!(?error, "Failed to disable network {network_id}");
                Error::internal(format!("Unable to disable network {network_id}"))
            })
    }

    #[instrument(skip(self))]
    async fn enable_network(&self, network_id: NetworkId) -> RpcResult<()> {
        self.state.enable_network(&network_id).map_err(|error| {
            error!(?error, "Failed to enable network {network_id}");
            Error::internal(format!("Unable to enable network {network_id}"))
        })
    }

    #[instrument(skip(self))]
    async fn insert_settlement_attempt(
        &self,
        job_id: SettlementJobId,
        attempt: InsertAttemptParams,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse> {
        warn!("(ADMIN) Inserting settlement attempt for job {job_id}");
        let (attempt_number, live_task) = self
            .settlement_service
            .admin_insert_settlement_attempt(job_id, attempt.into(), edit_even_if_completed(force))
            .await
            .map_err(settlement_internal_error)?;
        Ok(MutationResponse {
            attempt_number,
            live_task,
        })
    }

    #[instrument(skip(self))]
    async fn mark_settlement_attempt_definitely_failed(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        reason: String,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse> {
        warn!(
            "(ADMIN) Marking settlement attempt {attempt_number} of job {job_id} as definitely \
             failed"
        );
        let live_task = self
            .settlement_service
            .admin_mark_attempt_definitely_failed(
                job_id,
                attempt_number,
                &reason,
                edit_even_if_completed(force),
            )
            .await
            .map_err(settlement_internal_error)?;
        Ok(MutationResponse {
            attempt_number,
            live_task,
        })
    }

    #[instrument(skip(self))]
    async fn remove_settlement_attempt_result(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        force: Option<Force>,
    ) -> RpcResult<MutationResponse> {
        warn!("(ADMIN) Removing result of settlement attempt {attempt_number} of job {job_id}");
        let live_task = self
            .settlement_service
            .admin_remove_attempt_result(job_id, attempt_number, edit_even_if_completed(force))
            .await
            .map_err(settlement_internal_error)?;
        Ok(MutationResponse {
            attempt_number,
            live_task,
        })
    }

    #[instrument(skip(self))]
    async fn force_remove_settlement_job_result(&self, job_id: SettlementJobId) -> RpcResult<()> {
        warn!("(ADMIN) Force-removing terminal result of settlement job {job_id}");
        self.settlement_service
            .admin_force_remove_settlement_job_result(job_id)
            .await
            .map_err(settlement_internal_error)
    }

    #[instrument(skip(self))]
    async fn abort_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()> {
        warn!("(ADMIN) Aborting settlement task for job {job_id}");
        self.settlement_service
            .admin_abort_task(job_id)
            .await
            .map_err(settlement_internal_error)
    }

    #[instrument(skip(self))]
    async fn reload_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()> {
        warn!("(ADMIN) Reloading settlement task for job {job_id}");
        self.settlement_service
            .admin_reload_and_restart_task(job_id)
            .await
            .map_err(settlement_internal_error)
    }
}
