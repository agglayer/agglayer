//! Read-only storage inspection commands.

mod pricing;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    future::{Future, IntoFuture},
    io::{
        BufRead as _, BufReader, BufWriter, ErrorKind, Read as _, Seek as _, SeekFrom, Write as _,
    },
    path::{Component, Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use agglayer_storage::stores::tree_snapshot::{
    SettledCertificateSnapshot, TokenBalance, TreeSnapshotReader,
};
use agglayer_types::{primitives::Hashable as _, SettlementTxHash, B256};
use alloy::{
    consensus::BlockHeader as _,
    eips::BlockNumberOrTag,
    network::{BlockResponse as _, ReceiptResponse as _},
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    transports::{
        layers::{RateLimitRetryPolicy, RetryPolicy as _},
        TransportError, TransportErrorKind, TransportResult,
    },
};
use chrono::{DateTime, SecondsFormat, Utc};
use eyre::{bail, ensure, Context as _, ContextCompat as _};
use futures::{stream, StreamExt as _};
use pessimistic_proof::unified_bridge::{
    BridgeExit, Claim, ImportedBridgeExit, LeafType, TokenInfo,
};
pub(crate) use pricing::enrich_tree_prices;
use serde::{ser::SerializeSeq as _, Deserialize, Serialize, Serializer as _};
use tempfile::{Builder as TempDirBuilder, TempDir};
use url::Url;

const LET_DIR: &str = "let";
const LBT_DIR: &str = "lbt";
const IBE_DIR: &str = "ibe";
const OUTPUT_DIRS: [&str; 3] = [LET_DIR, LBT_DIR, IBE_DIR];
const SETTLEMENT_RPC_CACHE_FILE: &str = ".agglayer-settlement-rpc-cache-v1.jsonl";
const SETTLEMENT_RPC_CACHE_VERSION: u8 = 1;
const SETTLEMENT_RPC_CACHE_MAX_RECORD_BYTES: usize = 4 * 1024;
// A settlement resolution can consume the complete four-attempt budget once
// for its receipt and once for its canonical block lookup. This leaves margin
// above the 92-second worst case when both operations use capped backoff hints.
const L1_RPC_TIMEOUT: Duration = Duration::from_secs(120);
const L1_RPC_CONCURRENCY: usize = 16;
const L1_RPC_RETRY_CONFIG: L1RpcRetryConfig = L1RpcRetryConfig {
    max_attempts: 4,
    attempt_timeout: Duration::from_secs(10),
    initial_backoff: Duration::from_millis(250),
    max_backoff: Duration::from_secs(2),
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct SettlementInfo {
    block_number: u64,
    block_hash: B256,
    block_timestamp: u64,
    settled_at: String,
}

impl SettlementInfo {
    fn new(block_number: u64, block_hash: B256, block_timestamp: u64) -> eyre::Result<Self> {
        let settled_at = format_unix_timestamp(block_timestamp)
            .context("L1 block timestamp is outside the supported UTC range")?;
        Ok(Self {
            block_number,
            block_hash,
            block_timestamp,
            settled_at,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct L1ChainBinding {
    chain_id: u64,
    genesis_block_hash: B256,
}

#[derive(Clone, Copy)]
struct L1RpcRetryConfig {
    max_attempts: u32,
    attempt_timeout: Duration,
    initial_backoff: Duration,
    max_backoff: Duration,
}

#[derive(Clone, Copy)]
enum L1RpcOperation {
    ChainId,
    GenesisBlock,
    TransactionReceipt,
    SettlementBlock,
}

impl L1RpcOperation {
    const fn description(self) -> &'static str {
        match self {
            Self::ChainId => "fetching the L1 chain ID",
            Self::GenesisBlock => "fetching the L1 genesis block",
            Self::TransactionReceipt => "fetching an L1 settlement receipt",
            Self::SettlementBlock => "fetching an L1 settlement block",
        }
    }

    const fn failure(self) -> &'static str {
        match self {
            Self::ChainId => "unable to fetch the chain ID from the L1 RPC",
            Self::GenesisBlock => "unable to fetch the genesis block from the L1 RPC",
            Self::TransactionReceipt => "unable to fetch transaction receipt from the L1 RPC",
            Self::SettlementBlock => "unable to fetch the settlement block from the L1 RPC",
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
enum SettlementRpcCacheRecord {
    #[serde(rename = "header")]
    Header {
        version: u8,
        #[serde(rename = "chainId")]
        chain_id: u64,
        #[serde(rename = "genesisBlockHash")]
        genesis_block_hash: B256,
    },
    #[serde(rename = "settlement")]
    Settlement {
        #[serde(rename = "settlementTxHash")]
        settlement_tx_hash: SettlementTxHash,
        #[serde(rename = "blockNumber")]
        block_number: u64,
        #[serde(rename = "blockHash")]
        block_hash: B256,
        #[serde(rename = "blockTimestamp")]
        block_timestamp: u64,
    },
}

struct SettlementRpcCache {
    path: PathBuf,
    state: Mutex<SettlementRpcCacheState>,
}

struct SettlementRpcCacheState {
    file: File,
    entries: HashMap<SettlementTxHash, SettlementInfo>,
    poisoned: bool,
}

impl SettlementRpcCache {
    fn open(output_root: &Path, binding: L1ChainBinding) -> eyre::Result<Self> {
        let path = output_root.join(SETTLEMENT_RPC_CACHE_FILE);
        let exists = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink() && metadata.is_file(),
                    "settlement RPC cache {} must be a regular file",
                    path.display()
                );
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!("failed to inspect settlement RPC cache {}", path.display())
                });
            }
        };

        let mut options = OpenOptions::new();
        options.read(true).append(true);
        #[cfg(unix)]
        options.custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32);
        if exists {
            options.write(true);
        } else {
            options.create_new(true);
        }
        let mut file = options
            .open(&path)
            .wrap_err_with(|| format!("failed to open settlement RPC cache {}", path.display()))?;
        let opened_metadata = file
            .metadata()
            .wrap_err("failed to inspect the opened settlement RPC cache")?;
        ensure!(
            opened_metadata.is_file(),
            "settlement RPC cache {} must be a regular file",
            path.display()
        );
        #[cfg(unix)]
        ensure!(
            opened_metadata.nlink() == 1,
            "settlement RPC cache {} must not be hard-linked",
            path.display()
        );
        file.try_lock().wrap_err_with(|| {
            format!(
                "settlement RPC cache {} is already in use by another exporter",
                path.display()
            )
        })?;

        let entries = if exists {
            Self::load(&mut file, &path, binding)?
        } else {
            write_cache_record(
                &mut file,
                &SettlementRpcCacheRecord::Header {
                    version: SETTLEMENT_RPC_CACHE_VERSION,
                    chain_id: binding.chain_id,
                    genesis_block_hash: binding.genesis_block_hash,
                },
                &path,
            )?;
            file.flush()
                .wrap_err("failed to flush the settlement RPC cache header")?;
            file.sync_all()
                .wrap_err("failed to sync the settlement RPC cache header")?;
            sync_directory(output_root)?;
            HashMap::new()
        };

        Ok(Self {
            path,
            state: Mutex::new(SettlementRpcCacheState {
                file,
                entries,
                poisoned: false,
            }),
        })
    }

    fn load(
        file: &mut File,
        path: &Path,
        binding: L1ChainBinding,
    ) -> eyre::Result<HashMap<SettlementTxHash, SettlementInfo>> {
        file.seek(SeekFrom::Start(0))
            .wrap_err("failed to seek the settlement RPC cache")?;
        let reader_file = file
            .try_clone()
            .wrap_err("failed to clone the settlement RPC cache handle")?;
        let mut reader = BufReader::new(reader_file);
        let mut entries = HashMap::new();
        let mut complete_offset = 0_u64;
        let mut line_number = 0_u64;
        let mut saw_header = false;

        loop {
            let mut line = Vec::new();
            let read = reader
                .by_ref()
                .take((SETTLEMENT_RPC_CACHE_MAX_RECORD_BYTES + 1) as u64)
                .read_until(b'\n', &mut line)
                .wrap_err("failed to read the settlement RPC cache")?;
            if read == 0 {
                break;
            }
            ensure!(
                read <= SETTLEMENT_RPC_CACHE_MAX_RECORD_BYTES,
                "settlement RPC cache {} has an oversized record after line {line_number}",
                path.display()
            );
            if !line.ends_with(b"\n") {
                ensure!(
                    saw_header,
                    "settlement RPC cache {} has an incomplete header; refusing to modify it",
                    path.display()
                );
                eprintln!(
                    "warning: discarding an incomplete final record from settlement RPC cache {}",
                    path.display()
                );
                file.set_len(complete_offset)
                    .wrap_err("failed to truncate an incomplete settlement RPC cache record")?;
                file.sync_data()
                    .wrap_err("failed to sync the repaired settlement RPC cache")?;
                break;
            }

            line_number = line_number
                .checked_add(1)
                .context("settlement RPC cache has too many lines")?;
            complete_offset = complete_offset
                .checked_add(
                    u64::try_from(read)
                        .wrap_err("settlement RPC cache record length exceeds u64")?,
                )
                .context("settlement RPC cache length overflow")?;
            line.pop();
            ensure!(
                !line.is_empty(),
                "settlement RPC cache {} has an empty record at line {line_number}",
                path.display()
            );
            let record: SettlementRpcCacheRecord =
                serde_json::from_slice(&line).wrap_err_with(|| {
                    format!(
                        "settlement RPC cache {} has invalid JSON at line {line_number}",
                        path.display()
                    )
                })?;

            match record {
                SettlementRpcCacheRecord::Header {
                    version,
                    chain_id,
                    genesis_block_hash,
                } => {
                    ensure!(
                        line_number == 1 && !saw_header,
                        "settlement RPC cache {} has an unexpected header at line {line_number}",
                        path.display()
                    );
                    ensure!(
                        version == SETTLEMENT_RPC_CACHE_VERSION,
                        "settlement RPC cache {} uses unsupported version {version}",
                        path.display()
                    );
                    ensure!(
                        chain_id == binding.chain_id
                            && genesis_block_hash == binding.genesis_block_hash,
                        "settlement RPC cache {} belongs to a different L1 chain",
                        path.display()
                    );
                    saw_header = true;
                }
                SettlementRpcCacheRecord::Settlement {
                    settlement_tx_hash,
                    block_number,
                    block_hash,
                    block_timestamp,
                } => {
                    ensure!(
                        saw_header,
                        "settlement RPC cache {} is missing its header before line {line_number}",
                        path.display()
                    );
                    let info = SettlementInfo::new(block_number, block_hash, block_timestamp)
                        .wrap_err_with(|| {
                            format!(
                                "settlement RPC cache {} has an invalid timestamp at line \
                                 {line_number}",
                                path.display()
                            )
                        })?;
                    if let Some(previous) = entries.insert(settlement_tx_hash, info.clone()) {
                        ensure!(
                            previous == info,
                            "settlement RPC cache {} has conflicting records for transaction {} \
                             at line {line_number}",
                            path.display(),
                            format_settlement_hash(settlement_tx_hash)
                        );
                    }
                }
            }
        }

        file.seek(SeekFrom::End(0))
            .wrap_err("failed to seek to the end of the settlement RPC cache")?;
        ensure!(
            saw_header,
            "settlement RPC cache {} is empty or missing its header; refusing to modify it",
            path.display()
        );
        Ok(entries)
    }

    fn get(&self, tx_hash: SettlementTxHash) -> eyre::Result<Option<SettlementInfo>> {
        let state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("settlement RPC cache lock is poisoned"))?;
        Ok(state.entries.get(&tx_hash).cloned())
    }

    fn append(&self, tx_hash: SettlementTxHash, info: &SettlementInfo) -> eyre::Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("settlement RPC cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "settlement RPC cache is unavailable after an earlier write failure"
        );
        if let Some(previous) = state.entries.get(&tx_hash) {
            ensure!(
                previous == info,
                "refusing to append conflicting settlement data for transaction {}",
                format_settlement_hash(tx_hash)
            );
            return Ok(());
        }

        let committed_len = state
            .file
            .metadata()
            .wrap_err("failed to inspect the settlement RPC cache before appending")?
            .len();
        if let Err(write_error) = write_cache_record(
            &mut state.file,
            &SettlementRpcCacheRecord::Settlement {
                settlement_tx_hash: tx_hash,
                block_number: info.block_number,
                block_hash: info.block_hash,
                block_timestamp: info.block_timestamp,
            },
            &self.path,
        ) {
            state.poisoned = true;
            let rollback = state
                .file
                .set_len(committed_len)
                .and_then(|()| state.file.sync_data());
            return match rollback {
                Ok(()) => Err(write_error),
                Err(rollback_error) => Err(write_error).wrap_err(format!(
                    "additionally failed to roll back the partial settlement RPC cache record: \
                     {rollback_error}"
                )),
            };
        }
        state
            .file
            .flush()
            .wrap_err("failed to flush a settlement RPC cache record")?;
        state.entries.insert(tx_hash, info.clone());
        Ok(())
    }

    fn sync(&self) -> eyre::Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("settlement RPC cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "settlement RPC cache is unavailable after an earlier write failure"
        );
        state
            .file
            .flush()
            .wrap_err("failed to flush the settlement RPC cache")?;
        state
            .file
            .sync_data()
            .wrap_err("failed to sync the settlement RPC cache")
    }
}

fn write_cache_record(
    file: &mut File,
    record: &SettlementRpcCacheRecord,
    path: &Path,
) -> eyre::Result<()> {
    let mut encoded = serde_json::to_vec(record).wrap_err_with(|| {
        format!(
            "failed to serialize settlement RPC cache {}",
            path.display()
        )
    })?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .wrap_err_with(|| format!("failed to append settlement RPC cache {}", path.display()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeExitJson {
    leaf_type: &'static str,
    /// Token information committed in the bridge-exit leaf.
    token: String,
    /// Token in which `amount` is denominated. This differs from `token` for
    /// message leaves, whose amount is accounted as native L1 ETH.
    amount_token: String,
    amount: String,
    destination_network: u32,
    destination_address: String,
    metadata: Option<String>,
}

impl BridgeExitJson {
    fn new(bridge_exit: &BridgeExit) -> Self {
        Self {
            leaf_type: match bridge_exit.leaf_type {
                LeafType::Transfer => "transfer",
                LeafType::Message => "message",
            },
            token: format_token(bridge_exit.token_info),
            amount_token: format_token(bridge_exit.amount_token_info()),
            amount: bridge_exit.amount.to_string(),
            destination_network: bridge_exit.dest_network.to_u32(),
            destination_address: format!("{:#x}", bridge_exit.dest_address),
            metadata: bridge_exit
                .metadata
                .map(|metadata| format!("{metadata:#x}")),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CertificateSettlementJson {
    certificate_id: String,
    certificate_height: u64,
    epoch_number: u64,
    certificate_index: u64,
    settlement_tx_hash: String,
    settlement_block_number: Option<u64>,
    settlement_block_hash: Option<String>,
    settled_at: Option<String>,
}

impl CertificateSettlementJson {
    fn new(snapshot: &SettledCertificateSnapshot, settlement: Option<&SettlementInfo>) -> Self {
        Self {
            certificate_id: snapshot.certificate_id.to_string(),
            certificate_height: snapshot.certificate.height.as_u64(),
            epoch_number: snapshot.epoch_number.as_u64(),
            certificate_index: snapshot.certificate_index.as_u64(),
            settlement_tx_hash: format_settlement_hash(snapshot.settlement_tx_hash),
            settlement_block_number: settlement.map(|value| value.block_number),
            settlement_block_hash: settlement.map(|value| format!("{:#x}", value.block_hash)),
            settled_at: settlement.map(|value| value.settled_at.clone()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExitJson<'a> {
    leaf_index: u32,
    leaf_hash: String,
    #[serde(flatten)]
    bridge_exit: BridgeExitJson,
    #[serde(flatten)]
    settlement: &'a CertificateSettlementJson,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportedBridgeExitJson<'a> {
    imported_exit_index: u32,
    imported_bridge_exit_hash: String,
    bridge_exit_hash: String,
    #[serde(flatten)]
    bridge_exit: BridgeExitJson,
    source_network: u32,
    source_leaf_index: u32,
    global_index: String,
    global_index_hex: String,
    mainnet: bool,
    rollup_index: Option<u32>,
    claim_type: &'static str,
    l1_info_root: String,
    l1_info_tree_leaf_index: u32,
    l1_info_tree_leaf_hash: String,
    global_exit_root: String,
    mainnet_exit_root: String,
    rollup_exit_root: String,
    l1_info_tree_block_hash: String,
    l1_info_tree_timestamp: String,
    l1_info_tree_at: Option<String>,
    #[serde(flatten)]
    settlement: &'a CertificateSettlementJson,
}

impl<'a> ImportedBridgeExitJson<'a> {
    fn new(
        imported_exit_index: u32,
        imported_exit: &ImportedBridgeExit,
        settlement: &'a CertificateSettlementJson,
    ) -> Self {
        let global_index = imported_exit.global_index;
        let global_index_value = global_index.into_u256();
        let (claim_type, l1_leaf, l1_info_root) = match &imported_exit.claim_data {
            Claim::Mainnet(claim) => ("mainnet", &claim.l1_leaf, claim.proof_ger_l1root.root),
            Claim::Rollup(claim) => ("rollup", &claim.l1_leaf, claim.proof_ger_l1root.root),
        };
        let timestamp = l1_leaf.inner.timestamp;
        let l1_info_tree_at = format_unix_timestamp(timestamp);
        if l1_info_tree_at.is_none() {
            eprintln!(
                "warning: L1 info-tree timestamp {timestamp} for certificate {} imported exit {} \
                 is outside the supported UTC range; continuing with l1InfoTreeAt null",
                settlement.certificate_id, imported_exit_index
            );
        }

        Self {
            imported_exit_index,
            imported_bridge_exit_hash: format!("{:#x}", imported_exit.hash()),
            bridge_exit_hash: format!("{:#x}", imported_exit.bridge_exit.hash()),
            bridge_exit: BridgeExitJson::new(&imported_exit.bridge_exit),
            source_network: global_index.network_id().to_u32(),
            source_leaf_index: global_index.leaf_index(),
            global_index: global_index_value.to_string(),
            global_index_hex: format!("{global_index_value:#x}"),
            mainnet: global_index.is_mainnet(),
            rollup_index: global_index.rollup_index().map(|index| index.to_u32()),
            claim_type,
            l1_info_root: format!("{l1_info_root:#x}"),
            l1_info_tree_leaf_index: l1_leaf.l1_info_tree_index,
            l1_info_tree_leaf_hash: format!("{:#x}", l1_leaf.hash()),
            global_exit_root: format!("{:#x}", l1_leaf.inner.global_exit_root),
            mainnet_exit_root: format!("{:#x}", l1_leaf.mer),
            rollup_exit_root: format!("{:#x}", l1_leaf.rer),
            l1_info_tree_block_hash: format!("{:#x}", l1_leaf.inner.block_hash),
            l1_info_tree_timestamp: timestamp.to_string(),
            l1_info_tree_at,
            settlement,
        }
    }
}

/// Exports all validated local and imported bridge exits and the current local
/// balance tree for each settled network in a copied Agglayer storage
/// directory.
pub(crate) async fn export_trees(
    storage_path: &Path,
    output_path: &Path,
    l1_rpc_url: Option<Url>,
) -> eyre::Result<()> {
    let storage_path = canonical_storage_path(storage_path)?;
    let reader = TreeSnapshotReader::open(&storage_path)
        .wrap_err_with(|| format!("failed to open copied storage {}", storage_path.display()))?;
    let output = OutputWorkspace::prepare(&storage_path, output_path)?;

    write_snapshots(
        output.staging_path(),
        output.root_path(),
        &reader,
        l1_rpc_url.as_ref(),
    )
    .await?;
    output.publish()?;

    Ok(())
}

async fn resolve_settlements<P: Provider>(
    hashes: impl IntoIterator<Item = SettlementTxHash>,
    provider: &P,
    cache: &SettlementRpcCache,
) -> eyre::Result<HashMap<SettlementTxHash, SettlementInfo>> {
    // Key by the printable hash so iteration and RPC error ordering are stable.
    let mut unique_hashes = BTreeMap::new();
    for hash in hashes {
        unique_hashes
            .entry(format_settlement_hash(hash))
            .or_insert(hash);
    }

    let mut settlements = HashMap::new();
    let mut missing = Vec::new();
    for tx_hash in unique_hashes.into_values() {
        if let Some(info) = cache.get(tx_hash)? {
            settlements.insert(tx_hash, info);
        } else {
            missing.push(tx_hash);
        }
    }

    let resolved = resolve_settlement_hashes(missing, |tx_hash| async move {
        let info = resolve_settlement(provider, tx_hash).await?;
        cache.append(tx_hash, &info)?;
        Ok(info)
    })
    .await;
    // All successful lookups are flushed as they complete and synced here,
    // including when another concurrent lookup makes the batch fail. This is
    // what lets the next invocation resume without repeating those RPC calls.
    cache.sync()?;
    settlements.extend(resolved?);
    Ok(settlements)
}

async fn resolve_settlement_hashes<I, F, Fut>(
    hashes: I,
    resolver: F,
) -> eyre::Result<HashMap<SettlementTxHash, SettlementInfo>>
where
    I: IntoIterator<Item = SettlementTxHash>,
    F: Fn(SettlementTxHash) -> Fut,
    Fut: Future<Output = eyre::Result<SettlementInfo>>,
{
    let mut resolutions = Box::pin(
        stream::iter(hashes)
            .map(|tx_hash| {
                let resolution = resolver(tx_hash);
                async move {
                    let info = tokio::time::timeout(L1_RPC_TIMEOUT, resolution)
                        .await
                        .wrap_err("timed out resolving a settlement transaction")?
                        .wrap_err_with(|| {
                            format!(
                                "failed to resolve settlement transaction {} using the L1 RPC",
                                format_settlement_hash(tx_hash)
                            )
                        })?;
                    Ok((tx_hash, info))
                }
            })
            // `buffered` polls requests concurrently but yields them in the stable
            // hash order supplied above, including deterministic first-error
            // reporting.
            .buffered(L1_RPC_CONCURRENCY),
    );
    let mut settlements = HashMap::new();
    let mut first_error = None;
    while let Some(resolution) = resolutions.next().await {
        match resolution {
            Ok((tx_hash, info)) => {
                settlements.insert(tx_hash, info);
            }
            Err(error) if first_error.is_none() => first_error = Some(error),
            Err(_) => {}
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(settlements)
    }
}

async fn retry_l1_rpc<T, F, C>(operation: L1RpcOperation, request: F) -> eyre::Result<T>
where
    F: FnMut() -> C,
    C: IntoFuture<Output = TransportResult<T>>,
{
    retry_l1_rpc_with_config(operation, L1_RPC_RETRY_CONFIG, request).await
}

async fn retry_l1_rpc_with_config<T, F, C>(
    operation: L1RpcOperation,
    config: L1RpcRetryConfig,
    mut request: F,
) -> eyre::Result<T>
where
    F: FnMut() -> C,
    C: IntoFuture<Output = TransportResult<T>>,
{
    ensure!(
        config.max_attempts > 0,
        "L1 RPC max attempts must be positive"
    );
    let policy = RateLimitRetryPolicy::default();
    let mut next_backoff = config.initial_backoff;

    for attempt in 1..=config.max_attempts {
        let outcome = tokio::time::timeout(config.attempt_timeout, request().into_future()).await;
        let retry_delay = match outcome {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(error)) => {
                if attempt == config.max_attempts || !is_retryable_l1_rpc_error(&error) {
                    bail!("{} after {attempt} attempt(s)", operation.failure());
                }
                policy
                    .backoff_hint(&error)
                    .unwrap_or(next_backoff)
                    .min(config.max_backoff)
            }
            Err(_) => {
                if attempt == config.max_attempts {
                    bail!(
                        "{} after {attempt} timed-out attempt(s)",
                        operation.failure()
                    );
                }
                next_backoff
            }
        };

        eprintln!(
            "warning: transient L1 RPC failure while {}; retrying attempt {}/{} in {} ms",
            operation.description(),
            attempt + 1,
            config.max_attempts,
            retry_delay.as_millis()
        );
        tokio::time::sleep(retry_delay).await;
        next_backoff = next_backoff.saturating_mul(2).min(config.max_backoff);
    }

    unreachable!("positive retry attempt count always returns from the loop")
}

fn is_retryable_l1_rpc_error(error: &TransportError) -> bool {
    if RateLimitRetryPolicy::default().should_retry(error) {
        return true;
    }

    match error {
        TransportError::Transport(TransportErrorKind::BackendGone)
        | TransportError::Transport(TransportErrorKind::Custom(_)) => true,
        TransportError::Transport(TransportErrorKind::HttpError(error)) => {
            matches!(error.status, 408 | 425 | 500 | 502 | 504)
        }
        _ => false,
    }
}

async fn resolve_l1_chain_binding<P: Provider>(provider: &P) -> eyre::Result<L1ChainBinding> {
    let chain_id = retry_l1_rpc(L1RpcOperation::ChainId, || provider.get_chain_id()).await?;
    let genesis_block = retry_l1_rpc(L1RpcOperation::GenesisBlock, || async {
        provider
            .get_block_by_number(BlockNumberOrTag::Number(0))
            .await?
            .ok_or(TransportError::NullResp)
    })
    .await?;
    ensure!(
        genesis_block.header().number() == 0,
        "L1 RPC returned block number {} when genesis block 0 was requested",
        genesis_block.header().number()
    );

    Ok(L1ChainBinding {
        chain_id,
        genesis_block_hash: genesis_block.header().hash,
    })
}

async fn resolve_settlement<P: Provider>(
    provider: &P,
    settlement_tx_hash: SettlementTxHash,
) -> eyre::Result<SettlementInfo> {
    let tx_hash: TxHash = settlement_tx_hash.into();
    let receipt = retry_l1_rpc(L1RpcOperation::TransactionReceipt, || async {
        provider
            .get_transaction_receipt(tx_hash)
            .await?
            .ok_or(TransportError::NullResp)
    })
    .await?;

    ensure!(
        receipt.transaction_hash() == tx_hash,
        "L1 RPC returned receipt for transaction {} instead of requested transaction {}",
        receipt.transaction_hash(),
        tx_hash
    );
    ensure!(receipt.status(), "settlement transaction reverted");
    let receipt_block_hash = receipt
        .block_hash()
        .context("settlement receipt has no block hash")?;
    let receipt_block_number = receipt
        .block_number()
        .context("settlement receipt has no block number")?;

    // Resolve by number, then compare hashes. Looking up only by hash would not
    // establish that the receipt's block is still canonical after a reorg.
    let canonical_block = retry_l1_rpc(L1RpcOperation::SettlementBlock, || async {
        provider
            .get_block_by_number(BlockNumberOrTag::Number(receipt_block_number))
            .await?
            .ok_or(TransportError::NullResp)
    })
    .await?;
    let canonical_hash = canonical_block.header().hash;
    let canonical_number = canonical_block.header().number();
    ensure!(
        canonical_number == receipt_block_number,
        "L1 RPC returned block number {canonical_number} when settlement block number \
         {receipt_block_number} was requested"
    );
    ensure!(
        canonical_hash == receipt_block_hash,
        "settlement receipt block {receipt_block_hash} is not canonical at height \
         {receipt_block_number} (canonical hash: {canonical_hash})"
    );

    SettlementInfo::new(
        receipt_block_number,
        canonical_hash,
        canonical_block.header().timestamp(),
    )
}

async fn write_snapshots(
    staging_path: &Path,
    output_root: &Path,
    reader: &TreeSnapshotReader,
    l1_rpc_url: Option<&Url>,
) -> eyre::Result<()> {
    let let_path = staging_path.join(LET_DIR);
    let lbt_path = staging_path.join(LBT_DIR);
    let ibe_path = staging_path.join(IBE_DIR);
    fs::create_dir(&let_path).wrap_err("failed to create staged LET directory")?;
    fs::create_dir(&lbt_path).wrap_err("failed to create staged LBT directory")?;
    fs::create_dir(&ibe_path).wrap_err("failed to create staged IBE directory")?;

    let rpc = match l1_rpc_url {
        Some(rpc_url) => {
            let provider = ProviderBuilder::new().connect_http(rpc_url.clone());
            let binding = resolve_l1_chain_binding(&provider)
                .await
                .wrap_err("failed to identify the L1 RPC chain")?;
            let cache = SettlementRpcCache::open(output_root, binding)?;
            Some((provider, cache))
        }
        None => None,
    };

    for network_id in reader.network_ids() {
        let raw_network_id = network_id.to_u32();
        // Bound optional enrichment state to one network. The exit histories
        // themselves are still streamed one certificate at a time below.
        let settlements = match rpc.as_ref() {
            Some((provider, cache)) => {
                let hashes = reader
                    .network_settlement_tx_hashes(network_id)
                    .wrap_err_with(|| {
                        format!(
                            "failed to read settlement transaction hashes for network \
                             {raw_network_id}"
                        )
                    })?;
                resolve_settlements(hashes, provider, cache).await?
            }
            None => HashMap::new(),
        };
        write_network_bridge_history(
            &let_path.join(format!("{raw_network_id}.json")),
            &ibe_path.join(format!("{raw_network_id}.json")),
            &settlements,
            |visitor| {
                reader.try_visit_network_certificates_with_warnings(
                    network_id,
                    |warning| eprintln!("warning: {warning}; continuing with validated fallback"),
                    visitor,
                )
            },
        )?;

        let balances = reader
            .read_network_balances(network_id, |warning| {
                eprintln!("warning: {warning}; continuing best-effort");
            })
            .wrap_err_with(|| {
                format!("failed to read balance tree for network {raw_network_id}")
            })?;
        write_balances(
            &lbt_path.join(format!("{raw_network_id}.json")),
            raw_network_id,
            &balances,
        )?;
    }

    sync_directory(&let_path)?;
    sync_directory(&lbt_path)?;
    sync_directory(&ibe_path)?;
    sync_directory(staging_path)?;
    if let Some((_, cache)) = rpc.as_ref() {
        cache.sync()?;
    }
    Ok(())
}

fn write_network_bridge_history<F>(
    let_path: &Path,
    ibe_path: &Path,
    settlements: &HashMap<SettlementTxHash, SettlementInfo>,
    visit: F,
) -> eyre::Result<()>
where
    F: FnOnce(&mut dyn FnMut(SettledCertificateSnapshot) -> eyre::Result<()>) -> eyre::Result<()>,
{
    let let_file = create_output_file(let_path)?;
    let let_writer = BufWriter::new(let_file);
    let mut let_serializer = serde_json::Serializer::pretty(let_writer);
    let mut let_sequence = let_serializer
        .serialize_seq(None)
        .wrap_err_with(|| format!("failed to begin serializing {}", let_path.display()))?;

    let ibe_file = create_output_file(ibe_path)?;
    let ibe_writer = BufWriter::new(ibe_file);
    let mut ibe_serializer = serde_json::Serializer::pretty(ibe_writer);
    let mut ibe_sequence = ibe_serializer
        .serialize_seq(None)
        .wrap_err_with(|| format!("failed to begin serializing {}", ibe_path.display()))?;

    let mut write_certificate = |snapshot: SettledCertificateSnapshot| {
        let settlement = CertificateSettlementJson::new(
            &snapshot,
            settlements.get(&snapshot.settlement_tx_hash),
        );

        for (offset, bridge_exit) in snapshot.certificate.bridge_exits.iter().enumerate() {
            let offset = u32::try_from(offset)
                .wrap_err("certificate contains more local exits than u32 can index")?;
            let leaf_index = snapshot
                .first_local_exit_index
                .checked_add(offset)
                .context("local exit index overflow")?;
            let value = ExitJson {
                leaf_index,
                leaf_hash: format!("{:#x}", bridge_exit.hash()),
                bridge_exit: BridgeExitJson::new(bridge_exit),
                settlement: &settlement,
            };
            let_sequence
                .serialize_element(&value)
                .wrap_err_with(|| format!("failed to serialize {}", let_path.display()))?;
        }

        for (index, imported_exit) in snapshot
            .certificate
            .imported_bridge_exits
            .iter()
            .enumerate()
        {
            let index = u32::try_from(index)
                .wrap_err("certificate contains more imported exits than u32 can index")?;
            let value = ImportedBridgeExitJson::new(index, imported_exit, &settlement);
            ibe_sequence
                .serialize_element(&value)
                .wrap_err_with(|| format!("failed to serialize {}", ibe_path.display()))?;
        }

        Ok(())
    };

    visit(&mut write_certificate)?;
    let_sequence
        .end()
        .wrap_err_with(|| format!("failed to finish serializing {}", let_path.display()))?;
    ibe_sequence
        .end()
        .wrap_err_with(|| format!("failed to finish serializing {}", ibe_path.display()))?;
    finish_output_file(let_serializer.into_inner(), let_path)?;
    finish_output_file(ibe_serializer.into_inner(), ibe_path)
}

fn write_balances(path: &Path, network_id: u32, values: &[TokenBalance]) -> eyre::Result<()> {
    let mut balances = BTreeMap::new();
    for balance in values {
        let previous = balances.insert(format_token(balance.token), balance.amount.to_string());
        ensure!(
            previous.is_none(),
            "balance tree for network {network_id} contains a duplicate token"
        );
    }
    write_json(path, &balances)
}

fn write_json(path: &Path, value: &impl Serialize) -> eyre::Result<()> {
    let file = create_output_file(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)
        .wrap_err_with(|| format!("failed to serialize {}", path.display()))?;
    finish_output_file(writer, path)
}

fn create_output_file(path: &Path) -> eyre::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("failed to create {}", path.display()))
}

fn finish_output_file(mut writer: BufWriter<File>, path: &Path) -> eyre::Result<()> {
    writer
        .write_all(b"\n")
        .wrap_err_with(|| format!("failed to finish {}", path.display()))?;
    writer
        .flush()
        .wrap_err_with(|| format!("failed to flush {}", path.display()))?;
    writer
        .get_ref()
        .sync_all()
        .wrap_err_with(|| format!("failed to sync {}", path.display()))?;
    Ok(())
}

fn format_token(token: TokenInfo) -> String {
    format!(
        "{}:{:#x}",
        token.origin_network.to_u32(),
        token.origin_token_address
    )
}

fn format_settlement_hash(hash: SettlementTxHash) -> String {
    let hash: B256 = hash.into();
    format!("{hash:#x}")
}

fn format_unix_timestamp(timestamp: u64) -> Option<String> {
    let timestamp = i64::try_from(timestamp).ok()?;
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn canonical_storage_path(path: &Path) -> eyre::Result<PathBuf> {
    reject_parent_components(path, "storage")?;
    let path = fs::canonicalize(path)
        .wrap_err_with(|| format!("failed to resolve storage path {}", path.display()))?;
    ensure!(
        path.is_dir(),
        "storage path {} is not a directory",
        path.display()
    );
    ensure!(
        path.join("state").is_dir(),
        "storage path {} has no state directory",
        path.display()
    );
    ensure!(
        path.join("epochs").is_dir(),
        "storage path {} has no epochs directory",
        path.display()
    );
    Ok(path)
}

fn reject_parent_components(path: &Path, label: &str) -> eyre::Result<()> {
    ensure!(
        !path
            .components()
            .any(|component| component == Component::ParentDir),
        "{label} path must not contain '..': {}",
        path.display()
    );
    Ok(())
}

fn resolve_output_path(path: &Path) -> eyre::Result<PathBuf> {
    reject_parent_components(path, "output")?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .wrap_err("failed to resolve current directory")?
            .join(path)
    };

    let mut missing = Vec::new();
    let mut existing = absolute.as_path();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => break,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let name = existing.file_name().with_context(|| {
                    format!("unable to resolve output path {}", absolute.display())
                })?;
                missing.push(name.to_owned());
                existing = existing.parent().with_context(|| {
                    format!("unable to resolve output path {}", absolute.display())
                })?;
            }
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!("failed to inspect output path {}", existing.display())
                });
            }
        }
    }

    let mut resolved = fs::canonicalize(existing)
        .wrap_err_with(|| format!("failed to resolve output path {}", existing.display()))?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn protected_storage_paths(storage_path: &Path) -> eyre::Result<Vec<PathBuf>> {
    let mut paths = vec![storage_path.to_path_buf()];
    for path in [storage_path.join("state"), storage_path.join("epochs")] {
        paths.push(
            fs::canonicalize(&path)
                .wrap_err_with(|| format!("failed to resolve database path {}", path.display()))?,
        );
    }

    for entry in fs::read_dir(storage_path.join("epochs"))
        .wrap_err("failed to inspect epoch database directory")?
    {
        let entry = entry.wrap_err("failed to inspect an epoch database entry")?;
        if entry
            .file_type()
            .wrap_err("failed to inspect an epoch database entry type")?
            .is_dir()
            || entry.path().is_dir()
        {
            paths.push(fs::canonicalize(entry.path()).wrap_err_with(|| {
                format!(
                    "failed to resolve epoch database path {}",
                    entry.path().display()
                )
            })?);
        }
    }
    Ok(paths)
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn ensure_no_output_collision(output_path: &Path) -> eyre::Result<()> {
    for name in OUTPUT_DIRS {
        let path = output_path.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => bail!(
                "refusing to overwrite existing output path {}",
                path.display()
            ),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .wrap_err_with(|| format!("failed to inspect {}", path.display()));
            }
        }
    }
    Ok(())
}

struct OutputWorkspace {
    root: PathBuf,
    staging: Option<TempDir>,
    remove_root_if_empty: bool,
}

struct RemoveEmptyDirectoryOnDrop {
    path: PathBuf,
    armed: bool,
}

impl RemoveEmptyDirectoryOnDrop {
    fn new(path: PathBuf, armed: bool) -> Self {
        Self { path, armed }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for RemoveEmptyDirectoryOnDrop {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir(&self.path);
        }
    }
}

impl OutputWorkspace {
    fn prepare(storage_path: &Path, requested_output: &Path) -> eyre::Result<Self> {
        let intended_output = resolve_output_path(requested_output)?;
        for protected in protected_storage_paths(storage_path)? {
            ensure!(
                !paths_overlap(&intended_output, &protected),
                "output path {} overlaps storage path {}",
                intended_output.display(),
                protected.display()
            );
        }

        let existed = match fs::symlink_metadata(requested_output) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "output path {} must not be a symlink",
                    requested_output.display()
                );
                ensure!(
                    metadata.is_dir(),
                    "output path {} is not a directory",
                    requested_output.display()
                );
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!(
                        "failed to inspect output path {}",
                        requested_output.display()
                    )
                });
            }
        };

        if !existed {
            fs::create_dir_all(requested_output).wrap_err_with(|| {
                format!(
                    "failed to create output directory {}",
                    requested_output.display()
                )
            })?;
        }
        let mut new_root_cleanup =
            RemoveEmptyDirectoryOnDrop::new(requested_output.to_path_buf(), !existed);
        let root = fs::canonicalize(requested_output).wrap_err_with(|| {
            format!(
                "failed to resolve output directory {}",
                requested_output.display()
            )
        })?;

        for protected in protected_storage_paths(storage_path)? {
            ensure!(
                !paths_overlap(&root, &protected),
                "output path {} overlaps storage path {}",
                root.display(),
                protected.display()
            );
        }
        ensure_no_output_collision(&root)?;

        let staging = TempDirBuilder::new()
            .prefix(".agglayer-tree-export-")
            .tempdir_in(&root)
            .wrap_err_with(|| {
                format!("failed to create staging directory in {}", root.display())
            })?;

        let workspace = Self {
            root,
            staging: Some(staging),
            remove_root_if_empty: !existed,
        };
        new_root_cleanup.disarm();
        Ok(workspace)
    }

    fn staging_path(&self) -> &Path {
        self.staging
            .as_ref()
            .expect("staging directory exists until publication")
            .path()
    }

    fn root_path(&self) -> &Path {
        &self.root
    }

    fn publish(mut self) -> eyre::Result<()> {
        ensure_no_output_collision(&self.root)?;

        let mut published: Vec<&str> = Vec::new();
        for name in OUTPUT_DIRS {
            let staged = self.staging_path().join(name);
            let final_path = self.root.join(name);
            if let Err(publish_error) = rename_noreplace(&staged, &final_path) {
                let mut rollback_errors = Vec::new();
                for published_name in published.iter().rev() {
                    let published_path = self.root.join(published_name);
                    let staged_path = self.staging_path().join(published_name);
                    if let Err(error) = rename_noreplace(&published_path, &staged_path) {
                        rollback_errors.push(format!(
                            "{} output: {error}",
                            published_name.to_ascii_uppercase()
                        ));
                    }
                }

                if rollback_errors.is_empty() {
                    return Err(publish_error).wrap_err_with(|| {
                        format!("failed to publish {} output", name.to_ascii_uppercase())
                    });
                }
                bail!(
                    "failed to publish {} output: {publish_error}; additionally failed to roll \
                     back {}",
                    name.to_ascii_uppercase(),
                    rollback_errors.join(", ")
                );
            }
            published.push(name);
        }

        sync_directory(&self.root)?;
        drop(self.staging.take());
        Ok(())
    }
}

impl Drop for OutputWorkspace {
    fn drop(&mut self) {
        drop(self.staging.take());
        if self.remove_root_if_empty {
            let _ = fs::remove_dir(&self.root);
        }
    }
}

#[cfg(any(target_os = "linux", target_vendor = "apple", target_os = "redox"))]
fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        from,
        rustix::fs::CWD,
        to,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(Into::into)
}

#[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "redox")))]
fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(to) {
        Err(error) if error.kind() == ErrorKind::NotFound => fs::rename(from, to),
        Err(error) => Err(error),
        Ok(_) => Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            "destination already exists",
        )),
    }
}

fn sync_directory(path: &Path) -> eyre::Result<()> {
    File::open(path)
        .wrap_err_with(|| format!("failed to open directory {} for syncing", path.display()))?
        .sync_all()
        .wrap_err_with(|| format!("failed to sync directory {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use agglayer_storage::stores::tree_snapshot::{SettledCertificateSnapshot, TokenBalance};
    use agglayer_types::{
        aggchain_proof::{AggchainData, MultisigPayload},
        primitives::Hashable as _,
        Address, Certificate, CertificateId, CertificateIndex, Digest, EpochNumber, Height,
        NetworkId,
    };
    use alloy::providers::{mock::Asserter, ProviderBuilder};
    use pessimistic_proof::unified_bridge::{
        BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex, ImportedBridgeExit,
        L1InfoTreeLeaf, L1InfoTreeLeafInner, LeafType, MerkleProof,
    };

    use super::*;

    fn unique_test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agglayer-storage-export-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn output_collision_check_rejects_files_and_dangling_symlinks() {
        let root = unique_test_path("collisions");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");

        File::create(root.join(LET_DIR)).expect("create collision");
        assert!(ensure_no_output_collision(&root).is_err());
        fs::remove_file(root.join(LET_DIR)).expect("remove collision");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("missing"), root.join(LBT_DIR))
                .expect("create dangling symlink");
            assert!(ensure_no_output_collision(&root).is_err());
            fs::remove_file(root.join(LBT_DIR)).expect("remove dangling symlink");
        }

        File::create(root.join(IBE_DIR)).expect("create IBE collision");
        assert!(ensure_no_output_collision(&root).is_err());

        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn paths_overlap_in_both_directions() {
        let storage = Path::new("/var/lib/agglayer/storage");
        assert!(paths_overlap(storage, storage));
        assert!(paths_overlap(storage, &storage.join("output")));
        assert!(paths_overlap(&storage.join("state"), storage));
        assert!(!paths_overlap(
            storage,
            Path::new("/var/lib/agglayer/export")
        ));
    }

    #[test]
    fn token_format_is_lowercase_and_lossless() {
        let token = TokenInfo {
            origin_network: 42.into(),
            origin_token_address: agglayer_types::Address::from([0xab; 20]),
        };
        assert_eq!(
            format_token(token),
            "42:0xabababababababababababababababababababab"
        );
    }

    fn test_settlement_hash(seed: u8) -> SettlementTxHash {
        SettlementTxHash::new(Digest::from([seed; 32]))
    }

    fn test_merkle_proof(root_seed: u8, sibling_seed: u8) -> MerkleProof {
        MerkleProof::new(
            Digest::from([root_seed; 32]),
            [Digest::from([sibling_seed; 32]); 32],
        )
    }

    fn test_l1_leaf(seed: u8, timestamp: u64) -> L1InfoTreeLeaf {
        L1InfoTreeLeaf {
            l1_info_tree_index: u32::from(seed),
            rer: Digest::from([seed.wrapping_add(1); 32]),
            mer: Digest::from([seed.wrapping_add(2); 32]),
            inner: L1InfoTreeLeafInner {
                global_exit_root: Digest::from([seed.wrapping_add(3); 32]),
                block_hash: Digest::from([seed.wrapping_add(4); 32]),
                timestamp,
            },
        }
    }

    fn test_certificate(
        network_id: NetworkId,
        height: Height,
        bridge_exits: Vec<BridgeExit>,
        imported_bridge_exits: Vec<ImportedBridgeExit>,
    ) -> Certificate {
        Certificate {
            network_id,
            height,
            prev_local_exit_root: Digest::default().into(),
            new_local_exit_root: Digest::default().into(),
            bridge_exits,
            imported_bridge_exits,
            metadata: Default::default(),
            aggchain_data: AggchainData::MultisigOnly {
                multisig: MultisigPayload(Vec::new()),
            },
            custom_chain_data: Vec::new(),
            l1_info_tree_leaf_count: None,
        }
    }

    fn test_snapshot(
        certificate: Certificate,
        settlement_tx_hash: SettlementTxHash,
        first_local_exit_index: u32,
    ) -> SettledCertificateSnapshot {
        SettledCertificateSnapshot {
            certificate,
            certificate_id: CertificateId::new(Digest::from([0x77; 32])),
            epoch_number: EpochNumber::new(10),
            certificate_index: CertificateIndex::new(11),
            settlement_tx_hash,
            first_local_exit_index,
        }
    }

    fn rpc_receipt(
        tx_hash: SettlementTxHash,
        block_hash: B256,
        block_number: u64,
        succeeded: bool,
    ) -> alloy::rpc::types::TransactionReceipt {
        alloy::rpc::types::TransactionReceipt {
            inner: alloy::consensus::ReceiptEnvelope::Eip1559(alloy::consensus::ReceiptWithBloom {
                receipt: alloy::consensus::Receipt {
                    status: succeeded.into(),
                    cumulative_gas_used: 0,
                    logs: vec![],
                },
                logs_bloom: Default::default(),
            }),
            transaction_hash: tx_hash.into(),
            transaction_index: Some(0),
            block_hash: Some(block_hash),
            block_number: Some(block_number),
            gas_used: 0,
            effective_gas_price: 0,
            blob_gas_used: None,
            blob_gas_price: None,
            from: alloy::primitives::Address::ZERO,
            to: None,
            contract_address: None,
        }
    }

    fn rpc_block(number: u64, hash: B256, timestamp: u64) -> alloy::rpc::types::Block {
        let mut block: alloy::rpc::types::Block = Default::default();
        block.header.hash = hash;
        block.header.inner.number = number;
        block.header.inner.timestamp = timestamp;
        block
    }

    fn test_chain_binding(seed: u8) -> L1ChainBinding {
        L1ChainBinding {
            chain_id: u64::from(seed),
            genesis_block_hash: B256::repeat_byte(seed),
        }
    }

    fn test_settlement_info(seed: u8) -> SettlementInfo {
        SettlementInfo::new(
            u64::from(seed),
            B256::repeat_byte(seed),
            1_735_689_600 + u64::from(seed),
        )
        .expect("test timestamp is valid")
    }

    fn rate_limit_error(message: &'static str) -> TransportError {
        TransportError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
            code: 429,
            message: message.into(),
            data: None,
        })
    }

    #[tokio::test]
    async fn settlement_enrichment_requires_a_successful_canonical_receipt() {
        let tx_hash = test_settlement_hash(1);
        let block_hash = B256::repeat_byte(2);
        let block_number = 123;
        let asserter = Asserter::new();
        asserter.push_success(&rpc_receipt(tx_hash, block_hash, block_number, true));
        asserter.push_success(&rpc_block(block_number, block_hash, 1_735_689_600));
        let provider = ProviderBuilder::new().connect_mocked_client(asserter);

        let settlement = resolve_settlement(&provider, tx_hash)
            .await
            .expect("canonical successful settlement should resolve");

        assert_eq!(settlement.block_number, block_number);
        assert_eq!(settlement.block_hash, block_hash);
        assert_eq!(settlement.settled_at, "2025-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn settlement_enrichment_retries_transient_receipt_and_block_failures() {
        let tx_hash = test_settlement_hash(2);
        let block_hash = B256::repeat_byte(3);
        let block_number = 124;
        let asserter = Asserter::new();
        asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload {
            code: 429,
            message: "receipt rate limited".into(),
            data: None,
        });
        asserter.push_success(&rpc_receipt(tx_hash, block_hash, block_number, true));
        asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload {
            code: -32005,
            message: "block rate limited".into(),
            data: None,
        });
        asserter.push_success(&rpc_block(block_number, block_hash, 1_735_689_600));
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let settlement = resolve_settlement(&provider, tx_hash)
            .await
            .expect("transient failures should be retried");

        assert_eq!(settlement.block_number, block_number);
        assert!(asserter.read_q().is_empty());
    }

    #[tokio::test]
    async fn settlement_enrichment_retries_null_receipt_and_block_responses() {
        let tx_hash = test_settlement_hash(29);
        let block_hash = B256::repeat_byte(30);
        let block_number = 125;
        let asserter = Asserter::new();
        asserter.push_success(&Option::<alloy::rpc::types::TransactionReceipt>::None);
        asserter.push_success(&rpc_receipt(tx_hash, block_hash, block_number, true));
        asserter.push_success(&Option::<alloy::rpc::types::Block>::None);
        asserter.push_success(&rpc_block(block_number, block_hash, 1_735_689_600));
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let settlement = resolve_settlement(&provider, tx_hash)
            .await
            .expect("transient null responses should be retried");

        assert_eq!(settlement.block_hash, block_hash);
        assert!(asserter.read_q().is_empty());
    }

    #[tokio::test]
    async fn retry_helper_stops_immediately_on_permanent_rpc_errors() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let test_config = L1RpcRetryConfig {
            max_attempts: 4,
            attempt_timeout: Duration::from_secs(1),
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
        };

        let error = retry_l1_rpc_with_config(L1RpcOperation::ChainId, test_config, {
            let attempts = Arc::clone(&attempts);
            move || {
                attempts.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Err::<u64, _>(TransportError::ErrorResp(
                    alloy::rpc::json_rpc::ErrorPayload::invalid_params(),
                )))
            }
        })
        .await
        .expect_err("invalid parameters are permanent");

        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert!(error.to_string().contains("after 1 attempt"));
    }

    #[tokio::test]
    async fn retry_helper_is_bounded_and_redacts_transient_error_details() {
        const SECRET: &str = "rpc-secret-in-a-retry-error";
        let attempts = Arc::new(AtomicUsize::new(0));
        let test_config = L1RpcRetryConfig {
            max_attempts: 3,
            attempt_timeout: Duration::from_secs(1),
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
        };

        let error = retry_l1_rpc_with_config(L1RpcOperation::ChainId, test_config, {
            let attempts = Arc::clone(&attempts);
            move || {
                attempts.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Err::<u64, _>(rate_limit_error(SECRET)))
            }
        })
        .await
        .expect_err("retry exhaustion should fail");

        assert_eq!(attempts.load(Ordering::SeqCst), 3);
        assert!(!format!("{error:?}").contains(SECRET));
        assert!(error.to_string().contains("after 3 attempt"));
    }

    #[tokio::test]
    async fn settlement_enrichment_rejects_a_noncanonical_receipt() {
        let tx_hash = test_settlement_hash(3);
        let receipt_block_hash = B256::repeat_byte(4);
        let block_number = 456;
        let asserter = Asserter::new();
        asserter.push_success(&rpc_receipt(
            tx_hash,
            receipt_block_hash,
            block_number,
            true,
        ));
        asserter.push_success(&rpc_block(
            block_number,
            B256::repeat_byte(5),
            1_735_689_600,
        ));
        let provider = ProviderBuilder::new().connect_mocked_client(asserter);

        let error = resolve_settlement(&provider, tx_hash)
            .await
            .expect_err("noncanonical settlement should be rejected");

        assert!(error.to_string().contains("is not canonical"));
    }

    #[tokio::test]
    async fn settlement_enrichment_rejects_a_receipt_for_another_transaction() {
        let requested_hash = test_settlement_hash(6);
        let returned_hash = test_settlement_hash(7);
        let asserter = Asserter::new();
        asserter.push_success(&rpc_receipt(returned_hash, B256::repeat_byte(8), 789, true));
        let provider = ProviderBuilder::new().connect_mocked_client(asserter);

        let error = resolve_settlement(&provider, requested_hash)
            .await
            .expect_err("receipt for another transaction should be rejected");

        assert!(error
            .to_string()
            .contains("instead of requested transaction"));
    }

    #[tokio::test]
    async fn settlement_enrichment_does_not_expose_provider_error_details() {
        const SECRET: &str = "rpc-api-key-which-must-not-leak";

        let receipt_asserter = Asserter::new();
        receipt_asserter.push_failure_msg(format!(
            "connection failed for https://user:{SECRET}@l1.invalid?apikey={SECRET}"
        ));
        let receipt_provider = ProviderBuilder::new().connect_mocked_client(receipt_asserter);
        let receipt_error = resolve_settlement(&receipt_provider, test_settlement_hash(9))
            .await
            .expect_err("receipt request should fail");
        assert!(!format!("{receipt_error:?}").contains(SECRET));
        assert!(receipt_error
            .to_string()
            .contains("unable to fetch transaction receipt from the L1 RPC"));

        let tx_hash = test_settlement_hash(10);
        let block_asserter = Asserter::new();
        block_asserter.push_success(&rpc_receipt(tx_hash, B256::repeat_byte(11), 321, true));
        block_asserter.push_failure_msg(format!(
            "connection failed for https://user:{SECRET}@l1.invalid?apikey={SECRET}"
        ));
        let block_provider = ProviderBuilder::new().connect_mocked_client(block_asserter);
        let block_error = resolve_settlement(&block_provider, tx_hash)
            .await
            .expect_err("block request should fail");
        assert!(!format!("{block_error:?}").contains(SECRET));
        assert!(block_error
            .to_string()
            .contains("unable to fetch the settlement block from the L1 RPC"));
    }

    #[tokio::test]
    async fn settlement_enrichment_uses_bounded_concurrency() {
        let hashes = (0..(L1_RPC_CONCURRENCY * 2))
            .map(|seed| test_settlement_hash(seed as u8))
            .collect::<Vec<_>>();
        let active = Arc::new(AtomicUsize::new(0));
        let maximum_active = Arc::new(AtomicUsize::new(0));

        let settlements = resolve_settlement_hashes(hashes.iter().copied(), |tx_hash| {
            let active = Arc::clone(&active);
            let maximum_active = Arc::clone(&maximum_active);
            async move {
                let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_active.fetch_max(now_active, Ordering::SeqCst);
                tokio::task::yield_now().await;
                tokio::time::sleep(Duration::from_millis(5)).await;
                active.fetch_sub(1, Ordering::SeqCst);

                Ok(SettlementInfo {
                    block_number: 1,
                    block_hash: tx_hash.into(),
                    block_timestamp: 1_735_689_600,
                    settled_at: "2025-01-01T00:00:00Z".to_owned(),
                })
            }
        })
        .await
        .expect("settlements should resolve");

        assert_eq!(settlements.len(), hashes.len());
        assert!(maximum_active.load(Ordering::SeqCst) > 1);
        assert!(maximum_active.load(Ordering::SeqCst) <= L1_RPC_CONCURRENCY);
        for hash in hashes {
            assert_eq!(settlements[&hash].block_hash, B256::from(hash));
        }
    }

    #[tokio::test]
    async fn concurrent_settlement_errors_follow_the_input_order() {
        let first_hash = test_settlement_hash(12);
        let second_hash = test_settlement_hash(13);

        let error = resolve_settlement_hashes([first_hash, second_hash], |tx_hash| async move {
            if tx_hash == first_hash {
                tokio::time::sleep(Duration::from_millis(5)).await;
                Err::<SettlementInfo, _>(eyre::eyre!("first resolution failed"))
            } else {
                Err::<SettlementInfo, _>(eyre::eyre!("second resolution failed"))
            }
        })
        .await
        .expect_err("one failed resolution must fail the complete enrichment");

        let error = format!("{error:?}");
        assert!(error.contains(&format_settlement_hash(first_hash)));
        assert!(error.contains("first resolution failed"));
        assert!(!error.contains("second resolution failed"));
    }

    #[tokio::test]
    async fn an_early_resolution_failure_does_not_prevent_later_cache_progress() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(11);
        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");
        let failing_hash = test_settlement_hash(31);
        let later_hash = test_settlement_hash(32);
        let later_info = test_settlement_info(32);

        let error = resolve_settlement_hashes([failing_hash, later_hash], |tx_hash| {
            let later_info = later_info.clone();
            let cache = &cache;
            async move {
                if tx_hash == failing_hash {
                    Err::<SettlementInfo, _>(eyre::eyre!("first resolution failed"))
                } else {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    cache.append(tx_hash, &later_info)?;
                    Ok(later_info)
                }
            }
        })
        .await
        .expect_err("the complete batch still reports its first error");
        assert!(format!("{error:?}").contains("first resolution failed"));
        cache.sync().expect("sync cache after failed batch");
        drop(cache);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("reopen cache");
        assert_eq!(cache.get(later_hash).expect("read cache"), Some(later_info));
    }

    #[test]
    fn settlement_rpc_cache_persists_and_reloads_validated_entries() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(1);
        let tx_hash = test_settlement_hash(20);
        let info = test_settlement_info(20);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");
        assert_eq!(cache.get(tx_hash).expect("read cache"), None);
        cache.append(tx_hash, &info).expect("append cache record");
        cache.sync().expect("sync cache");
        assert_eq!(cache.get(tx_hash).expect("read cache"), Some(info.clone()));
        drop(cache);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("reopen cache");
        assert_eq!(cache.get(tx_hash).expect("read cache"), Some(info));
        let contents = fs::read_to_string(temp.path().join(SETTLEMENT_RPC_CACHE_FILE))
            .expect("read cache file");
        assert_eq!(contents.lines().count(), 2);
        assert!(contents
            .lines()
            .next()
            .expect("header")
            .contains("\"kind\":\"header\""));
    }

    #[test]
    fn settlement_rpc_cache_rejects_chain_mismatches_and_conflicts() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(2);
        let tx_hash = test_settlement_hash(21);
        let info = test_settlement_info(21);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");
        cache.append(tx_hash, &info).expect("append cache record");
        let conflict = test_settlement_info(22);
        let error = cache
            .append(tx_hash, &conflict)
            .expect_err("conflicting data must be rejected");
        assert!(error.to_string().contains("conflicting settlement data"));
        drop(cache);

        let error = SettlementRpcCache::open(temp.path(), test_chain_binding(3))
            .err()
            .expect("different chain must be rejected");
        assert!(error.to_string().contains("different L1 chain"));
    }

    #[test]
    fn settlement_rpc_cache_discards_only_an_incomplete_final_record() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(4);
        let tx_hash = test_settlement_hash(23);
        let info = test_settlement_info(23);
        let cache_path = temp.path().join(SETTLEMENT_RPC_CACHE_FILE);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");
        cache.append(tx_hash, &info).expect("append cache record");
        cache.sync().expect("sync cache");
        drop(cache);
        let complete_len = fs::metadata(&cache_path).expect("cache metadata").len();
        OpenOptions::new()
            .append(true)
            .open(&cache_path)
            .expect("open cache tail")
            .write_all(b"{\"kind\":\"settlement\",\"settlementTxHash\":")
            .expect("write incomplete tail");
        assert!(fs::metadata(&cache_path).expect("cache metadata").len() > complete_len);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("repair cache");
        assert_eq!(cache.get(tx_hash).expect("read cache"), Some(info));
        assert_eq!(
            fs::metadata(&cache_path).expect("cache metadata").len(),
            complete_len
        );
    }

    #[test]
    fn settlement_rpc_cache_rejects_complete_corrupt_records_without_modifying_them() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(5);
        let cache_path = temp.path().join(SETTLEMENT_RPC_CACHE_FILE);
        drop(SettlementRpcCache::open(temp.path(), binding).expect("create cache"));
        OpenOptions::new()
            .append(true)
            .open(&cache_path)
            .expect("open cache")
            .write_all(b"not-json\n")
            .expect("write corrupt record");
        let before = fs::read(&cache_path).expect("read corrupt cache");

        let error = SettlementRpcCache::open(temp.path(), binding)
            .err()
            .expect("complete corruption must fail");
        assert!(error.to_string().contains("invalid JSON at line 2"));
        assert_eq!(fs::read(&cache_path).expect("read cache"), before);
    }

    #[test]
    fn settlement_rpc_cache_does_not_adopt_empty_or_incomplete_existing_files() {
        for (name, contents) in [
            ("empty", b"".as_slice()),
            ("partial", b"{\"kind\":\"header\""),
        ] {
            let temp = tempfile::tempdir().expect("create cache directory");
            let cache_path = temp.path().join(SETTLEMENT_RPC_CACHE_FILE);
            fs::write(&cache_path, contents).expect("write pre-existing file");
            let before = fs::read(&cache_path).expect("read pre-existing file");

            let error = SettlementRpcCache::open(temp.path(), test_chain_binding(12))
                .err()
                .unwrap_or_else(|| panic!("{name} pre-existing file must be rejected"));
            assert!(error.to_string().contains("header"));
            assert_eq!(fs::read(&cache_path).expect("read cache"), before);
        }
    }

    #[test]
    fn settlement_rpc_cache_rejects_oversized_records_without_modifying_them() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(13);
        let cache_path = temp.path().join(SETTLEMENT_RPC_CACHE_FILE);
        drop(SettlementRpcCache::open(temp.path(), binding).expect("create cache"));
        let oversized = vec![b'x'; SETTLEMENT_RPC_CACHE_MAX_RECORD_BYTES + 1];
        OpenOptions::new()
            .append(true)
            .open(&cache_path)
            .expect("open cache")
            .write_all(&oversized)
            .expect("write oversized record");
        let before = fs::read(&cache_path).expect("read cache");

        let error = SettlementRpcCache::open(temp.path(), binding)
            .err()
            .expect("oversized cache record must fail");
        assert!(error.to_string().contains("oversized record"));
        assert_eq!(fs::read(&cache_path).expect("read cache"), before);
    }

    #[cfg(unix)]
    #[test]
    fn settlement_rpc_cache_rejects_symlinks() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let outside = temp.path().join("outside");
        File::create(&outside).expect("create outside file");
        std::os::unix::fs::symlink(&outside, temp.path().join(SETTLEMENT_RPC_CACHE_FILE))
            .expect("create cache symlink");

        let error = SettlementRpcCache::open(temp.path(), test_chain_binding(6))
            .err()
            .expect("cache symlink must fail");
        assert!(error.to_string().contains("must be a regular file"));
        assert_eq!(fs::metadata(outside).expect("outside metadata").len(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn settlement_rpc_cache_rejects_hard_links() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let outside = temp.path().join("outside");
        fs::write(&outside, b"outside").expect("create outside file");
        fs::hard_link(&outside, temp.path().join(SETTLEMENT_RPC_CACHE_FILE))
            .expect("create cache hard link");

        let error = SettlementRpcCache::open(temp.path(), test_chain_binding(14))
            .err()
            .expect("cache hard link must fail");
        assert!(error.to_string().contains("must not be hard-linked"));
        assert_eq!(fs::read(outside).expect("read outside file"), b"outside");
    }

    #[tokio::test]
    async fn settlement_rpc_cache_hits_skip_receipt_and_block_requests() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(7);
        let tx_hash = test_settlement_hash(24);
        let info = test_settlement_info(24);
        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");
        cache.append(tx_hash, &info).expect("append cache record");
        cache.sync().expect("sync cache");

        // The mock has no responses. Any accidental RPC request would make
        // this fail, while a cache hit resolves without touching the provider.
        let provider = ProviderBuilder::new().connect_mocked_client(Asserter::new());
        let settlements = resolve_settlements([tx_hash], &provider, &cache)
            .await
            .expect("cache hit should resolve offline");
        assert_eq!(settlements.get(&tx_hash), Some(&info));
    }

    #[tokio::test]
    async fn settlement_rpc_cache_resumes_after_a_later_rpc_failure() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(9);
        let cached_hash = test_settlement_hash(25);
        let failing_hash = test_settlement_hash(26);
        let block_hash = B256::repeat_byte(27);
        let block_number = 1_234;
        let cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");

        let success_asserter = Asserter::new();
        success_asserter.push_success(&rpc_receipt(cached_hash, block_hash, block_number, true));
        success_asserter.push_success(&rpc_block(block_number, block_hash, 1_735_689_600));
        let success_provider = ProviderBuilder::new().connect_mocked_client(success_asserter);
        resolve_settlements([cached_hash], &success_provider, &cache)
            .await
            .expect("first lookup should populate the cache");

        let failure_asserter = Asserter::new();
        failure_asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload::invalid_params());
        let failure_provider = ProviderBuilder::new().connect_mocked_client(failure_asserter);
        let _error = resolve_settlements([cached_hash, failing_hash], &failure_provider, &cache)
            .await
            .expect_err("later permanent RPC failure should abort enrichment");
        drop(cache);

        let cache = SettlementRpcCache::open(temp.path(), binding).expect("reopen cache");
        let empty_provider = ProviderBuilder::new().connect_mocked_client(Asserter::new());
        let resumed = resolve_settlements([cached_hash], &empty_provider, &cache)
            .await
            .expect("completed lookup should be reusable without RPC calls");
        assert_eq!(resumed[&cached_hash].block_number, block_number);
        assert_eq!(cache.get(failing_hash).expect("read cache"), None);
    }

    #[test]
    fn settlement_rpc_cache_lock_rejects_a_concurrent_exporter() {
        let temp = tempfile::tempdir().expect("create cache directory");
        let binding = test_chain_binding(8);
        let _cache = SettlementRpcCache::open(temp.path(), binding).expect("create cache");

        let error = SettlementRpcCache::open(temp.path(), binding)
            .err()
            .expect("second cache opener must fail");
        assert!(error.to_string().contains("already in use"));
    }

    #[test]
    fn failed_output_workspace_preserves_the_rpc_cache_for_resume() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let storage = temp.path().join("storage");
        fs::create_dir_all(storage.join("state")).expect("create state directory");
        fs::create_dir_all(storage.join("epochs")).expect("create epochs directory");
        let output = temp.path().join("output");

        let workspace = OutputWorkspace::prepare(&storage, &output).expect("prepare output");
        let staging = workspace.staging_path().to_path_buf();
        drop(
            SettlementRpcCache::open(workspace.root_path(), test_chain_binding(10))
                .expect("create cache"),
        );
        drop(workspace);

        assert!(output.join(SETTLEMENT_RPC_CACHE_FILE).is_file());
        assert!(!staging.exists());
        assert!(!output.join(LET_DIR).exists());
        assert!(!output.join(LBT_DIR).exists());
        assert!(!output.join(IBE_DIR).exists());
    }

    #[test]
    fn writes_lossless_deterministic_json() {
        let temp = tempfile::tempdir().expect("create temp directory");
        let settlement_tx_hash = test_settlement_hash(6);
        let token = TokenInfo {
            origin_network: 2.into(),
            origin_token_address: Address::from([0x22; 20]),
        };
        let bridge_exit = BridgeExit {
            leaf_type: LeafType::Transfer,
            token_info: token,
            dest_network: 8.into(),
            dest_address: Address::from([0x88; 20]),
            amount: agglayer_types::U256::MAX,
            metadata: Some(Digest::from([0x44; 32])),
        };
        let leaf_hash = bridge_exit.hash();
        let snapshot = test_snapshot(
            test_certificate(7.into(), Height::new(9), vec![bridge_exit], Vec::new()),
            settlement_tx_hash,
            0,
        );
        let settlements = HashMap::from([(
            settlement_tx_hash,
            SettlementInfo {
                block_number: 12,
                block_hash: B256::repeat_byte(0x12),
                block_timestamp: 1_735_689_600,
                settled_at: "2025-01-01T00:00:00Z".to_owned(),
            },
        )]);

        let let_path = temp.path().join("7-let.json");
        let ibe_path = temp.path().join("7-ibe.json");
        write_network_bridge_history(&let_path, &ibe_path, &settlements, |visitor| {
            visitor(snapshot)
        })
        .expect("write exits");
        let lbt_path = temp.path().join("7-lbt.json");
        write_balances(
            &lbt_path,
            7,
            &[TokenBalance {
                token,
                amount: agglayer_types::U256::MAX,
            }],
        )
        .expect("write balances");

        let exits: serde_json::Value =
            serde_json::from_reader(File::open(let_path).expect("open LET JSON"))
                .expect("parse LET JSON");
        let imported_exits: serde_json::Value =
            serde_json::from_reader(File::open(ibe_path).expect("open IBE JSON"))
                .expect("parse IBE JSON");
        let balances: serde_json::Value =
            serde_json::from_reader(File::open(lbt_path).expect("open LBT JSON"))
                .expect("parse LBT JSON");

        assert_eq!(exits[0]["leafIndex"], 0);
        assert_eq!(exits[0]["leafHash"], format!("{leaf_hash:#x}"));
        assert_eq!(exits[0]["token"], format_token(token));
        assert_eq!(exits[0]["amountToken"], format_token(token));
        assert_eq!(exits[0]["amount"], agglayer_types::U256::MAX.to_string());
        assert_eq!(
            exits[0]["settlementTxHash"],
            format_settlement_hash(settlement_tx_hash)
        );
        assert_eq!(exits[0]["settledAt"], "2025-01-01T00:00:00Z");
        assert_eq!(imported_exits, serde_json::json!([]));
        assert_eq!(
            balances[format_token(token)],
            agglayer_types::U256::MAX.to_string()
        );
    }

    #[test]
    fn message_exit_distinguishes_leaf_token_from_amount_token() {
        let temp = tempfile::tempdir().expect("create temp directory");
        let leaf_token = TokenInfo {
            origin_network: 42.into(),
            origin_token_address: Address::from([0x42; 20]),
        };
        let bridge_exit = BridgeExit {
            leaf_type: LeafType::Message,
            token_info: leaf_token,
            dest_network: 8.into(),
            dest_address: Address::from([0x88; 20]),
            amount: agglayer_types::U256::from(123u64),
            metadata: None,
        };
        let snapshot = test_snapshot(
            test_certificate(7.into(), Height::ZERO, vec![bridge_exit], Vec::new()),
            test_settlement_hash(15),
            0,
        );
        let let_path = temp.path().join("message-let.json");
        let ibe_path = temp.path().join("message-ibe.json");

        write_network_bridge_history(&let_path, &ibe_path, &HashMap::new(), |visitor| {
            visitor(snapshot)
        })
        .expect("write message exit");

        let exits: serde_json::Value =
            serde_json::from_reader(File::open(let_path).expect("open message JSON"))
                .expect("parse message JSON");
        assert_eq!(exits[0]["token"], format_token(leaf_token));
        assert_eq!(
            exits[0]["amountToken"],
            "0:0x0000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn writes_compact_mainnet_and_rollup_imported_exits() {
        let temp = tempfile::tempdir().expect("create temp directory");
        let network_id = NetworkId::new(7);
        let settlement_tx_hash = test_settlement_hash(16);
        let mainnet_bridge_exit = BridgeExit {
            leaf_type: LeafType::Transfer,
            token_info: TokenInfo {
                origin_network: NetworkId::ETH_L1,
                origin_token_address: Address::from([0x21; 20]),
            },
            dest_network: network_id,
            dest_address: Address::from([0x31; 20]),
            amount: agglayer_types::U256::MAX,
            metadata: Some(Digest::from([0x41; 32])),
        };
        let mainnet_l1_leaf = test_l1_leaf(4, 1_735_689_600);
        let mainnet_import = ImportedBridgeExit {
            bridge_exit: mainnet_bridge_exit,
            claim_data: Claim::Mainnet(Box::new(ClaimFromMainnet {
                proof_leaf_mer: test_merkle_proof(1, 2),
                proof_ger_l1root: test_merkle_proof(3, 4),
                l1_leaf: mainnet_l1_leaf.clone(),
            })),
            global_index: GlobalIndex::new(NetworkId::ETH_L1, 41),
        };

        let message_token = TokenInfo {
            origin_network: NetworkId::new(9),
            origin_token_address: Address::from([0x29; 20]),
        };
        let rollup_bridge_exit = BridgeExit {
            leaf_type: LeafType::Message,
            token_info: message_token,
            dest_network: network_id,
            dest_address: Address::from([0x39; 20]),
            amount: agglayer_types::U256::from(123u64),
            metadata: None,
        };
        let rollup_l1_leaf = test_l1_leaf(5, u64::MAX);
        let rollup_import = ImportedBridgeExit {
            bridge_exit: rollup_bridge_exit,
            claim_data: Claim::Rollup(Box::new(ClaimFromRollup {
                proof_leaf_ler: test_merkle_proof(5, 6),
                proof_ler_rer: test_merkle_proof(7, 8),
                proof_ger_l1root: test_merkle_proof(9, 10),
                l1_leaf: rollup_l1_leaf.clone(),
            })),
            // Global indexes encode the zero-based rollup index. This value
            // resolves to source network 2 and rollup index 1.
            global_index: GlobalIndex::new(NetworkId::new(1), 42),
        };

        let snapshot = test_snapshot(
            test_certificate(
                network_id,
                Height::new(12),
                Vec::new(),
                vec![mainnet_import.clone(), rollup_import.clone()],
            ),
            settlement_tx_hash,
            0,
        );
        let settlements = HashMap::from([(
            settlement_tx_hash,
            SettlementInfo {
                block_number: 99,
                block_hash: B256::repeat_byte(0x99),
                block_timestamp: 1_735_787_045,
                settled_at: "2025-01-02T03:04:05Z".to_owned(),
            },
        )]);
        let let_path = temp.path().join("7-let.json");
        let ibe_path = temp.path().join("7-ibe.json");

        write_network_bridge_history(&let_path, &ibe_path, &settlements, |visitor| {
            visitor(snapshot)
        })
        .expect("write imported exits");

        let exits: serde_json::Value =
            serde_json::from_reader(File::open(let_path).expect("open LET JSON"))
                .expect("parse LET JSON");
        let imported: serde_json::Value =
            serde_json::from_reader(File::open(ibe_path).expect("open IBE JSON"))
                .expect("parse IBE JSON");
        assert_eq!(exits, serde_json::json!([]));
        assert_eq!(imported.as_array().map(Vec::len), Some(2));

        let mainnet = &imported[0];
        let mainnet_global_index = mainnet_import.global_index.into_u256();
        assert_eq!(mainnet["importedExitIndex"], 0);
        assert_eq!(
            mainnet["importedBridgeExitHash"],
            format!("{:#x}", mainnet_import.hash())
        );
        assert_eq!(
            mainnet["bridgeExitHash"],
            format!("{:#x}", mainnet_import.bridge_exit.hash())
        );
        assert_eq!(mainnet["globalIndex"], mainnet_global_index.to_string());
        assert_eq!(
            mainnet["globalIndexHex"],
            format!("{mainnet_global_index:#x}")
        );
        assert_eq!(mainnet["sourceNetwork"], 0);
        assert_eq!(mainnet["sourceLeafIndex"], 41);
        assert_eq!(mainnet["mainnet"], true);
        assert!(mainnet["rollupIndex"].is_null());
        assert_eq!(mainnet["claimType"], "mainnet");
        assert_eq!(
            mainnet["l1InfoRoot"],
            format!("{:#x}", Digest::from([3; 32]))
        );
        assert_eq!(mainnet["l1InfoTreeLeafIndex"], 4);
        assert_eq!(
            mainnet["l1InfoTreeLeafHash"],
            format!("{:#x}", mainnet_l1_leaf.hash())
        );
        assert_eq!(
            mainnet["globalExitRoot"],
            format!("{:#x}", mainnet_l1_leaf.inner.global_exit_root)
        );
        assert_eq!(
            mainnet["mainnetExitRoot"],
            format!("{:#x}", mainnet_l1_leaf.mer)
        );
        assert_eq!(
            mainnet["rollupExitRoot"],
            format!("{:#x}", mainnet_l1_leaf.rer)
        );
        assert_eq!(
            mainnet["l1InfoTreeBlockHash"],
            format!("{:#x}", mainnet_l1_leaf.inner.block_hash)
        );
        assert_eq!(mainnet["l1InfoTreeTimestamp"], "1735689600");
        assert_eq!(mainnet["l1InfoTreeAt"], "2025-01-01T00:00:00Z");
        assert_eq!(mainnet["certificateHeight"], 12);
        assert_eq!(mainnet["settlementBlockNumber"], 99);
        assert_eq!(mainnet["settledAt"], "2025-01-02T03:04:05Z");
        assert!(mainnet.get("claimData").is_none());

        let rollup = &imported[1];
        assert_eq!(rollup["importedExitIndex"], 1);
        assert_eq!(rollup["claimType"], "rollup");
        assert_eq!(rollup["sourceNetwork"], 2);
        assert_eq!(rollup["sourceLeafIndex"], 42);
        assert_eq!(rollup["mainnet"], false);
        assert_eq!(rollup["rollupIndex"], 1);
        assert_eq!(rollup["leafType"], "message");
        assert_eq!(rollup["token"], format_token(message_token));
        assert_eq!(
            rollup["amountToken"],
            "0:0x0000000000000000000000000000000000000000"
        );
        assert_eq!(rollup["l1InfoTreeTimestamp"], u64::MAX.to_string());
        assert!(rollup["l1InfoTreeAt"].is_null());
    }

    #[test]
    fn publishes_all_fresh_directories_without_touching_siblings() {
        let temp = tempfile::tempdir().expect("create temp directory");
        let storage = temp.path().join("storage");
        fs::create_dir_all(storage.join("state")).expect("create state directory");
        fs::create_dir_all(storage.join("epochs")).expect("create epochs directory");
        let output = temp.path().join("output");
        fs::create_dir(&output).expect("create output directory");
        fs::write(output.join("keep.txt"), "keep").expect("write sibling file");

        let workspace = OutputWorkspace::prepare(&storage, &output).expect("prepare output");
        fs::create_dir(workspace.staging_path().join(LET_DIR)).expect("create staged LET");
        fs::create_dir(workspace.staging_path().join(LBT_DIR)).expect("create staged LBT");
        fs::create_dir(workspace.staging_path().join(IBE_DIR)).expect("create staged IBE");
        workspace.publish().expect("publish output");

        assert!(output.join(LET_DIR).is_dir());
        assert!(output.join(LBT_DIR).is_dir());
        assert!(output.join(IBE_DIR).is_dir());
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).expect("read sibling file"),
            "keep"
        );
    }
}
