//! Settlement-day pricing enrichment for exported tree snapshots.
//!
//! This module deliberately treats certificate settlement as the pricing
//! observation, not as the source-chain exit time. The export does not contain
//! the latter. LET rows use their certificate settlement and IBE rows use the
//! settlement of the certificate that claimed the imported exit.

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    fs::{self, File, OpenOptions},
    future::Future,
    io::{
        BufRead as _, BufReader, BufWriter, ErrorKind, Read as _, Seek as _, SeekFrom, Write as _,
    },
    path::{Component, Path, PathBuf},
    pin::Pin,
    sync::Mutex,
    time::Duration,
};

use agglayer_types::U256;
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use eyre::{bail, ensure, Context as _, ContextCompat as _};
use futures::{stream, StreamExt as _};
use num_bigint::BigUint;
use reqwest::{Client, StatusCode};
use serde::{
    de::{self, Deserializer as _, SeqAccess, Visitor},
    ser::SerializeSeq as _,
    Deserialize, Serialize, Serializer as _,
};
use serde_json::{Map, Number, Value};
use tempfile::{Builder as TempDirBuilder, TempDir};

use super::{
    finish_output_file, paths_overlap, rename_noreplace, resolve_output_path, sync_directory,
};

const LET_DIR: &str = "let";
const IBE_DIR: &str = "ibe";
const LBT_DIR: &str = "lbt";
const OUTPUT_NAMES: [&str; 4] = [LET_DIR, IBE_DIR, LBT_DIR, PRICING_REPORT_FILE];
const PRICING_FIELD: &str = "settlementDayPricing";
const PRICING_REPORT_FILE: &str = "pricing-report.json";
const PRICE_CACHE_FILE: &str = ".agglayer-defillama-price-cache-v1.jsonl";
const PRICE_CACHE_VERSION: u8 = 1;
const PRICE_CACHE_MAX_RECORD_BYTES: usize = 16 * 1024;
const PROVIDER: &str = "defillama";
const QUOTE_CURRENCY: &str = "USD";
const DEFI_LLAMA_BASE_URL: &str = "https://coins.llama.fi";
const REQUEST_TIME_UTC: &str = "12:00:00";
const SEARCH_WIDTH: &str = "12h";
const SEARCH_WIDTH_SECONDS: i64 = 12 * 60 * 60;
const REQUEST_CONCURRENCY: usize = 2;
const REQUEST_MAX_COINS: usize = 50;
const REQUEST_MAX_TARGET_BYTES: usize = 3_500;
const REQUEST_MAX_ATTEMPTS: u32 = 5;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const REQUEST_INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const REQUEST_MAX_BACKOFF: Duration = Duration::from_secs(8);
const RESPONSE_MAX_BYTES: usize = 4 * 1024 * 1024;
const DECIMAL_TEXT_MAX_BYTES: usize = 256;
const DECIMAL_EXPONENT_ABS_MAX: i32 = 1_000;

const U256_MAX_DECIMAL: &str =
    "115792089237316195423570985008687907853269984665640564039457584007913129639935";

const NETWORK_MAPPINGS: &[(u32, &str)] = &[
    (0, "ethereum"),
    (1, "polygon_zkevm"),
    (2, "astrzk"),
    (3, "xlayer"),
    (7, "lumia"),
    (13, "ternoa"),
    (14, "z"),
    (16, "pentagonchain"),
    (20, "katana"),
];

#[derive(Clone, Copy)]
struct VaultRedirect {
    vault: &'static str,
    underlying: &'static str,
    decimals: u8,
}

const VAULT_REDIRECTS: &[VaultRedirect] = &[
    VaultRedirect {
        vault: "0x2dc70fb75b88d2eb4715bc06e1595e6d97c34dff",
        underlying: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        decimals: 18,
    },
    VaultRedirect {
        vault: "0x53e82abbb12638f09d9e624578ccb666217a765e",
        underlying: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        decimals: 6,
    },
    VaultRedirect {
        vault: "0x6d4f9f9f8f0155509ecd6ac6c544ff27999845cc",
        underlying: "0xdac17f958d2ee523a2206206994597c13d831ec7",
        decimals: 6,
    },
    VaultRedirect {
        vault: "0x2c24b57e2ccd1f273045af6a5f632504c432374f",
        underlying: "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599",
        decimals: 8,
    },
    VaultRedirect {
        vault: "0x3dd459de96f9c28e3a343b831cbdc2b93c8c4855",
        underlying: "0xdc035d45d973e3ec169d2276ddab16f1e407384f",
        decimals: 18,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HistoryKind {
    Let,
    Ibe,
}

impl HistoryKind {
    const fn directory(self) -> &'static str {
        match self {
            Self::Let => LET_DIR,
            Self::Ibe => IBE_DIR,
        }
    }

    const fn timestamp_basis(self) -> &'static str {
        match self {
            Self::Let => "certificateSettlement",
            Self::Ibe => "claimingCertificateSettlement",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ProviderLookup {
    provider_coin_id: String,
    pricing_date: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LogicalLookup {
    amount_token: String,
    pricing_date: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TokenIdentity {
    canonical: String,
    origin_network: u32,
    address: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderIdentity {
    provider_coin_id: String,
    method: &'static str,
    decimals_override: Option<(u8, &'static str)>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedPrice {
    status: CachedPriceStatus,
    symbol: Option<String>,
    decimals: Option<u8>,
    unit_price_usd: Option<String>,
    price_timestamp: Option<u64>,
    confidence: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CachedPriceStatus {
    Available,
    Missing,
}

#[derive(Clone, Debug)]
struct ProviderPrice {
    symbol: Option<String>,
    decimals: Option<u8>,
    unit_price_usd: String,
    price_timestamp: u64,
    confidence: Option<String>,
}

#[derive(Clone, Debug)]
struct RequestChunk {
    pricing_date: String,
    requested_timestamp: u64,
    provider_coin_ids: Vec<String>,
}

type FetchFuture<'a> =
    Pin<Box<dyn Future<Output = eyre::Result<BTreeMap<String, ProviderPrice>>> + Send + 'a>>;

trait PriceFetcher: Sync {
    fn fetch<'a>(&'a self, chunk: &'a RequestChunk) -> FetchFuture<'a>;
}

struct DefiLlamaFetcher {
    client: Client,
    base_url: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum PricingStatus {
    Priced,
    TimestampUnavailable,
    UnsupportedNetwork,
    PriceUnavailable,
    DecimalsUnavailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettlementDayPricing {
    status: PricingStatus,
    provider: &'static str,
    provider_coin_id: Option<String>,
    provider_coin_method: Option<&'static str>,
    quote_currency: &'static str,
    pricing_date: Option<String>,
    timestamp_basis: &'static str,
    requested_at: Option<String>,
    price_timestamp: Option<u64>,
    price_at: Option<String>,
    provider_confidence: Option<String>,
    provider_symbol: Option<String>,
    decimals: Option<u8>,
    decimals_source: Option<&'static str>,
    unit_price_usd: Option<String>,
    normalized_amount: Option<String>,
    value_usd: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum PriceCacheRecord {
    Header {
        version: u8,
        provider: String,
        quote_currency: String,
        requested_time_utc: String,
        search_width: String,
        same_utc_day_required: bool,
        network_mappings: Vec<CacheNetworkMapping>,
        vault_redirects: Vec<CacheVaultRedirect>,
    },
    Price {
        provider_coin_id: String,
        pricing_date: String,
        #[serde(flatten)]
        price: CachedPrice,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheNetworkMapping {
    origin_network: u32,
    provider_namespace: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheVaultRedirect {
    origin_network: u32,
    vault: String,
    underlying: String,
    decimals: u8,
    method: String,
}

struct PriceCache {
    path: PathBuf,
    state: Mutex<PriceCacheState>,
}

struct PriceCacheState {
    file: File,
    entries: HashMap<ProviderLookup, CachedPrice>,
    poisoned: bool,
}

#[derive(Default)]
struct Inventory {
    amount_tokens: BTreeSet<String>,
    logical_lookups: BTreeSet<LogicalLookup>,
    provider_lookups: BTreeSet<ProviderLookup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NetworkFile {
    network_id: u32,
    file_name: String,
    path: PathBuf,
}

struct InputExport {
    root: PathBuf,
    let_files: Vec<NetworkFile>,
    ibe_files: Vec<NetworkFile>,
    lbt_files: Vec<NetworkFile>,
}

struct PricingOutputWorkspace {
    root: PathBuf,
    staging: Option<TempDir>,
    remove_root_if_empty: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusCounts {
    priced: u64,
    timestamp_unavailable: u64,
    unsupported_network: u64,
    price_unavailable: u64,
    decimals_unavailable: u64,
}

impl StatusCounts {
    fn increment(&mut self, status: PricingStatus) {
        match status {
            PricingStatus::Priced => self.priced += 1,
            PricingStatus::TimestampUnavailable => self.timestamp_unavailable += 1,
            PricingStatus::UnsupportedNetwork => self.unsupported_network += 1,
            PricingStatus::PriceUnavailable => self.price_unavailable += 1,
            PricingStatus::DecimalsUnavailable => self.decimals_unavailable += 1,
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct KindCoverage {
    rows: u64,
    by_status: StatusCounts,
    by_network: BTreeMap<u32, StatusCounts>,
}

impl KindCoverage {
    fn record(&mut self, network_id: u32, status: PricingStatus) {
        self.rows += 1;
        self.by_status.increment(status);
        self.by_network
            .entry(network_id)
            .or_default()
            .increment(status);
    }
}

#[derive(Default)]
struct Coverage {
    let_rows: KindCoverage,
    ibe_rows: KindCoverage,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct LookupReport {
    cache_hits: u64,
    refreshed_misses: u64,
    requested: u64,
    available: u64,
    missing: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DailyPriceConvention {
    requested_time_utc: &'static str,
    search_width: &'static str,
    same_utc_day_required: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TimestampBases {
    r#let: &'static str,
    ibe: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportCoverage {
    r#let: KindCoverage,
    ibe: KindCoverage,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PricingReport {
    schema_version: u8,
    provider: &'static str,
    quote_currency: &'static str,
    daily_price_convention: DailyPriceConvention,
    timestamp_bases: TimestampBases,
    lbt: &'static str,
    let_and_ibe_aggregated: bool,
    unique_amount_tokens: usize,
    unique_token_days: usize,
    unique_provider_lookups: usize,
    lookups: LookupReport,
    coverage: ReportCoverage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExactDecimal {
    coefficient: BigUint,
    scale: i32,
}

impl ExactDecimal {
    fn parse_non_negative(value: &str, label: &str) -> eyre::Result<Self> {
        ensure!(
            !value.is_empty() && value.len() <= DECIMAL_TEXT_MAX_BYTES,
            "{label} has an invalid encoded length"
        );
        ensure!(!value.starts_with('-'), "{label} must not be negative");

        let value = value.strip_prefix('+').unwrap_or(value);
        let (mantissa, exponent) = match value.find(['e', 'E']) {
            Some(index) => {
                ensure!(
                    !value[index + 1..].contains(['e', 'E']),
                    "{label} contains multiple exponents"
                );
                let exponent = value[index + 1..]
                    .parse::<i32>()
                    .wrap_err_with(|| format!("{label} contains an invalid exponent"))?;
                ensure!(
                    exponent.unsigned_abs() <= DECIMAL_EXPONENT_ABS_MAX as u32,
                    "{label} exponent is outside the supported range"
                );
                (&value[..index], exponent)
            }
            None => (value, 0),
        };

        let (whole, fractional) = match mantissa.split_once('.') {
            Some((whole, fractional)) => {
                ensure!(
                    !fractional.contains('.'),
                    "{label} contains multiple decimal points"
                );
                (whole, fractional)
            }
            None => (mantissa, ""),
        };
        ensure!(
            !whole.is_empty() || !fractional.is_empty(),
            "{label} contains no digits"
        );
        ensure!(
            whole
                .bytes()
                .chain(fractional.bytes())
                .all(|byte| byte.is_ascii_digit()),
            "{label} contains a non-decimal digit"
        );

        let mut digits = format!("{whole}{fractional}");
        let first_nonzero = digits.find(|character| character != '0');
        if let Some(index) = first_nonzero {
            digits.drain(..index);
        } else {
            return Ok(Self {
                coefficient: BigUint::from(0u8),
                scale: 0,
            });
        }
        let mut scale = i32::try_from(fractional.len())
            .wrap_err("decimal fractional precision is too large")?
            .checked_sub(exponent)
            .context("decimal scale overflow")?;
        while scale > 0 && digits.ends_with('0') {
            digits.pop();
            scale -= 1;
        }
        let coefficient = BigUint::parse_bytes(digits.as_bytes(), 10)
            .context("failed to parse exact decimal coefficient")?;
        Ok(Self { coefficient, scale })
    }

    fn canonical(&self) -> eyre::Result<String> {
        format_decimal(&self.coefficient, self.scale)
    }

    fn is_zero(&self) -> bool {
        self.coefficient == BigUint::from(0u8)
    }

    fn is_at_most_one(&self) -> bool {
        if self.is_zero() {
            return true;
        }
        match self.scale.cmp(&0) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => self.coefficient <= BigUint::from(1u8),
            std::cmp::Ordering::Greater => {
                let Ok(scale) = u32::try_from(self.scale) else {
                    return false;
                };
                self.coefficient <= BigUint::from(10u8).pow(scale)
            }
        }
    }
}

fn format_decimal(coefficient: &BigUint, mut scale: i32) -> eyre::Result<String> {
    if coefficient == &BigUint::from(0u8) {
        return Ok("0".to_owned());
    }
    let mut digits = coefficient.to_str_radix(10);
    while scale > 0 && digits.ends_with('0') {
        digits.pop();
        scale -= 1;
    }
    if scale <= 0 {
        let zeros = usize::try_from(scale.checked_neg().context("decimal scale overflow")?)
            .wrap_err("decimal scale is outside the supported range")?;
        ensure!(
            zeros <= DECIMAL_EXPONENT_ABS_MAX as usize,
            "decimal scale is outside the supported range"
        );
        digits.extend(std::iter::repeat_n('0', zeros));
        return Ok(digits);
    }

    let scale = usize::try_from(scale).wrap_err("decimal scale is outside the supported range")?;
    ensure!(
        scale <= DECIMAL_EXPONENT_ABS_MAX as usize + 255,
        "decimal scale is outside the supported range"
    );
    if digits.len() <= scale {
        let mut rendered = String::with_capacity(scale + 2);
        rendered.push_str("0.");
        rendered.extend(std::iter::repeat_n('0', scale - digits.len()));
        rendered.push_str(&digits);
        Ok(rendered)
    } else {
        digits.insert(digits.len() - scale, '.');
        Ok(digits)
    }
}

fn normalize_amount(amount: &str, decimals: u8) -> eyre::Result<String> {
    let coefficient = parse_amount(amount)?;
    format_decimal(&coefficient, i32::from(decimals))
}

fn calculate_value_usd(amount: &str, decimals: u8, unit_price_usd: &str) -> eyre::Result<String> {
    let amount = parse_amount(amount)?;
    let price = ExactDecimal::parse_non_negative(unit_price_usd, "DefiLlama price")?;
    let coefficient = amount * price.coefficient;
    let scale = i32::from(decimals)
        .checked_add(price.scale)
        .context("USD value scale overflow")?;
    format_decimal(&coefficient, scale)
}

fn parse_amount(amount: &str) -> eyre::Result<BigUint> {
    ensure!(
        !amount.is_empty() && amount.bytes().all(|byte| byte.is_ascii_digit()),
        "amount must be an unsigned decimal string"
    );
    ensure!(
        amount.len() < U256_MAX_DECIMAL.len()
            || (amount.len() == U256_MAX_DECIMAL.len() && amount <= U256_MAX_DECIMAL),
        "amount is outside the U256 range"
    );
    U256::from_str_radix(amount, 10).wrap_err("amount is outside the U256 range")?;
    BigUint::parse_bytes(amount.as_bytes(), 10).context("failed to parse amount")
}

fn parse_token_identity(value: &str) -> eyre::Result<TokenIdentity> {
    let (network, address) = value
        .split_once(':')
        .context("amountToken must use '<network>:<address>' format")?;
    ensure!(
        !network.is_empty()
            && network.bytes().all(|byte| byte.is_ascii_digit())
            && (network == "0" || !network.starts_with('0')),
        "amountToken network must be canonical decimal"
    );
    let origin_network = network
        .parse::<u32>()
        .wrap_err("amountToken network is outside the u32 range")?;
    ensure!(
        address.len() == 42
            && address.starts_with("0x")
            && address[2..]
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "amountToken address must be lowercase 20-byte hex"
    );
    Ok(TokenIdentity {
        canonical: value.to_owned(),
        origin_network,
        address: address.to_owned(),
    })
}

fn provider_identity(token: &TokenIdentity) -> Option<ProviderIdentity> {
    if token.origin_network == 0 {
        if token.address == "0x0000000000000000000000000000000000000000" {
            return Some(ProviderIdentity {
                provider_coin_id: "coingecko:ethereum".to_owned(),
                method: "direct",
                decimals_override: Some((18, "protocol")),
            });
        }
        if let Some(redirect) = VAULT_REDIRECTS
            .iter()
            .find(|redirect| redirect.vault == token.address)
        {
            return Some(ProviderIdentity {
                provider_coin_id: format!("ethereum:{}", redirect.underlying),
                method: "vaultBridge1To1",
                decimals_override: Some((redirect.decimals, "vaultBridge1To1")),
            });
        }
    }

    let namespace = NETWORK_MAPPINGS.iter().find_map(|(network, namespace)| {
        (*network == token.origin_network).then_some(*namespace)
    })?;
    Some(ProviderIdentity {
        provider_coin_id: format!("{namespace}:{}", token.address),
        method: "direct",
        decimals_override: None,
    })
}

fn parse_settlement_date(value: &Value) -> eyre::Result<Option<String>> {
    if value.is_null() {
        return Ok(None);
    }
    let value = value
        .as_str()
        .context("settledAt must be an RFC3339 string or null")?;
    let timestamp = DateTime::parse_from_rfc3339(value)
        .wrap_err("settledAt must be a valid RFC3339 timestamp")?
        .with_timezone(&Utc);
    Ok(Some(timestamp.format("%Y-%m-%d").to_string()))
}

fn requested_timestamp(pricing_date: &str) -> eyre::Result<u64> {
    let date = NaiveDate::parse_from_str(pricing_date, "%Y-%m-%d")
        .wrap_err("pricing date is not YYYY-MM-DD")?;
    let timestamp = date
        .and_hms_opt(12, 0, 0)
        .context("unable to construct UTC-noon pricing timestamp")?
        .and_utc()
        .timestamp();
    u64::try_from(timestamp).wrap_err("pricing timestamp precedes the Unix epoch")
}

fn format_timestamp(timestamp: u64) -> eyre::Result<String> {
    let timestamp = i64::try_from(timestamp).wrap_err("timestamp exceeds the i64 range")?;
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true))
        .context("timestamp is outside the supported UTC range")
}

fn timestamp_date(timestamp: u64) -> eyre::Result<String> {
    Ok(format_timestamp(timestamp)?[..10].to_owned())
}

fn expected_cache_header() -> PriceCacheRecord {
    PriceCacheRecord::Header {
        version: PRICE_CACHE_VERSION,
        provider: PROVIDER.to_owned(),
        quote_currency: QUOTE_CURRENCY.to_owned(),
        requested_time_utc: REQUEST_TIME_UTC.to_owned(),
        search_width: SEARCH_WIDTH.to_owned(),
        same_utc_day_required: true,
        network_mappings: NETWORK_MAPPINGS
            .iter()
            .map(|(origin_network, namespace)| CacheNetworkMapping {
                origin_network: *origin_network,
                provider_namespace: (*namespace).to_owned(),
            })
            .collect(),
        vault_redirects: VAULT_REDIRECTS
            .iter()
            .map(|redirect| CacheVaultRedirect {
                origin_network: 0,
                vault: redirect.vault.to_owned(),
                underlying: redirect.underlying.to_owned(),
                decimals: redirect.decimals,
                method: "vaultBridge1To1".to_owned(),
            })
            .collect(),
    }
}

impl PriceCache {
    fn open(output_root: &Path) -> eyre::Result<Self> {
        let path = output_root.join(PRICE_CACHE_FILE);
        let exists = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink() && metadata.is_file(),
                    "price cache {} must be a regular file",
                    path.display()
                );
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => {
                return Err(error)
                    .wrap_err_with(|| format!("failed to inspect price cache {}", path.display()));
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
            .wrap_err_with(|| format!("failed to open price cache {}", path.display()))?;
        let metadata = file
            .metadata()
            .wrap_err("failed to inspect opened price cache")?;
        ensure!(metadata.is_file(), "price cache must be a regular file");
        #[cfg(unix)]
        ensure!(
            metadata.nlink() == 1,
            "price cache {} must not be hard-linked",
            path.display()
        );
        file.try_lock().wrap_err_with(|| {
            format!(
                "price cache {} is already in use by another enricher",
                path.display()
            )
        })?;

        let entries = if exists {
            Self::load(&mut file, &path)?
        } else {
            write_cache_record(&mut file, &expected_cache_header(), &path)?;
            file.flush()
                .wrap_err("failed to flush price cache header")?;
            file.sync_all()
                .wrap_err("failed to sync price cache header")?;
            sync_directory(output_root)?;
            HashMap::new()
        };
        Ok(Self {
            path,
            state: Mutex::new(PriceCacheState {
                file,
                entries,
                poisoned: false,
            }),
        })
    }

    fn load(file: &mut File, path: &Path) -> eyre::Result<HashMap<ProviderLookup, CachedPrice>> {
        file.seek(SeekFrom::Start(0))
            .wrap_err("failed to rewind price cache")?;
        let mut reader = BufReader::new(&mut *file);
        let mut buffer = Vec::new();
        let mut offset = 0u64;
        let mut line_number = 0usize;
        let mut entries = HashMap::new();
        let mut saw_header = false;
        loop {
            buffer.clear();
            let record_start = offset;
            let read = (&mut reader)
                .take((PRICE_CACHE_MAX_RECORD_BYTES + 1) as u64)
                .read_until(b'\n', &mut buffer)
                .wrap_err_with(|| format!("failed to read price cache {}", path.display()))?;
            if read == 0 {
                break;
            }
            offset = offset
                .checked_add(u64::try_from(read).wrap_err("price cache offset overflow")?)
                .context("price cache offset overflow")?;
            line_number += 1;
            ensure!(
                buffer.len() <= PRICE_CACHE_MAX_RECORD_BYTES,
                "price cache {} record {line_number} exceeds the size limit",
                path.display()
            );
            if !buffer.ends_with(b"\n") {
                ensure!(
                    saw_header,
                    "price cache {} has no complete header",
                    path.display()
                );
                drop(reader);
                file.set_len(record_start).wrap_err_with(|| {
                    format!(
                        "failed to truncate partial price cache record in {}",
                        path.display()
                    )
                })?;
                file.sync_data()
                    .wrap_err("failed to sync truncated price cache")?;
                file.seek(SeekFrom::End(0))
                    .wrap_err("failed to seek to end of price cache")?;
                return Ok(entries);
            }

            let record: PriceCacheRecord = serde_json::from_slice(&buffer[..buffer.len() - 1])
                .wrap_err_with(|| {
                    format!(
                        "price cache {} contains invalid JSON at record {line_number}",
                        path.display()
                    )
                })?;
            match record {
                header @ PriceCacheRecord::Header { .. } if line_number == 1 => {
                    ensure!(
                        header == expected_cache_header(),
                        "price cache {} uses incompatible pricing methodology",
                        path.display()
                    );
                    saw_header = true;
                }
                PriceCacheRecord::Header { .. } => bail!(
                    "price cache {} contains a second header at record {line_number}",
                    path.display()
                ),
                PriceCacheRecord::Price {
                    provider_coin_id,
                    pricing_date,
                    price,
                } => {
                    ensure!(saw_header, "price cache must begin with its header");
                    validate_cached_price(&price)?;
                    let key = ProviderLookup {
                        provider_coin_id,
                        pricing_date,
                    };
                    if let Some(previous) = entries.get(&key) {
                        ensure!(
                            previous == &price
                                || (previous.status == CachedPriceStatus::Missing
                                    && price.status == CachedPriceStatus::Available),
                            "price cache {} contains conflicting records for {} on {}",
                            path.display(),
                            key.provider_coin_id,
                            key.pricing_date
                        );
                    }
                    entries.insert(key, price);
                }
            }
        }
        ensure!(saw_header, "price cache {} is empty", path.display());
        drop(reader);
        file.seek(SeekFrom::End(0))
            .wrap_err("failed to seek to end of price cache")?;
        Ok(entries)
    }

    fn get(&self, key: &ProviderLookup) -> eyre::Result<Option<CachedPrice>> {
        let state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("price cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "price cache is unavailable after a write failure"
        );
        Ok(state.entries.get(key).cloned())
    }

    fn append(&self, key: ProviderLookup, price: CachedPrice) -> eyre::Result<()> {
        validate_cached_price(&price)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("price cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "price cache is unavailable after a write failure"
        );
        if let Some(previous) = state.entries.get(&key) {
            if previous == &price {
                return Ok(());
            }
            ensure!(
                previous.status == CachedPriceStatus::Missing
                    && price.status == CachedPriceStatus::Available,
                "attempted to append a conflicting cached price for {} on {}",
                key.provider_coin_id,
                key.pricing_date
            );
        }
        let committed_len = state
            .file
            .metadata()
            .wrap_err("failed to inspect price cache before append")?
            .len();
        let record = PriceCacheRecord::Price {
            provider_coin_id: key.provider_coin_id.clone(),
            pricing_date: key.pricing_date.clone(),
            price: price.clone(),
        };
        if let Err(write_error) = write_cache_record(&mut state.file, &record, &self.path) {
            state.poisoned = true;
            let rollback = state
                .file
                .set_len(committed_len)
                .and_then(|()| state.file.sync_data());
            return match rollback {
                Ok(()) => Err(write_error),
                Err(rollback_error) => Err(write_error).wrap_err(format!(
                    "additionally failed to roll back partial price cache record: {rollback_error}"
                )),
            };
        }
        state
            .file
            .flush()
            .wrap_err("failed to flush price cache record")?;
        state.entries.insert(key, price);
        Ok(())
    }

    fn sync(&self) -> eyre::Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("price cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "price cache is unavailable after a write failure"
        );
        state.file.flush().wrap_err("failed to flush price cache")?;
        state
            .file
            .sync_data()
            .wrap_err("failed to sync price cache")
    }

    fn snapshot(&self) -> eyre::Result<HashMap<ProviderLookup, CachedPrice>> {
        let state = self
            .state
            .lock()
            .map_err(|_| eyre::eyre!("price cache lock is poisoned"))?;
        ensure!(
            !state.poisoned,
            "price cache is unavailable after a write failure"
        );
        Ok(state.entries.clone())
    }
}

fn copy_seed_price_cache(output_root: &Path, seed_path: Option<&Path>) -> eyre::Result<()> {
    let Some(seed_path) = seed_path else {
        return Ok(());
    };
    let metadata = fs::symlink_metadata(seed_path)
        .wrap_err_with(|| format!("failed to inspect seed price cache {}", seed_path.display()))?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "seed price cache {} must be a real file",
        seed_path.display()
    );

    let mut source_options = OpenOptions::new();
    source_options.read(true);
    #[cfg(unix)]
    source_options.custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32);
    let mut source = source_options
        .open(seed_path)
        .wrap_err_with(|| format!("failed to open seed price cache {}", seed_path.display()))?;
    ensure!(
        source
            .metadata()
            .wrap_err("failed to inspect opened seed price cache")?
            .is_file(),
        "seed price cache must be a regular file"
    );

    let destination = output_root.join(PRICE_CACHE_FILE);
    let mut output_options = OpenOptions::new();
    output_options.write(true).create_new(true);
    #[cfg(unix)]
    output_options.custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32);
    let mut output = output_options.open(&destination).wrap_err_with(|| {
        format!(
            "failed to create seeded price cache {}; the output may already contain a resumable \
             cache",
            destination.display()
        )
    })?;
    std::io::copy(&mut source, &mut output).wrap_err_with(|| {
        format!(
            "failed to copy seed price cache {} to {}",
            seed_path.display(),
            destination.display()
        )
    })?;
    output
        .flush()
        .wrap_err("failed to flush seeded price cache")?;
    output
        .sync_all()
        .wrap_err("failed to sync seeded price cache")?;
    sync_directory(output_root)
}

fn validate_cached_price(price: &CachedPrice) -> eyre::Result<()> {
    match price.status {
        CachedPriceStatus::Available => {
            ensure!(
                price.unit_price_usd.is_some() && price.price_timestamp.is_some(),
                "available cache record is missing price data"
            );
            let parsed = ExactDecimal::parse_non_negative(
                price.unit_price_usd.as_deref().expect("checked above"),
                "cached DefiLlama price",
            )?;
            ensure!(!parsed.is_zero(), "cached DefiLlama price must be positive");
            if let Some(confidence) = price.confidence.as_deref() {
                let confidence =
                    ExactDecimal::parse_non_negative(confidence, "cached DefiLlama confidence")?;
                ensure!(
                    confidence.is_at_most_one(),
                    "cached DefiLlama confidence exceeds one"
                );
            }
        }
        CachedPriceStatus::Missing => ensure!(
            price.symbol.is_none()
                && price.decimals.is_none()
                && price.unit_price_usd.is_none()
                && price.price_timestamp.is_none()
                && price.confidence.is_none(),
            "missing cache record unexpectedly contains price data"
        ),
    }
    Ok(())
}

fn write_cache_record(file: &mut File, record: &PriceCacheRecord, path: &Path) -> eyre::Result<()> {
    let mut encoded = serde_json::to_vec(record)
        .wrap_err_with(|| format!("failed to serialize price cache {}", path.display()))?;
    ensure!(
        encoded.len() < PRICE_CACHE_MAX_RECORD_BYTES,
        "price cache record exceeds the size limit"
    );
    encoded.push(b'\n');
    file.write_all(&encoded)
        .wrap_err_with(|| format!("failed to append price cache {}", path.display()))
}

impl DefiLlamaFetcher {
    fn new(base_url: &str) -> eyre::Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_owned();
        ensure!(
            base_url.starts_with("https://") || cfg!(test),
            "DefiLlama base URL must use HTTPS"
        );
        let client = Client::builder()
            .user_agent(concat!("agglayer/", env!("CARGO_PKG_VERSION")))
            .timeout(REQUEST_TIMEOUT)
            .build()
            .wrap_err("failed to construct DefiLlama HTTP client")?;
        Ok(Self { client, base_url })
    }

    async fn fetch_with_retry(
        &self,
        chunk: &RequestChunk,
    ) -> eyre::Result<BTreeMap<String, ProviderPrice>> {
        let coin_list = chunk.provider_coin_ids.join(",");
        let url = format!(
            "{}/prices/historical/{}/{}",
            self.base_url, chunk.requested_timestamp, coin_list
        );
        'attempts: for attempt in 1..=REQUEST_MAX_ATTEMPTS {
            let response = self
                .client
                .get(&url)
                .query(&[("searchWidth", SEARCH_WIDTH)])
                .send()
                .await;
            match response {
                Ok(mut response) if response.status().is_success() => {
                    let length = response.content_length();
                    ensure!(
                        length.is_none_or(|length| length <= RESPONSE_MAX_BYTES as u64),
                        "DefiLlama response exceeds the size limit"
                    );
                    let mut bytes = Vec::with_capacity(
                        length
                            .and_then(|length| usize::try_from(length).ok())
                            .unwrap_or_default(),
                    );
                    loop {
                        match response.chunk().await {
                            Ok(Some(response_chunk)) => {
                                let response_length = bytes
                                    .len()
                                    .checked_add(response_chunk.len())
                                    .context("DefiLlama response length overflow")?;
                                ensure!(
                                    response_length <= RESPONSE_MAX_BYTES,
                                    "DefiLlama response exceeds the size limit"
                                );
                                bytes.extend_from_slice(&response_chunk);
                            }
                            Ok(None) => return parse_defillama_response(&bytes, chunk),
                            Err(error) => {
                                if attempt == REQUEST_MAX_ATTEMPTS
                                    || !retryable_transport_error(&error)
                                {
                                    bail!(
                                        "failed to read DefiLlama response after {attempt} \
                                         attempt(s)"
                                    );
                                }
                                let delay = retry_delay(chunk, attempt);
                                eprintln!(
                                    "warning: transient DefiLlama response-body failure; retrying \
                                     attempt {}/{} in {} ms",
                                    attempt + 1,
                                    REQUEST_MAX_ATTEMPTS,
                                    delay.as_millis()
                                );
                                tokio::time::sleep(delay).await;
                                continue 'attempts;
                            }
                        }
                    }
                }
                Ok(response) => {
                    let status = response.status();
                    if attempt == REQUEST_MAX_ATTEMPTS || !retryable_status(status) {
                        bail!(
                            "DefiLlama historical price request failed with HTTP status {status} \
                             after {attempt} attempt(s)"
                        );
                    }
                    let retry_after = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse::<u64>().ok())
                        .map(Duration::from_secs)
                        .map(|duration| duration.min(REQUEST_MAX_BACKOFF));
                    let delay = retry_after.unwrap_or_else(|| retry_delay(chunk, attempt));
                    eprintln!(
                        "warning: transient DefiLlama HTTP status {status}; retrying attempt \
                         {}/{} in {} ms",
                        attempt + 1,
                        REQUEST_MAX_ATTEMPTS,
                        delay.as_millis()
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(error) => {
                    if attempt == REQUEST_MAX_ATTEMPTS || !retryable_transport_error(&error) {
                        bail!(
                            "DefiLlama historical price request failed after {attempt} attempt(s)"
                        );
                    }
                    let delay = retry_delay(chunk, attempt);
                    eprintln!(
                        "warning: transient DefiLlama transport failure; retrying attempt {}/{} \
                         in {} ms",
                        attempt + 1,
                        REQUEST_MAX_ATTEMPTS,
                        delay.as_millis()
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
        unreachable!("positive retry count always returns")
    }
}

impl PriceFetcher for DefiLlamaFetcher {
    fn fetch<'a>(&'a self, chunk: &'a RequestChunk) -> FetchFuture<'a> {
        Box::pin(self.fetch_with_retry(chunk))
    }
}

fn retryable_status(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

fn retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() || error.is_body()
}

fn retry_delay(chunk: &RequestChunk, attempt: u32) -> Duration {
    let exponent = attempt.saturating_sub(1).min(31);
    let backoff = REQUEST_INITIAL_BACKOFF
        .saturating_mul(1u32 << exponent)
        .min(REQUEST_MAX_BACKOFF);
    if backoff == REQUEST_MAX_BACKOFF {
        return backoff;
    }

    // A deterministic per-request jitter avoids synchronized retries without
    // adding an RNG dependency to this devtool.
    let hash = chunk
        .provider_coin_ids
        .iter()
        .flat_map(|coin| coin.bytes().chain(std::iter::once(0)))
        .fold(
            chunk.requested_timestamp ^ u64::from(attempt),
            |hash, byte| hash.wrapping_mul(0x100_0000_01b3) ^ u64::from(byte),
        );
    let jitter = Duration::from_millis(hash % 251);
    backoff.saturating_add(jitter).min(REQUEST_MAX_BACKOFF)
}

#[derive(Deserialize)]
struct DefiLlamaResponse {
    coins: BTreeMap<String, DefiLlamaCoin>,
}

#[derive(Deserialize)]
struct DefiLlamaCoin {
    decimals: Option<u8>,
    symbol: Option<String>,
    price: Number,
    timestamp: i64,
    confidence: Option<Number>,
}

fn parse_defillama_response(
    bytes: &[u8],
    chunk: &RequestChunk,
) -> eyre::Result<BTreeMap<String, ProviderPrice>> {
    let response: DefiLlamaResponse =
        serde_json::from_slice(bytes).wrap_err("DefiLlama returned malformed JSON")?;
    let requested: BTreeSet<_> = chunk.provider_coin_ids.iter().cloned().collect();
    let mut prices = BTreeMap::new();
    for (provider_coin_id, coin) in response.coins {
        if !requested.contains(&provider_coin_id) {
            continue;
        }
        ensure!(
            coin.timestamp >= 0,
            "DefiLlama returned a negative timestamp"
        );
        let price_timestamp = u64::try_from(coin.timestamp)
            .wrap_err("DefiLlama price timestamp is outside the supported range")?;
        let observed_date = timestamp_date(price_timestamp)?;
        let distance = coin
            .timestamp
            .checked_sub(i64::try_from(chunk.requested_timestamp).unwrap_or(i64::MAX))
            .context("DefiLlama timestamp distance overflow")?
            .unsigned_abs();
        if observed_date != chunk.pricing_date || distance > SEARCH_WIDTH_SECONDS as u64 {
            continue;
        }
        let unit_price_usd =
            ExactDecimal::parse_non_negative(&coin.price.to_string(), "DefiLlama price")?;
        ensure!(
            !unit_price_usd.is_zero(),
            "DefiLlama price must be positive"
        );
        let unit_price_usd = unit_price_usd.canonical()?;
        let confidence = coin
            .confidence
            .map(|value| {
                let confidence =
                    ExactDecimal::parse_non_negative(&value.to_string(), "DefiLlama confidence")?;
                ensure!(
                    confidence.is_at_most_one(),
                    "DefiLlama confidence exceeds one"
                );
                confidence.canonical()
            })
            .transpose()?;
        if let Some(symbol) = coin.symbol.as_deref() {
            ensure!(
                !symbol.is_empty() && symbol.len() <= 64 && !symbol.chars().any(char::is_control),
                "DefiLlama returned an invalid symbol"
            );
        }
        prices.insert(
            provider_coin_id,
            ProviderPrice {
                symbol: coin.symbol,
                decimals: coin.decimals,
                unit_price_usd,
                price_timestamp,
                confidence,
            },
        );
    }
    Ok(prices)
}

fn build_request_chunks(lookups: &[ProviderLookup]) -> eyre::Result<Vec<RequestChunk>> {
    let mut by_date: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for lookup in lookups {
        by_date
            .entry(lookup.pricing_date.clone())
            .or_default()
            .insert(lookup.provider_coin_id.clone());
    }

    let mut chunks = Vec::new();
    for (pricing_date, coin_ids) in by_date {
        let timestamp = requested_timestamp(&pricing_date)?;
        let prefix_length = DEFI_LLAMA_BASE_URL.len()
            + "/prices/historical/".len()
            + timestamp.to_string().len()
            + 1
            + "?searchWidth=12h".len();
        let mut current = Vec::new();
        let mut current_length = prefix_length;
        for coin_id in coin_ids {
            let added = coin_id.len() + usize::from(!current.is_empty());
            ensure!(
                prefix_length + coin_id.len() <= REQUEST_MAX_TARGET_BYTES,
                "DefiLlama coin ID is too long to request"
            );
            if current.len() == REQUEST_MAX_COINS
                || current_length + added > REQUEST_MAX_TARGET_BYTES
            {
                chunks.push(RequestChunk {
                    pricing_date: pricing_date.clone(),
                    requested_timestamp: timestamp,
                    provider_coin_ids: std::mem::take(&mut current),
                });
                current_length = prefix_length;
            }
            current_length += coin_id.len() + usize::from(!current.is_empty());
            current.push(coin_id);
        }
        if !current.is_empty() {
            chunks.push(RequestChunk {
                pricing_date,
                requested_timestamp: timestamp,
                provider_coin_ids: current,
            });
        }
    }
    Ok(chunks)
}

async fn resolve_provider_lookups<F: PriceFetcher>(
    lookups: &BTreeSet<ProviderLookup>,
    fetcher: &F,
    cache: &PriceCache,
    refresh_misses: bool,
) -> eyre::Result<LookupReport> {
    let mut report = LookupReport::default();
    let mut missing = Vec::new();
    for lookup in lookups {
        if let Some(price) = cache.get(lookup)? {
            if refresh_misses && price.status == CachedPriceStatus::Missing {
                report.refreshed_misses += 1;
                missing.push(lookup.clone());
            } else {
                report.cache_hits += 1;
                match price.status {
                    CachedPriceStatus::Available => report.available += 1,
                    CachedPriceStatus::Missing => report.missing += 1,
                }
            }
        } else {
            missing.push(lookup.clone());
        }
    }
    report.requested = u64::try_from(missing.len()).wrap_err("lookup count exceeds u64")?;

    let chunks = build_request_chunks(&missing)?;
    let chunk_count = chunks.len();
    if chunk_count > 0 {
        eprintln!(
            "Fetching {} DefiLlama token-day price(s) not reused from cache in {chunk_count} \
             request(s)...",
            missing.len()
        );
    } else {
        eprintln!("All DefiLlama token-day prices were found in the local cache.");
    }
    let mut responses = Box::pin(
        stream::iter(chunks)
            .map(|chunk| async move {
                let result = fetcher.fetch(&chunk).await;
                (chunk, result)
            })
            .buffer_unordered(REQUEST_CONCURRENCY),
    );
    let mut completed_chunks = 0usize;
    while let Some((chunk, response)) = responses.next().await {
        match response {
            Ok(mut prices) => {
                for provider_coin_id in chunk.provider_coin_ids {
                    let key = ProviderLookup {
                        provider_coin_id: provider_coin_id.clone(),
                        pricing_date: chunk.pricing_date.clone(),
                    };
                    let price = match prices.remove(&provider_coin_id) {
                        Some(price) => {
                            report.available += 1;
                            CachedPrice {
                                status: CachedPriceStatus::Available,
                                symbol: price.symbol,
                                decimals: price.decimals,
                                unit_price_usd: Some(price.unit_price_usd),
                                price_timestamp: Some(price.price_timestamp),
                                confidence: price.confidence,
                            }
                        }
                        None => {
                            report.missing += 1;
                            CachedPrice {
                                status: CachedPriceStatus::Missing,
                                symbol: None,
                                decimals: None,
                                unit_price_usd: None,
                                price_timestamp: None,
                                confidence: None,
                            }
                        }
                    };
                    if let Err(error) = cache.append(key, price) {
                        return sync_cache_before_error(cache, error);
                    }
                }
                // Each completed provider response becomes a durable resume
                // point before another completed response is consumed.
                cache.sync()?;
                completed_chunks += 1;
                if completed_chunks == chunk_count || completed_chunks.is_multiple_of(25) {
                    eprintln!("Cached {completed_chunks}/{chunk_count} DefiLlama response(s)...");
                }
            }
            Err(error) => return sync_cache_before_error(cache, error),
        }
    }
    cache.sync()?;
    Ok(report)
}

fn sync_cache_before_error<T>(cache: &PriceCache, error: eyre::Report) -> eyre::Result<T> {
    match cache.sync() {
        Ok(()) => Err(error),
        Err(sync_error) => Err(error).wrap_err(format!(
            "additionally failed to sync completed price-cache responses: {sync_error}"
        )),
    }
}

impl InputExport {
    fn open(input_path: &Path) -> eyre::Result<Self> {
        reject_parent_components(input_path, "input")?;
        let root = fs::canonicalize(input_path)
            .wrap_err_with(|| format!("failed to resolve input path {}", input_path.display()))?;
        let metadata = fs::symlink_metadata(input_path)
            .wrap_err_with(|| format!("failed to inspect input path {}", input_path.display()))?;
        ensure!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "input path {} must be a real directory",
            input_path.display()
        );
        let let_files = read_network_files(&root, LET_DIR)?;
        let ibe_files = read_network_files(&root, IBE_DIR)?;
        let lbt_files = read_network_files(&root, LBT_DIR)?;
        let let_networks: BTreeSet<_> = let_files.iter().map(|file| file.network_id).collect();
        let ibe_networks: BTreeSet<_> = ibe_files.iter().map(|file| file.network_id).collect();
        let lbt_networks: BTreeSet<_> = lbt_files.iter().map(|file| file.network_id).collect();
        ensure!(
            let_networks == ibe_networks && let_networks == lbt_networks,
            "LET, IBE, and LBT input directories must contain matching network files"
        );
        Ok(Self {
            root,
            let_files,
            ibe_files,
            lbt_files,
        })
    }

    fn history_files(&self, kind: HistoryKind) -> &[NetworkFile] {
        match kind {
            HistoryKind::Let => &self.let_files,
            HistoryKind::Ibe => &self.ibe_files,
        }
    }
}

fn read_network_files(root: &Path, directory: &str) -> eyre::Result<Vec<NetworkFile>> {
    let path = root.join(directory);
    let metadata = fs::symlink_metadata(&path)
        .wrap_err_with(|| format!("failed to inspect input directory {}", path.display()))?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "input directory {} must be a real directory",
        path.display()
    );
    let mut files = Vec::new();
    for entry in fs::read_dir(&path)
        .wrap_err_with(|| format!("failed to read input directory {}", path.display()))?
    {
        let entry = entry.wrap_err("failed to inspect input directory entry")?;
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| eyre::eyre!("input file name is not valid UTF-8"))?;
        let file_path = entry.path();
        let metadata = fs::symlink_metadata(&file_path)
            .wrap_err_with(|| format!("failed to inspect input file {}", file_path.display()))?;
        ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "input directory {} contains non-regular entry {file_name}",
            path.display()
        );
        let stem = file_name
            .strip_suffix(".json")
            .with_context(|| format!("input file {file_name} does not end in .json"))?;
        let network_id = stem
            .parse::<u32>()
            .wrap_err_with(|| format!("input file {file_name} has an invalid network ID"))?;
        ensure!(
            file_name == format!("{network_id}.json"),
            "input file {file_name} does not use a canonical network ID"
        );
        files.push(NetworkFile {
            network_id,
            file_name,
            path: file_path,
        });
    }
    files.sort_by_key(|file| file.network_id);
    ensure!(
        !files.is_empty(),
        "input directory {} is empty",
        path.display()
    );
    Ok(files)
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

impl PricingOutputWorkspace {
    fn prepare(input_root: &Path, output_path: &Path) -> eyre::Result<Self> {
        reject_parent_components(output_path, "output")?;
        let intended_output = resolve_output_path(output_path)?;
        ensure!(
            !paths_overlap(&intended_output, input_root),
            "output path {} overlaps input path {}",
            intended_output.display(),
            input_root.display()
        );
        let existed = match fs::symlink_metadata(output_path) {
            Ok(metadata) => {
                ensure!(
                    metadata.is_dir() && !metadata.file_type().is_symlink(),
                    "output path {} must be a real directory",
                    output_path.display()
                );
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!("failed to inspect output path {}", output_path.display())
                });
            }
        };
        if !existed {
            fs::create_dir_all(output_path).wrap_err_with(|| {
                format!(
                    "failed to create output directory {}",
                    output_path.display()
                )
            })?;
        }
        let root = fs::canonicalize(output_path).wrap_err_with(|| {
            format!(
                "failed to resolve output directory {}",
                output_path.display()
            )
        })?;
        ensure!(
            !paths_overlap(&root, input_root),
            "output path {} overlaps input path {}",
            root.display(),
            input_root.display()
        );
        ensure_no_pricing_output_collision(&root)?;
        let staging = TempDirBuilder::new()
            .prefix(".agglayer-price-enrichment-")
            .tempdir_in(&root)
            .wrap_err_with(|| {
                format!(
                    "failed to create pricing staging directory in {}",
                    root.display()
                )
            })?;
        Ok(Self {
            root,
            staging: Some(staging),
            remove_root_if_empty: !existed,
        })
    }

    fn root_path(&self) -> &Path {
        &self.root
    }

    fn staging_path(&self) -> &Path {
        self.staging
            .as_ref()
            .expect("staging directory exists until publication")
            .path()
    }

    fn publish(mut self) -> eyre::Result<()> {
        ensure_no_pricing_output_collision(&self.root)?;
        let mut published = Vec::new();
        for name in OUTPUT_NAMES {
            let staged = self.staging_path().join(name);
            let destination = self.root.join(name);
            if let Err(publish_error) = rename_noreplace(&staged, &destination) {
                let mut rollback_errors = Vec::new();
                for published_name in published.iter().rev() {
                    if let Err(error) = rename_noreplace(
                        &self.root.join(published_name),
                        &self.staging_path().join(published_name),
                    ) {
                        rollback_errors.push(format!("{published_name}: {error}"));
                    }
                }
                if rollback_errors.is_empty() {
                    return Err(publish_error)
                        .wrap_err_with(|| format!("failed to publish pricing output {name}"));
                }
                bail!(
                    "failed to publish pricing output {name}: {publish_error}; additionally \
                     failed to roll back {}",
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

impl Drop for PricingOutputWorkspace {
    fn drop(&mut self) {
        drop(self.staging.take());
        if self.remove_root_if_empty {
            let cache_path = self.root.join(PRICE_CACHE_FILE);
            match fs::symlink_metadata(&cache_path) {
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    let _ = fs::remove_dir(&self.root);
                }
                _ => {}
            }
        }
    }
}

fn ensure_no_pricing_output_collision(root: &Path) -> eyre::Result<()> {
    for name in OUTPUT_NAMES {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => bail!(
                "refusing to overwrite existing pricing output {}",
                path.display()
            ),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!("failed to inspect pricing output {}", path.display())
                });
            }
        }
    }
    Ok(())
}

struct MapArrayVisitor<'a, F> {
    visit: &'a mut F,
}

impl<'de, F> Visitor<'de> for MapArrayVisitor<'_, F>
where
    F: FnMut(Map<String, Value>) -> eyre::Result<()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON array of objects")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(row) = sequence.next_element::<Map<String, Value>>()? {
            (self.visit)(row).map_err(de::Error::custom)?;
        }
        Ok(())
    }
}

fn visit_map_array<F>(path: &Path, mut visit: F) -> eyre::Result<()>
where
    F: FnMut(Map<String, Value>) -> eyre::Result<()>,
{
    let file = File::open(path)
        .wrap_err_with(|| format!("failed to open input history {}", path.display()))?;
    let mut deserializer = serde_json::Deserializer::from_reader(BufReader::new(file));
    deserializer
        .deserialize_seq(MapArrayVisitor { visit: &mut visit })
        .wrap_err_with(|| format!("failed to stream input history {}", path.display()))?;
    deserializer
        .end()
        .wrap_err_with(|| format!("input history {} has trailing data", path.display()))
}

fn inspect_row(row: &Map<String, Value>) -> eyre::Result<(TokenIdentity, String, Option<String>)> {
    ensure!(
        !row.contains_key(PRICING_FIELD),
        "input row already contains {PRICING_FIELD}"
    );
    let amount_token = row
        .get("amountToken")
        .and_then(Value::as_str)
        .context("input row amountToken must be a string")?;
    let token = parse_token_identity(amount_token)?;
    let amount = row
        .get("amount")
        .and_then(Value::as_str)
        .context("input row amount must be a decimal string")?
        .to_owned();
    parse_amount(&amount)?;
    let settled_at = row
        .get("settledAt")
        .context("input row is missing settledAt")?;
    let pricing_date = parse_settlement_date(settled_at)?;
    Ok((token, amount, pricing_date))
}

fn build_inventory(input: &InputExport) -> eyre::Result<Inventory> {
    let mut inventory = Inventory::default();
    for kind in [HistoryKind::Let, HistoryKind::Ibe] {
        for network_file in input.history_files(kind) {
            let mut row_number = 0u64;
            visit_map_array(&network_file.path, |row| {
                row_number += 1;
                let (token, _, pricing_date) = inspect_row(&row).wrap_err_with(|| {
                    format!(
                        "invalid {} row {row_number} for network {}",
                        kind.directory().to_ascii_uppercase(),
                        network_file.network_id
                    )
                })?;
                inventory.amount_tokens.insert(token.canonical.clone());
                if let Some(pricing_date) = pricing_date {
                    inventory.logical_lookups.insert(LogicalLookup {
                        amount_token: token.canonical.clone(),
                        pricing_date: pricing_date.clone(),
                    });
                    if let Some(provider) = provider_identity(&token) {
                        inventory.provider_lookups.insert(ProviderLookup {
                            provider_coin_id: provider.provider_coin_id,
                            pricing_date,
                        });
                    }
                }
                Ok(())
            })?;
        }
    }
    Ok(inventory)
}

fn enrich_row(
    mut row: Map<String, Value>,
    kind: HistoryKind,
    prices: &HashMap<ProviderLookup, CachedPrice>,
) -> eyre::Result<(Map<String, Value>, PricingStatus)> {
    let (token, amount, pricing_date) = inspect_row(&row)?;
    let provider = provider_identity(&token);
    let pricing = create_row_pricing(kind, &amount, pricing_date, provider, prices)?;
    let status = pricing.status;
    row.insert(
        PRICING_FIELD.to_owned(),
        serde_json::to_value(pricing).wrap_err("failed to encode settlement-day pricing")?,
    );
    Ok((row, status))
}

fn create_row_pricing(
    kind: HistoryKind,
    amount: &str,
    pricing_date: Option<String>,
    provider: Option<ProviderIdentity>,
    prices: &HashMap<ProviderLookup, CachedPrice>,
) -> eyre::Result<SettlementDayPricing> {
    let provider_coin_id = provider
        .as_ref()
        .map(|provider| provider.provider_coin_id.clone());
    let provider_coin_method = provider.as_ref().map(|provider| provider.method);
    let mut output = SettlementDayPricing {
        status: PricingStatus::TimestampUnavailable,
        provider: PROVIDER,
        provider_coin_id,
        provider_coin_method,
        quote_currency: QUOTE_CURRENCY,
        pricing_date: pricing_date.clone(),
        timestamp_basis: kind.timestamp_basis(),
        requested_at: pricing_date
            .as_deref()
            .map(requested_timestamp)
            .transpose()?
            .map(format_timestamp)
            .transpose()?,
        price_timestamp: None,
        price_at: None,
        provider_confidence: None,
        provider_symbol: None,
        decimals: None,
        decimals_source: None,
        unit_price_usd: None,
        normalized_amount: None,
        value_usd: None,
    };

    let Some(pricing_date) = pricing_date else {
        apply_static_decimals(&mut output, amount, provider.as_ref())?;
        return Ok(output);
    };
    let Some(provider) = provider else {
        output.status = PricingStatus::UnsupportedNetwork;
        return Ok(output);
    };
    let key = ProviderLookup {
        provider_coin_id: provider.provider_coin_id.clone(),
        pricing_date,
    };
    let cached = prices.get(&key).with_context(|| {
        format!(
            "resolved price map is missing {} on {}",
            key.provider_coin_id, key.pricing_date
        )
    })?;
    if cached.status == CachedPriceStatus::Missing {
        output.status = PricingStatus::PriceUnavailable;
        apply_static_decimals(&mut output, amount, Some(&provider))?;
        return Ok(output);
    }

    let price = cached
        .unit_price_usd
        .as_deref()
        .context("available cached price has no unit price")?;
    let price_timestamp = cached
        .price_timestamp
        .context("available cached price has no timestamp")?;
    ensure!(
        timestamp_date(price_timestamp)? == key.pricing_date,
        "cached price observation falls outside its pricing date"
    );
    output.price_timestamp = Some(price_timestamp);
    output.price_at = Some(format_timestamp(price_timestamp)?);
    output.provider_confidence = cached.confidence.clone();
    output.provider_symbol = cached.symbol.clone();
    output.unit_price_usd = Some(price.to_owned());

    let (decimals, source) = match provider.decimals_override {
        Some(value) => value,
        None => match cached.decimals {
            Some(decimals) => (decimals, "defillama"),
            None => {
                output.status = PricingStatus::DecimalsUnavailable;
                return Ok(output);
            }
        },
    };
    output.status = PricingStatus::Priced;
    output.decimals = Some(decimals);
    output.decimals_source = Some(source);
    output.normalized_amount = Some(normalize_amount(amount, decimals)?);
    output.value_usd = Some(calculate_value_usd(amount, decimals, price)?);
    Ok(output)
}

fn apply_static_decimals(
    output: &mut SettlementDayPricing,
    amount: &str,
    provider: Option<&ProviderIdentity>,
) -> eyre::Result<()> {
    if let Some((decimals, source)) = provider.and_then(|provider| provider.decimals_override) {
        output.decimals = Some(decimals);
        output.decimals_source = Some(source);
        output.normalized_amount = Some(normalize_amount(amount, decimals)?);
    }
    Ok(())
}

fn write_enriched_history(
    source: &NetworkFile,
    destination: &Path,
    kind: HistoryKind,
    prices: &HashMap<ProviderLookup, CachedPrice>,
    coverage: &mut KindCoverage,
) -> eyre::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .wrap_err_with(|| format!("failed to create {}", destination.display()))?;
    let writer = BufWriter::new(file);
    let mut serializer = serde_json::Serializer::pretty(writer);
    let mut sequence = serializer
        .serialize_seq(None)
        .wrap_err_with(|| format!("failed to begin serializing {}", destination.display()))?;
    let mut row_number = 0u64;
    visit_map_array(&source.path, |row| {
        row_number += 1;
        let (row, status) = enrich_row(row, kind, prices).wrap_err_with(|| {
            format!(
                "failed to enrich {} row {row_number} for network {}",
                kind.directory().to_ascii_uppercase(),
                source.network_id
            )
        })?;
        coverage.record(source.network_id, status);
        sequence
            .serialize_element(&row)
            .wrap_err_with(|| format!("failed to serialize {}", destination.display()))
    })?;
    sequence
        .end()
        .wrap_err_with(|| format!("failed to finish serializing {}", destination.display()))?;
    finish_output_file(serializer.into_inner(), destination)
}

fn copy_lbt_file(source: &Path, destination: &Path) -> eyre::Result<()> {
    let mut input = File::open(source)
        .wrap_err_with(|| format!("failed to open LBT input {}", source.display()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .wrap_err_with(|| format!("failed to create LBT output {}", destination.display()))?;
    std::io::copy(&mut input, &mut output).wrap_err_with(|| {
        format!(
            "failed to copy LBT input {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    output
        .flush()
        .wrap_err("failed to flush copied LBT output")?;
    output
        .sync_all()
        .wrap_err("failed to sync copied LBT output")
}

fn write_pricing_report(path: &Path, report: &PricingReport) -> eyre::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("failed to create pricing report {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, report)
        .wrap_err("failed to serialize pricing report")?;
    finish_output_file(writer, path)
}

/// Enriches a tree export with DefiLlama USD pricing at UTC noon on each
/// certificate's L1 settlement day.
pub(crate) async fn enrich_tree_prices(
    input_path: &Path,
    output_path: &Path,
    refresh_misses: bool,
    seed_price_cache: Option<&Path>,
) -> eyre::Result<()> {
    let fetcher = DefiLlamaFetcher::new(DEFI_LLAMA_BASE_URL)?;
    enrich_tree_prices_with_fetcher(
        input_path,
        output_path,
        refresh_misses,
        seed_price_cache,
        &fetcher,
    )
    .await
}

async fn enrich_tree_prices_with_fetcher<F: PriceFetcher>(
    input_path: &Path,
    output_path: &Path,
    refresh_misses: bool,
    seed_price_cache: Option<&Path>,
    fetcher: &F,
) -> eyre::Result<()> {
    let input = InputExport::open(input_path)?;
    let output = PricingOutputWorkspace::prepare(&input.root, output_path)?;
    eprintln!(
        "Scanning LET and IBE histories in {} for unique token-day lookups...",
        input.root.display()
    );
    let inventory = build_inventory(&input)?;
    eprintln!(
        "Found {} amount token(s), {} token-day pair(s), and {} provider lookup(s).",
        inventory.amount_tokens.len(),
        inventory.logical_lookups.len(),
        inventory.provider_lookups.len()
    );
    copy_seed_price_cache(output.root_path(), seed_price_cache)?;
    let cache = PriceCache::open(output.root_path())?;
    let lookup_report =
        resolve_provider_lookups(&inventory.provider_lookups, fetcher, &cache, refresh_misses)
            .await?;
    let prices = cache.snapshot()?;

    let let_output = output.staging_path().join(LET_DIR);
    let ibe_output = output.staging_path().join(IBE_DIR);
    let lbt_output = output.staging_path().join(LBT_DIR);
    fs::create_dir(&let_output).wrap_err("failed to create staged LET output")?;
    fs::create_dir(&ibe_output).wrap_err("failed to create staged IBE output")?;
    fs::create_dir(&lbt_output).wrap_err("failed to create staged LBT output")?;

    let mut coverage = Coverage::default();
    for kind in [HistoryKind::Let, HistoryKind::Ibe] {
        let directory = match kind {
            HistoryKind::Let => &let_output,
            HistoryKind::Ibe => &ibe_output,
        };
        let kind_coverage = match kind {
            HistoryKind::Let => &mut coverage.let_rows,
            HistoryKind::Ibe => &mut coverage.ibe_rows,
        };
        for source in input.history_files(kind) {
            eprintln!(
                "Writing enriched {} history for network {}...",
                kind.directory().to_ascii_uppercase(),
                source.network_id
            );
            write_enriched_history(
                source,
                &directory.join(&source.file_name),
                kind,
                &prices,
                kind_coverage,
            )?;
        }
    }
    for source in &input.lbt_files {
        copy_lbt_file(&source.path, &lbt_output.join(&source.file_name))?;
    }

    sync_directory(&let_output)?;
    sync_directory(&ibe_output)?;
    sync_directory(&lbt_output)?;
    let report = PricingReport {
        schema_version: 1,
        provider: PROVIDER,
        quote_currency: QUOTE_CURRENCY,
        daily_price_convention: DailyPriceConvention {
            requested_time_utc: REQUEST_TIME_UTC,
            search_width: SEARCH_WIDTH,
            same_utc_day_required: true,
        },
        timestamp_bases: TimestampBases {
            r#let: HistoryKind::Let.timestamp_basis(),
            ibe: HistoryKind::Ibe.timestamp_basis(),
        },
        lbt: "copiedUnchanged",
        let_and_ibe_aggregated: false,
        unique_amount_tokens: inventory.amount_tokens.len(),
        unique_token_days: inventory.logical_lookups.len(),
        unique_provider_lookups: inventory.provider_lookups.len(),
        lookups: lookup_report,
        coverage: ReportCoverage {
            r#let: coverage.let_rows,
            ibe: coverage.ibe_rows,
        },
    };
    write_pricing_report(&output.staging_path().join(PRICING_REPORT_FILE), &report)?;
    sync_directory(output.staging_path())?;
    cache.sync()?;
    output.publish()?;
    eprintln!("Pricing enrichment completed in {}.", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write as _, sync::Mutex};

    use serde_json::json;

    use super::*;

    #[test]
    fn exact_decimal_parsing_and_arithmetic_do_not_round() {
        let cases = [
            ("0", "0"),
            ("001.2300", "1.23"),
            ("1e-6", "0.000001"),
            ("1.25E3", "1250"),
            ("0.000000000000000001", "0.000000000000000001"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                ExactDecimal::parse_non_negative(input, "test")
                    .unwrap()
                    .canonical()
                    .unwrap(),
                expected
            );
        }
        assert_eq!(normalize_amount("100000000000000000", 18).unwrap(), "0.1");
        assert_eq!(
            calculate_value_usd("100000000000000000", 18, "1234.50").unwrap(),
            "123.45"
        );
        assert_eq!(
            calculate_value_usd("1", 6, "1e-6").unwrap(),
            "0.000000000001"
        );
        assert_eq!(
            normalize_amount(U256_MAX_DECIMAL, 0).unwrap(),
            U256_MAX_DECIMAL
        );
    }

    #[test]
    fn exact_decimal_rejects_invalid_or_unbounded_values() {
        for invalid in ["", "-1", ".", "1.2.3", "nan", "1e", "1e1001"] {
            assert!(ExactDecimal::parse_non_negative(invalid, "test").is_err());
        }
        assert!(parse_amount("-1").is_err());
        assert!(parse_amount(&format!("{U256_MAX_DECIMAL}0")).is_err());
    }

    #[test]
    fn provider_identity_uses_all_static_namespaces_and_native_eth() {
        let zero = "0x0000000000000000000000000000000000000000";
        let eth = parse_token_identity(&format!("0:{zero}")).unwrap();
        let identity = provider_identity(&eth).unwrap();
        assert_eq!(identity.provider_coin_id, "coingecko:ethereum");
        assert_eq!(identity.decimals_override, Some((18, "protocol")));

        for (network, namespace) in NETWORK_MAPPINGS {
            if *network == 0 {
                continue;
            }
            let token = parse_token_identity(&format!(
                "{network}:0x1111111111111111111111111111111111111111"
            ))
            .unwrap();
            assert_eq!(
                provider_identity(&token).unwrap().provider_coin_id,
                format!("{namespace}:0x1111111111111111111111111111111111111111")
            );
        }
        let unknown =
            parse_token_identity("99:0x1111111111111111111111111111111111111111").unwrap();
        assert!(provider_identity(&unknown).is_none());
    }

    #[test]
    fn official_vault_redirects_have_explicit_one_to_one_provenance() {
        for redirect in VAULT_REDIRECTS {
            let token = parse_token_identity(&format!("0:{}", redirect.vault)).unwrap();
            let identity = provider_identity(&token).unwrap();
            assert_eq!(
                identity.provider_coin_id,
                format!("ethereum:{}", redirect.underlying)
            );
            assert_eq!(identity.method, "vaultBridge1To1");
            assert_eq!(
                identity.decimals_override,
                Some((redirect.decimals, "vaultBridge1To1"))
            );
        }
    }

    #[test]
    fn historical_response_requires_same_utc_day_and_search_window() {
        let chunk = RequestChunk {
            pricing_date: "2025-04-10".to_owned(),
            requested_timestamp: requested_timestamp("2025-04-10").unwrap(),
            provider_coin_ids: vec![
                "ethereum:0x1111111111111111111111111111111111111111".to_owned(),
                "ethereum:0x2222222222222222222222222222222222222222".to_owned(),
            ],
        };
        let body = json!({
            "coins": {
                "ethereum:0x1111111111111111111111111111111111111111": {
                    "decimals": 6,
                    "symbol": "TOK",
                    "price": 1.2300,
                    "timestamp": chunk.requested_timestamp - 1,
                    "confidence": 0.99
                },
                "ethereum:0x2222222222222222222222222222222222222222": {
                    "decimals": 18,
                    "symbol": "OLD",
                    "price": 2,
                    "timestamp": requested_timestamp("2025-04-09").unwrap()
                },
                "ethereum:0xffffffffffffffffffffffffffffffffffffffff": {
                    "decimals": 18,
                    "price": 9,
                    "timestamp": chunk.requested_timestamp
                }
            }
        });
        let parsed = parse_defillama_response(&serde_json::to_vec(&body).unwrap(), &chunk).unwrap();
        assert_eq!(parsed.len(), 1);
        let price = &parsed["ethereum:0x1111111111111111111111111111111111111111"];
        assert_eq!(price.unit_price_usd, "1.23");
        assert_eq!(price.confidence.as_deref(), Some("0.99"));
    }

    #[test]
    fn historical_response_rejects_zero_prices_and_out_of_range_confidence() {
        let chunk = RequestChunk {
            pricing_date: "2025-04-10".to_owned(),
            requested_timestamp: requested_timestamp("2025-04-10").unwrap(),
            provider_coin_ids: vec![
                "ethereum:0x1111111111111111111111111111111111111111".to_owned()
            ],
        };
        let coin_id = chunk.provider_coin_ids[0].clone();
        let zero_price = json!({
            "coins": {
                (coin_id.clone()): {
                    "price": 0,
                    "timestamp": chunk.requested_timestamp,
                    "confidence": 1
                }
            }
        });
        let error = parse_defillama_response(&serde_json::to_vec(&zero_price).unwrap(), &chunk)
            .unwrap_err();
        assert!(error.to_string().contains("price must be positive"));

        let invalid_confidence = json!({
            "coins": {
                (coin_id): {
                    "price": 1,
                    "timestamp": chunk.requested_timestamp,
                    "confidence": 1.0001
                }
            }
        });
        let error =
            parse_defillama_response(&serde_json::to_vec(&invalid_confidence).unwrap(), &chunk)
                .unwrap_err();
        assert!(error.to_string().contains("confidence exceeds one"));
    }

    #[test]
    fn request_chunks_are_stable_bounded_and_grouped_by_day() {
        let mut lookups = Vec::new();
        for day in ["2025-01-02", "2025-01-01"] {
            for index in (0..75).rev() {
                lookups.push(ProviderLookup {
                    provider_coin_id: format!("ethereum:0x{index:040x}"),
                    pricing_date: day.to_owned(),
                });
            }
        }
        lookups.sort();
        let chunks = build_request_chunks(&lookups).unwrap();
        assert_eq!(chunks.len(), 4);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.provider_coin_ids.len() <= REQUEST_MAX_COINS));
        assert_eq!(chunks[0].pricing_date, "2025-01-01");
        assert!(chunks[0].provider_coin_ids.is_sorted());
    }

    #[test]
    fn cache_round_trips_and_recovers_an_incomplete_tail() {
        let temp = tempfile::tempdir().unwrap();
        let key = ProviderLookup {
            provider_coin_id: "coingecko:ethereum".to_owned(),
            pricing_date: "2025-01-01".to_owned(),
        };
        let price = CachedPrice {
            status: CachedPriceStatus::Available,
            symbol: Some("ETH".to_owned()),
            decimals: None,
            unit_price_usd: Some("1234.5".to_owned()),
            price_timestamp: Some(requested_timestamp("2025-01-01").unwrap()),
            confidence: Some("0.99".to_owned()),
        };
        {
            let cache = PriceCache::open(temp.path()).unwrap();
            cache.append(key.clone(), price.clone()).unwrap();
            cache.sync().unwrap();
        }
        let cache_path = temp.path().join(PRICE_CACHE_FILE);
        let complete_length = fs::metadata(&cache_path).unwrap().len();
        {
            let mut file = OpenOptions::new().append(true).open(&cache_path).unwrap();
            file.write_all(b"{\"kind\":\"price\"").unwrap();
            file.sync_all().unwrap();
        }
        {
            let cache = PriceCache::open(temp.path()).unwrap();
            assert_eq!(cache.get(&key).unwrap(), Some(price));
        }
        assert_eq!(fs::metadata(cache_path).unwrap().len(), complete_length);
    }

    #[test]
    fn cache_rejects_an_incomplete_header_without_truncating_it() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join(PRICE_CACHE_FILE);
        let partial_header = b"{\"kind\":\"header\"";
        fs::write(&cache_path, partial_header).unwrap();

        let error = match PriceCache::open(temp.path()) {
            Ok(_) => panic!("an incomplete header must not be accepted"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("has no complete header"));
        assert_eq!(fs::read(cache_path).unwrap(), partial_header);
    }

    struct OutOfOrderFailureFetcher {
        stalled_chunk_first_coin: String,
        successful_chunk_first_coin: String,
        fatal_chunk_first_coin: String,
        calls: Mutex<Vec<String>>,
    }

    impl PriceFetcher for OutOfOrderFailureFetcher {
        fn fetch<'a>(&'a self, chunk: &'a RequestChunk) -> FetchFuture<'a> {
            Box::pin(async move {
                let first_coin = chunk.provider_coin_ids[0].clone();
                self.calls.lock().unwrap().push(first_coin.clone());
                if first_coin == self.stalled_chunk_first_coin {
                    std::future::pending::<()>().await;
                    unreachable!("the stalled request must be cancelled after the fatal response");
                }
                if first_coin == self.successful_chunk_first_coin {
                    return Ok(BTreeMap::new());
                }
                assert_eq!(first_coin, self.fatal_chunk_first_coin);
                Err(eyre::eyre!("intentional provider failure"))
            })
        }
    }

    #[tokio::test]
    async fn completed_out_of_order_chunk_is_cached_before_first_fatal_response() {
        let pricing_date = "2025-01-01";
        let lookups: BTreeSet<_> = (0..101)
            .map(|index| ProviderLookup {
                provider_coin_id: format!("ethereum:0x{index:040x}"),
                pricing_date: pricing_date.to_owned(),
            })
            .collect();
        let chunks = build_request_chunks(&lookups.iter().cloned().collect::<Vec<_>>()).unwrap();
        assert_eq!(chunks.len(), 3);
        let fetcher = OutOfOrderFailureFetcher {
            stalled_chunk_first_coin: chunks[0].provider_coin_ids[0].clone(),
            successful_chunk_first_coin: chunks[1].provider_coin_ids[0].clone(),
            fatal_chunk_first_coin: chunks[2].provider_coin_ids[0].clone(),
            calls: Mutex::new(Vec::new()),
        };
        let temp = tempfile::tempdir().unwrap();
        {
            let cache = PriceCache::open(temp.path()).unwrap();
            let result = tokio::time::timeout(
                Duration::from_secs(1),
                resolve_provider_lookups(&lookups, &fetcher, &cache, false),
            )
            .await
            .expect("fatal provider response should cancel the stalled request");
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("intentional provider failure"));

            let successful_key = ProviderLookup {
                provider_coin_id: chunks[1].provider_coin_ids[0].clone(),
                pricing_date: pricing_date.to_owned(),
            };
            assert_eq!(
                cache.get(&successful_key).unwrap().unwrap().status,
                CachedPriceStatus::Missing
            );
            let stalled_key = ProviderLookup {
                provider_coin_id: chunks[0].provider_coin_ids[0].clone(),
                pricing_date: pricing_date.to_owned(),
            };
            assert!(cache.get(&stalled_key).unwrap().is_none());
        }

        let reopened = PriceCache::open(temp.path()).unwrap();
        let successful_key = ProviderLookup {
            provider_coin_id: chunks[1].provider_coin_ids[0].clone(),
            pricing_date: pricing_date.to_owned(),
        };
        assert_eq!(
            reopened.get(&successful_key).unwrap().unwrap().status,
            CachedPriceStatus::Missing
        );
        assert_eq!(fetcher.calls.lock().unwrap().len(), 3);
    }

    #[derive(Default)]
    struct FakeFetcher {
        prices: BTreeMap<(String, String), ProviderPrice>,
        calls: Mutex<Vec<RequestChunk>>,
    }

    impl PriceFetcher for FakeFetcher {
        fn fetch<'a>(&'a self, chunk: &'a RequestChunk) -> FetchFuture<'a> {
            Box::pin(async move {
                self.calls.lock().unwrap().push(chunk.clone());
                Ok(chunk
                    .provider_coin_ids
                    .iter()
                    .filter_map(|coin| {
                        self.prices
                            .get(&(coin.clone(), chunk.pricing_date.clone()))
                            .cloned()
                            .map(|price| (coin.clone(), price))
                    })
                    .collect())
            })
        }
    }

    fn sample_price(price: &str, decimals: Option<u8>, date: &str) -> ProviderPrice {
        ProviderPrice {
            symbol: Some("TEST".to_owned()),
            decimals,
            unit_price_usd: price.to_owned(),
            price_timestamp: requested_timestamp(date).unwrap(),
            confidence: Some("0.9".to_owned()),
        }
    }

    #[tokio::test]
    async fn refresh_misses_retries_only_negative_cache_entries() {
        let temp = tempfile::tempdir().unwrap();
        let key = ProviderLookup {
            provider_coin_id: "ethereum:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
            pricing_date: "2025-04-10".to_owned(),
        };
        let lookups = BTreeSet::from([key.clone()]);
        let cache = PriceCache::open(temp.path()).unwrap();
        cache
            .append(
                key.clone(),
                CachedPrice {
                    status: CachedPriceStatus::Missing,
                    symbol: None,
                    decimals: None,
                    unit_price_usd: None,
                    price_timestamp: None,
                    confidence: None,
                },
            )
            .unwrap();
        cache.sync().unwrap();

        let mut fetcher = FakeFetcher::default();
        fetcher.prices.insert(
            (key.provider_coin_id.clone(), key.pricing_date.clone()),
            sample_price("0.9999", Some(6), &key.pricing_date),
        );

        let cached = resolve_provider_lookups(&lookups, &fetcher, &cache, false)
            .await
            .unwrap();
        assert_eq!(cached.cache_hits, 1);
        assert_eq!(cached.requested, 0);
        assert!(fetcher.calls.lock().unwrap().is_empty());

        let refreshed = resolve_provider_lookups(&lookups, &fetcher, &cache, true)
            .await
            .unwrap();
        assert_eq!(refreshed.refreshed_misses, 1);
        assert_eq!(refreshed.requested, 1);
        assert_eq!(refreshed.available, 1);
        assert_eq!(fetcher.calls.lock().unwrap().len(), 1);
        assert_eq!(
            cache.get(&key).unwrap().unwrap().status,
            CachedPriceStatus::Available
        );
        drop(cache);

        let reopened = PriceCache::open(temp.path()).unwrap();
        assert_eq!(
            reopened.get(&key).unwrap().unwrap().status,
            CachedPriceStatus::Available
        );
    }

    #[test]
    fn seed_price_cache_is_copied_without_modifying_its_source() {
        let temp = tempfile::tempdir().unwrap();
        let seed_root = temp.path().join("seed");
        let output_root = temp.path().join("output");
        fs::create_dir(&seed_root).unwrap();
        fs::create_dir(&output_root).unwrap();
        let key = ProviderLookup {
            provider_coin_id: "coingecko:ethereum".to_owned(),
            pricing_date: "2025-04-10".to_owned(),
        };
        {
            let cache = PriceCache::open(&seed_root).unwrap();
            cache
                .append(
                    key.clone(),
                    CachedPrice {
                        status: CachedPriceStatus::Available,
                        symbol: Some("ETH".to_owned()),
                        decimals: None,
                        unit_price_usd: Some("2000".to_owned()),
                        price_timestamp: Some(requested_timestamp(&key.pricing_date).unwrap()),
                        confidence: Some("0.99".to_owned()),
                    },
                )
                .unwrap();
            cache.sync().unwrap();
        }
        let seed_path = seed_root.join(PRICE_CACHE_FILE);
        let seed_before = fs::read(&seed_path).unwrap();

        copy_seed_price_cache(&output_root, Some(&seed_path)).unwrap();
        assert_eq!(fs::read(&seed_path).unwrap(), seed_before);
        let copied = PriceCache::open(&output_root).unwrap();
        assert_eq!(
            copied.get(&key).unwrap().unwrap().status,
            CachedPriceStatus::Available
        );
        drop(copied);
        assert!(copy_seed_price_cache(&output_root, Some(&seed_path)).is_err());
        assert_eq!(fs::read(&seed_path).unwrap(), seed_before);
    }

    fn write_export(root: &Path, let_rows: Value, ibe_rows: Value, lbt: &[u8]) {
        for directory in [LET_DIR, IBE_DIR, LBT_DIR] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::write(
            root.join(LET_DIR).join("1.json"),
            serde_json::to_vec_pretty(&let_rows).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(IBE_DIR).join("1.json"),
            serde_json::to_vec_pretty(&ibe_rows).unwrap(),
        )
        .unwrap();
        fs::write(root.join(LBT_DIR).join("1.json"), lbt).unwrap();
    }

    #[tokio::test]
    async fn enrichment_streams_rows_preserves_unknown_fields_and_reports_coverage() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("input");
        let output = temp.path().join("output");
        let usdc = "0:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let dai = "0:0x6b175474e89094c44da98b954eedeac495271d0f";
        let missing = "0:0x1111111111111111111111111111111111111111";
        let unsupported = "99:0x2222222222222222222222222222222222222222";
        let let_rows = json!([
            {"amountToken": usdc, "amount": "1000000", "settledAt": "2025-04-10T01:02:03Z", "unknownFutureField": {"kept": true}},
            {"amountToken": dai, "amount": "1000000000000000000", "settledAt": "2025-04-10T02:00:00Z"},
            {"amountToken": missing, "amount": "7", "settledAt": "2025-04-10T03:00:00Z"},
            {"amountToken": unsupported, "amount": "8", "settledAt": "2025-04-10T04:00:00Z"},
            {"amountToken": "0:0x0000000000000000000000000000000000000000", "amount": "100000000000000000", "settledAt": null}
        ]);
        let vault = VAULT_REDIRECTS[0];
        let ibe_rows = json!([
            {"amountToken": format!("0:{}", vault.vault), "amount": "2000000000000000000", "settledAt": "2025-04-10T05:00:00Z"}
        ]);
        let lbt = b"{\n  \"0:0x00\": \"1\"\n}\n";
        write_export(&input, let_rows, ibe_rows, lbt);
        let source_before = fs::read(input.join(LET_DIR).join("1.json")).unwrap();

        let mut fetcher = FakeFetcher::default();
        fetcher.prices.insert(
            (
                "ethereum:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
                "2025-04-10".to_owned(),
            ),
            sample_price("0.9999", Some(6), "2025-04-10"),
        );
        fetcher.prices.insert(
            (
                "ethereum:0x6b175474e89094c44da98b954eedeac495271d0f".to_owned(),
                "2025-04-10".to_owned(),
            ),
            sample_price("1", None, "2025-04-10"),
        );
        fetcher.prices.insert(
            (
                format!("ethereum:{}", vault.underlying),
                "2025-04-10".to_owned(),
            ),
            sample_price("2000", Some(18), "2025-04-10"),
        );

        enrich_tree_prices_with_fetcher(&input, &output, false, None, &fetcher)
            .await
            .unwrap();

        assert_eq!(
            fs::read(input.join(LET_DIR).join("1.json")).unwrap(),
            source_before
        );
        assert_eq!(fs::read(output.join(LBT_DIR).join("1.json")).unwrap(), lbt);
        let priced: Value =
            serde_json::from_slice(&fs::read(output.join(LET_DIR).join("1.json")).unwrap())
                .unwrap();
        assert_eq!(priced[0]["unknownFutureField"]["kept"], true);
        assert_eq!(priced[0][PRICING_FIELD]["status"], "priced");
        assert_eq!(priced[0][PRICING_FIELD]["normalizedAmount"], "1");
        assert_eq!(priced[0][PRICING_FIELD]["valueUsd"], "0.9999");
        assert_eq!(priced[1][PRICING_FIELD]["status"], "decimalsUnavailable");
        assert_eq!(priced[1][PRICING_FIELD]["unitPriceUsd"], "1");
        assert!(priced[1][PRICING_FIELD]["valueUsd"].is_null());
        assert_eq!(priced[2][PRICING_FIELD]["status"], "priceUnavailable");
        assert_eq!(priced[3][PRICING_FIELD]["status"], "unsupportedNetwork");
        assert_eq!(priced[4][PRICING_FIELD]["status"], "timestampUnavailable");
        assert_eq!(priced[4][PRICING_FIELD]["normalizedAmount"], "0.1");

        let imported: Value =
            serde_json::from_slice(&fs::read(output.join(IBE_DIR).join("1.json")).unwrap())
                .unwrap();
        assert_eq!(
            imported[0][PRICING_FIELD]["providerCoinMethod"],
            "vaultBridge1To1"
        );
        assert_eq!(
            imported[0][PRICING_FIELD]["decimalsSource"],
            "vaultBridge1To1"
        );
        assert_eq!(imported[0][PRICING_FIELD]["valueUsd"], "4000");
        assert_eq!(
            imported[0][PRICING_FIELD]["timestampBasis"],
            "claimingCertificateSettlement"
        );

        let report: Value =
            serde_json::from_slice(&fs::read(output.join(PRICING_REPORT_FILE)).unwrap()).unwrap();
        assert_eq!(report["coverage"]["let"]["rows"], 5);
        assert_eq!(report["coverage"]["ibe"]["rows"], 1);
        assert_eq!(report["letAndIbeAggregated"], false);
        assert_eq!(report["lbt"], "copiedUnchanged");
        assert!(output.join(PRICE_CACHE_FILE).is_file());
    }

    #[tokio::test]
    async fn overlap_and_existing_output_are_rejected_without_touching_source() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("input");
        write_export(&input, json!([]), json!([]), b"{}\n");
        let source = fs::read(input.join(LBT_DIR).join("1.json")).unwrap();
        let fetcher = FakeFetcher::default();
        assert!(enrich_tree_prices_with_fetcher(
            &input,
            &input.join("nested"),
            false,
            None,
            &fetcher
        )
        .await
        .is_err());
        assert_eq!(
            fs::read(input.join(LBT_DIR).join("1.json")).unwrap(),
            source
        );

        let output = temp.path().join("output");
        fs::create_dir_all(output.join(LET_DIR)).unwrap();
        assert!(
            enrich_tree_prices_with_fetcher(&input, &output, false, None, &fetcher)
                .await
                .is_err()
        );
    }

    #[test]
    fn duplicate_hash_fields_are_not_interpreted_or_deduplicated() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("rows.json");
        fs::write(
            &path,
            serde_json::to_vec(&json!([
                {"leafHash": "same", "amountToken": "0:0x0000000000000000000000000000000000000000", "amount": "1", "settledAt": null},
                {"leafHash": "same", "amountToken": "0:0x0000000000000000000000000000000000000000", "amount": "1", "settledAt": null}
            ]))
            .unwrap(),
        )
        .unwrap();
        let mut rows = 0;
        visit_map_array(&path, |_| {
            rows += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(rows, 2);
    }
}
