//! Measures SP1 network proving latency over the matrix
//! `{pp, noop} x {compressed, plonk, groth16}` to separate the cost of the
//! guest program from the cost of the SNARK wrap.
//!
//! Every request uses `FulfillmentStrategy::Reserved` so the measurement
//! reflects dedicated capacity rather than the on-demand `Hosted` queue.

use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx, PessimisticRootInput, U256,
};
use clap::Parser;
use pessimistic_proof::{
    core::commitment::PessimisticRootCommitmentVersion, unified_bridge::TokenInfo,
};
use pessimistic_proof_test_suite::{
    runner::Runner, sample_data as data, PESSIMISTIC_PROOF_ELF,
};
use serde::Serialize;
use sp1_sdk::{
    network::{
        proto::{
            types::{FulfillmentStatus, FulfillmentStrategy, ProofMode},
            GetFilteredProofRequestsResponse,
        },
        signer::NetworkSigner,
        NetworkClient, NetworkMode, B256,
    },
    Elf, HashableKey as _, NetworkProver, ProveRequest as _, Prover as _, ProvingKey as _,
    SP1ProofMode, SP1Stdin,
};
use tracing::{info, warn};

/// A guest that does nothing: the irreducible floor of the proving pipeline.
const NOOP_ELF: &[u8] = include_bytes!("../../../agglayer-sp1/tests/empty.elf");

const RESERVED_RPC_URL: &str = "https://rpc.production.succinct.xyz";
const RESERVED_EXPLORER: &str = "https://explorer.reserved.succinct.xyz";
const POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of bridge exits in the pp workload.
    #[clap(long, default_value = "1")]
    n_exits: usize,

    /// Number of imported bridge exits in the pp workload.
    #[clap(long, default_value = "1")]
    n_imported_exits: usize,

    /// Number of bridge exits in the pp-large workload.
    #[clap(long, default_value = "100")]
    n_exits_large: usize,

    /// Number of imported bridge exits in the pp-large workload.
    #[clap(long, default_value = "100")]
    n_imported_exits_large: usize,

    /// Comma-separated workloads to run. Lets a later run add one workload to
    /// an existing report without re-measuring the others.
    #[clap(long, default_value = "noop,pp,pp-large")]
    workloads: String,

    /// Comma-separated proof modes to run. Lets a later run top up only the
    /// cells that are actually noisy.
    #[clap(long, default_value = "compressed,plonk,groth16")]
    modes: String,

    /// Baseline repetitions per cell.
    #[clap(long, default_value = "2")]
    reps: usize,

    /// Relative spread above which a cell earns one extra repetition.
    #[clap(long, default_value = "0.25")]
    adaptive_threshold: f64,

    /// Per-request timeout in seconds.
    #[clap(long, default_value = "1200")]
    timeout_secs: u64,

    /// Where to write the report pair.
    #[clap(long, default_value = "benchmarks/sp1-proving")]
    out_dir: PathBuf,

    /// Skip the discarded warm-up request per ELF.
    #[clap(long)]
    no_warmup: bool,

    /// Validate the local path (execute + setup) and exit without submitting
    /// anything to the network.
    #[clap(long)]
    dry_run: bool,

    /// Instead of proving, survey a requester's recent fulfilled requests on
    /// the network and write them out. Takes a hex address.
    #[clap(long)]
    survey: Option<String>,

    /// How many requests to pull when surveying.
    #[clap(long, default_value = "100")]
    survey_limit: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Compressed,
    Plonk,
    Groth16,
}

impl Mode {
    const ALL: [Mode; 3] = [Mode::Compressed, Mode::Plonk, Mode::Groth16];

    fn sp1(self) -> SP1ProofMode {
        match self {
            Mode::Compressed => SP1ProofMode::Compressed,
            Mode::Plonk => SP1ProofMode::Plonk,
            Mode::Groth16 => SP1ProofMode::Groth16,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Mode::Compressed => "compressed",
            Mode::Plonk => "plonk",
            Mode::Groth16 => "groth16",
        }
    }
}

/// One ELF plus the stdin it expects.
struct Workload {
    name: &'static str,
    elf: Elf,
    stdin: SP1Stdin,
    local_cycles: u64,
    local_gas: Option<u64>,
}

/// One measured request.
#[derive(Serialize, Clone, Debug)]
struct Run {
    workload: String,
    mode: String,
    rep: usize,
    started_unix: u64,
    request_id: String,
    request_url: String,
    ok: bool,
    error: Option<String>,
    /// `fulfilled_at - created_at` from the network. The primary metric: it
    /// excludes all client-side work (ELF upload, local simulation, polling).
    server_latency_s: Option<f64>,
    /// Client wall time from request submission to observing `Fulfilled`.
    client_wall_s: f64,
    /// Client-side cost of `request()` itself, including local simulation.
    submit_s: f64,
    /// Submission to first observed `Assigned` (2s poll granularity).
    queue_s: Option<f64>,
    /// First observed `Assigned` to `Fulfilled` (2s poll granularity).
    prove_s: Option<f64>,
    created_at: Option<u64>,
    fulfilled_at: Option<u64>,
    cycles: Option<u64>,
    gas_used: Option<u64>,
    cost_prove: Option<String>,
    fulfiller: Option<String>,
    /// Raw `FulfillmentStrategy` the network recorded. Must be 2 (RESERVED).
    strategy: Option<i32>,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn sample_events(n: usize) -> Vec<(TokenInfo, U256)> {
    data::sample_bridge_exits_01()
        .cycle()
        .take(n)
        .map(|e| (e.token_info, e.amount))
        .collect()
}

/// Builds the pp stdin exactly the way `ppgen` does.
fn pp_stdin(n_exits: usize, n_imported_exits: usize) -> SP1Stdin {
    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();

    let bridge_exits = sample_events(n_exits);
    let imported_bridge_exits = sample_events(n_imported_exits);
    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    let multi_batch_header = old_state
        .make_multi_batch_header(
            &certificate,
            L1WitnessCtx {
                l1_info_root: certificate.l1_info_root().unwrap().unwrap_or_default(),
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: state.get_signer(),
                },
            },
        )
        .expect("failed to build the multi batch header");

    Runner::prepare_stdin(&old_state.into(), &multi_batch_header)
}

/// Submits one request and follows it to a terminal state.
async fn measure(
    prover: &NetworkProver,
    pk: &sp1_sdk::SP1ProvingKey,
    workload: &str,
    stdin: SP1Stdin,
    mode: Mode,
    rep: usize,
    timeout: Duration,
) -> Run {
    let started_unix = now_unix();
    let mut run = Run {
        workload: workload.to_owned(),
        mode: mode.name().to_owned(),
        rep,
        started_unix,
        request_id: String::new(),
        request_url: String::new(),
        ok: false,
        error: None,
        server_latency_s: None,
        client_wall_s: 0.0,
        submit_s: 0.0,
        queue_s: None,
        prove_s: None,
        created_at: None,
        fulfilled_at: None,
        cycles: None,
        gas_used: None,
        cost_prove: None,
        fulfiller: None,
        strategy: None,
    };

    let submit_start = Instant::now();
    let request_id: B256 = match prover
        .prove(pk, stdin)
        .mode(mode.sp1())
        .strategy(FulfillmentStrategy::Reserved)
        .timeout(timeout)
        .request()
        .await
    {
        Ok(id) => id,
        Err(e) => {
            run.submit_s = submit_start.elapsed().as_secs_f64();
            run.error = Some(format!("submit failed: {e}"));
            warn!("{} {} rep{}: {}", workload, mode.name(), rep, run.error.as_ref().unwrap());
            return run;
        }
    };
    run.submit_s = submit_start.elapsed().as_secs_f64();
    run.request_id = format!("{request_id}");
    run.request_url = format!("{RESERVED_EXPLORER}/request/{request_id}");
    info!(
        "{} {} rep{}: {}",
        workload,
        mode.name(),
        rep,
        run.request_url
    );

    let watch_start = Instant::now();
    let mut assigned: Option<f64> = None;
    loop {
        if watch_start.elapsed() > timeout {
            run.error = Some(format!("timed out after {}s", timeout.as_secs()));
            break;
        }

        let status = match prover.get_proof_status(request_id).await {
            Ok((status, _)) => status,
            Err(e) => {
                // A transient status error should not kill the cell.
                warn!("status poll failed, retrying: {e}");
                tokio::time::sleep(POLL_INTERVAL).await;
                continue;
            }
        };

        match FulfillmentStatus::try_from(status.fulfillment_status()) {
            Ok(FulfillmentStatus::Fulfilled) => {
                let elapsed = watch_start.elapsed().as_secs_f64();
                run.client_wall_s = elapsed;
                run.queue_s = assigned;
                run.prove_s = assigned.map(|a| elapsed - a);
                run.ok = true;
                break;
            }
            Ok(FulfillmentStatus::Assigned) => {
                if assigned.is_none() {
                    assigned = Some(watch_start.elapsed().as_secs_f64());
                }
            }
            Ok(
                terminal @ (FulfillmentStatus::Unfulfillable
                | FulfillmentStatus::Reverted
                | FulfillmentStatus::Expired),
            ) => {
                run.error = Some(format!("terminal status {terminal:?}"));
                break;
            }
            _ => {}
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }

    if run.client_wall_s == 0.0 {
        run.client_wall_s = watch_start.elapsed().as_secs_f64();
    }

    // Server-side numbers are authoritative; fetch them even on failure.
    match prover.get_proof_request(request_id).await {
        Ok(Some(details)) => {
            run.created_at = Some(details.created_at);
            run.fulfilled_at = details.fulfilled_at;
            run.server_latency_s = details
                .fulfilled_at
                .map(|f| f.saturating_sub(details.created_at) as f64);
            run.cycles = details.cycles;
            run.gas_used = details.gas_used;
            run.cost_prove = details.deduction_amount;
            run.fulfiller = details.fulfiller_name;
            run.strategy = Some(details.strategy);
        }
        Ok(None) => warn!("no request details for {request_id}"),
        Err(e) => warn!("failed to fetch request details: {e}"),
    }

    if let Some(e) = &run.error {
        warn!("{} {} rep{}: {}", workload, mode.name(), rep, e);
    } else {
        info!(
            "{} {} rep{}: server {:?}s (queue {:?}s), {:?} cycles",
            workload,
            mode.name(),
            rep,
            run.server_latency_s,
            run.queue_s,
            run.cycles
        );
    }

    run
}

/// One fulfilled request as the network reports it.
#[derive(Serialize)]
struct SurveyRow {
    request_id: String,
    mode: String,
    strategy: i32,
    created_at: u64,
    fulfilled_at: Option<u64>,
    latency_s: Option<f64>,
    cycles: Option<u64>,
    gas_used: Option<u64>,
    cost_prove: Option<String>,
    vk_hash: String,
    program_name: Option<String>,
    requester: String,
    fulfillment_status: i32,
}

/// Pulls a requester's recent fulfilled requests straight from the network, so
/// the production figures in the report are not hand-copied from the explorer.
async fn survey(
    signer: NetworkSigner,
    rpc_url: &str,
    addr: &str,
    limit: u32,
    out_dir: &PathBuf,
) -> eyre::Result<()> {
    let requester = if addr == "any" {
        None
    } else {
        Some(
            hex::decode(addr.trim_start_matches("0x"))
                .map_err(|e| eyre::eyre!("bad requester address {addr}: {e}"))?,
        )
    };

    let client = NetworkClient::new(signer, rpc_url, NetworkMode::Reserved);
    let response = client
        .get_filtered_proof_requests(
            None,
            None,
            None,
            None,
            None,
            requester,
            None,
            None,
            None,
            Some(limit),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| eyre::eyre!("survey request failed: {e}"))?;

    // The two network modes return structurally identical but distinct types.
    macro_rules! rows {
        ($reqs:expr) => {
            $reqs
                .iter()
                .map(|r| SurveyRow {
                    request_id: format!("0x{}", hex::encode(&r.request_id)),
                    mode: ProofMode::try_from(r.mode)
                        .map(|m| format!("{m:?}").to_lowercase())
                        .unwrap_or_else(|_| format!("unknown({})", r.mode)),
                    strategy: r.strategy,
                    created_at: r.created_at,
                    fulfilled_at: r.fulfilled_at,
                    latency_s: r.fulfilled_at.map(|f| f.saturating_sub(r.created_at) as f64),
                    cycles: r.cycles,
                    gas_used: r.gas_used,
                    cost_prove: r.deduction_amount.clone(),
                    vk_hash: format!("0x{}", hex::encode(&r.vk_hash)),
                    program_name: r.program_name.clone(),
                    requester: format!("0x{}", hex::encode(&r.requester)),
                    fulfillment_status: r.fulfillment_status,
                })
                .collect::<Vec<_>>()
        };
    }
    let rows: Vec<SurveyRow> = match response {
        GetFilteredProofRequestsResponse::Auction(r) => rows!(r.requests),
        GetFilteredProofRequestsResponse::Base(r) => rows!(r.requests),
    };

    let mut by_status: BTreeMap<i32, usize> = BTreeMap::new();
    let mut by_requester: BTreeMap<String, usize> = BTreeMap::new();
    for r in &rows {
        *by_status.entry(r.fulfillment_status).or_default() += 1;
        *by_requester.entry(r.requester.clone()).or_default() += 1;
    }
    info!("surveyed {} requests for {}", rows.len(), addr);
    info!("  by fulfillment_status: {:?}", by_status);
    info!("  distinct requesters: {}", by_requester.len());
    for (who, n) in by_requester.iter().take(5) {
        info!("    {who} -> {n}");
    }

    let payload = serde_json::json!({
        "survey": {
            "requester": addr,
            "rpc_url": rpc_url,
            "limit": limit,
            "fetched": rows.len(),
            "fetched_at_unix": now_unix(),
        },
        "requests": rows,
    });

    fs::create_dir_all(out_dir)?;
    let path = out_dir.join(format!("survey-{}.json", chrono_stamp(now_unix())));
    fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
    info!("wrote {}", path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();
    let timeout = Duration::from_secs(args.timeout_secs);

    let private_key = std::env::var("NETWORK_PRIVATE_KEY").map_err(|_| {
        eyre::eyre!("NETWORK_PRIVATE_KEY is not set; source ~/.ssh/.env.succinct first")
    })?;
    // The reserved-capacity endpoint. Pinned rather than required from the
    // environment so a key-only env file is enough to run this.
    let rpc_url =
        std::env::var("NETWORK_RPC_URL").unwrap_or_else(|_| RESERVED_RPC_URL.to_owned());

    // Pin NetworkMode::Reserved explicitly rather than relying on the
    // `reserved-capacity` cargo feature to pick the right default.
    let signer = NetworkSigner::local(&private_key)
        .map_err(|e| eyre::eyre!("bad NETWORK_PRIVATE_KEY: {e}"))?;
    let prover = NetworkProver::new(signer, &rpc_url, NetworkMode::Reserved).await;
    info!("network mode {:?} against {}", prover.network_mode(), rpc_url);

    if let Some(addr) = args.survey.clone() {
        // Building the prover above installed the rustls crypto provider that
        // NetworkClient needs, so the survey has to come after it.
        let signer = NetworkSigner::local(&private_key)
            .map_err(|e| eyre::eyre!("bad NETWORK_PRIVATE_KEY: {e}"))?;
        return survey(signer, &rpc_url, &addr, args.survey_limit, &args.out_dir).await;
    }

    let selected: Vec<&str> = args.workloads.split(',').map(str::trim).collect();
    let mut workloads = Vec::new();
    for name in ["noop", "pp", "pp-large"] {
        if !selected.contains(&name) {
            continue;
        }
        let (elf, stdin) = match name {
            "noop" => (Elf::Static(NOOP_ELF), SP1Stdin::new()),
            "pp" => (
                Elf::Static(PESSIMISTIC_PROOF_ELF),
                pp_stdin(args.n_exits, args.n_imported_exits),
            ),
            _ => (
                Elf::Static(PESSIMISTIC_PROOF_ELF),
                pp_stdin(args.n_exits_large, args.n_imported_exits_large),
            ),
        };
        workloads.push(Workload {
            name,
            elf,
            stdin,
            local_cycles: 0,
            local_gas: None,
        });
    }
    eyre::ensure!(!workloads.is_empty(), "no known workload in --workloads");

    let selected_modes: Vec<Mode> = Mode::ALL
        .into_iter()
        .filter(|m| args.modes.split(',').map(str::trim).any(|s| s == m.name()))
        .collect();
    eyre::ensure!(!selected_modes.is_empty(), "no known mode in --modes");


    // Local execute: proves the noop ELF really is a runnable noop, and gives
    // the cycle counts the report leans on.
    for w in workloads.iter_mut() {
        let (_public_values, report) = prover
            .execute(w.elf.clone(), w.stdin.clone())
            .await
            .map_err(|e| eyre::eyre!("local execute of {} failed: {e}", w.name))?;
        w.local_cycles = report.total_instruction_count();
        w.local_gas = report.gas();
        info!(
            "{}: {} cycles, {:?} gas locally",
            w.name, w.local_cycles, w.local_gas
        );
    }

    // Register both programs up front so no measured request pays for an
    // ELF upload.
    let mut keys = Vec::new();
    for w in &workloads {
        let pk = prover
            .setup(w.elf.clone())
            .await
            .map_err(|e| eyre::eyre!("setup of {} failed: {e}", w.name))?;
        info!("{}: vkey {}", w.name, pk.verifying_key().bytes32());
        keys.push(pk);
    }

    if args.dry_run {
        info!("dry run: local path is good, nothing submitted to the network");
        return Ok(());
    }

    if !args.no_warmup && selected_modes.contains(&Mode::Compressed) {
        info!("warm-up: one discarded compressed request per ELF");
        for (i, w) in workloads.iter().enumerate() {
            let run = measure(
                &prover,
                &keys[i],
                w.name,
                w.stdin.clone(),
                Mode::Compressed,
                0,
                timeout,
            )
            .await;
            if !run.ok {
                eyre::bail!(
                    "warm-up for {} did not reach Fulfilled ({:?}). Reserved capacity may not \
                     be provisioned for this account — stopping rather than silently falling \
                     back to Hosted, which would change what is being measured. Request: {}",
                    w.name,
                    run.error,
                    run.request_url
                );
            }
            if run.strategy != Some(FulfillmentStrategy::Reserved as i32) {
                eyre::bail!(
                    "warm-up for {} recorded strategy {:?}, expected {} (RESERVED). Request: {}",
                    w.name,
                    run.strategy,
                    FulfillmentStrategy::Reserved as i32,
                    run.request_url
                );
            }
            info!("warm-up {} ok, strategy RESERVED confirmed", w.name);
        }
    }

    // Round-robin across cells so repeats of a cell are separated in time, and
    // reverse the order on alternate reps so position cannot alias with drift.
    let mut runs: Vec<Run> = Vec::new();
    for rep in 1..=args.reps {
        let mut cells: Vec<(usize, Mode)> = workloads
            .iter()
            .enumerate()
            .flat_map(|(i, _)| selected_modes.clone().into_iter().map(move |m| (i, m)))
            .collect();
        if rep % 2 == 0 {
            cells.reverse();
        }
        for (i, mode) in cells {
            let w = &workloads[i];
            runs.push(
                measure(&prover, &keys[i], w.name, w.stdin.clone(), mode, rep, timeout).await,
            );
        }
    }

    // Adaptive extra rep where the baseline samples disagree.
    let mut extra: Vec<(usize, Mode)> = Vec::new();
    for (i, w) in workloads.iter().enumerate() {
        for mode in selected_modes.iter().copied() {
            let mut samples: Vec<f64> = runs
                .iter()
                .filter(|r| r.workload == w.name && r.mode == mode.name() && r.ok)
                .filter_map(|r| r.server_latency_s)
                .collect();
            samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let spread = match (samples.first(), samples.last()) {
                (Some(&lo), Some(&hi)) if lo > 0.0 => (hi - lo) / lo,
                // A cell with fewer than two usable samples also earns a retry.
                _ => f64::INFINITY,
            };
            if samples.len() < 2 || spread > args.adaptive_threshold {
                info!(
                    "{} {}: spread {:.2} over {} samples, adding a rep",
                    w.name,
                    mode.name(),
                    spread,
                    samples.len()
                );
                extra.push((i, mode));
            }
        }
    }
    for (i, mode) in extra {
        let w = &workloads[i];
        runs.push(
            measure(
                &prover,
                &keys[i],
                w.name,
                w.stdin.clone(),
                mode,
                args.reps + 1,
                timeout,
            )
            .await,
        );
    }

    let payload = serde_json::json!({
        "parameters": {
            "n_exits": args.n_exits,
            "n_imported_exits": args.n_imported_exits,
            "reps": args.reps,
            "modes": args.modes,
            "workloads_run": args.workloads,
            "adaptive_threshold": args.adaptive_threshold,
            "timeout_secs": args.timeout_secs,
            "strategy": "RESERVED",
            "network_mode": "Reserved",
            "rpc_url": rpc_url,
            "circuit_version": sp1_sdk::SP1_CIRCUIT_VERSION,
            "sequential": true,
            "warmup_discarded": !args.no_warmup,
        },
        "workloads": workloads.iter().map(|w| serde_json::json!({
            "name": w.name,
            "local_cycles": w.local_cycles,
            "local_gas": w.local_gas,
            "elf_bytes": match &w.elf { Elf::Static(b) => b.len(), _ => 0 },
            "exits": match w.name {
                "noop" => None,
                "pp" => Some([args.n_exits, args.n_imported_exits]),
                _ => Some([args.n_exits_large, args.n_imported_exits_large]),
            },
        })).collect::<Vec<_>>(),
        "runs": runs,
    });

    fs::create_dir_all(&args.out_dir)?;
    let stamp = chrono_stamp(now_unix());
    let json_path = args.out_dir.join(format!("{stamp}.json"));
    fs::write(&json_path, serde_json::to_string_pretty(&payload)?)?;
    info!("wrote {}", json_path.display());

    let failures = runs.iter().filter(|r| !r.ok).count();
    info!(
        "{} runs, {} failed. Raw data in {}",
        runs.len(),
        failures,
        json_path.display()
    );

    Ok(())
}

/// Minimal UTC `YYYY-MM-DDTHH-MM-SSZ` stamp from a unix timestamp.
fn chrono_stamp(secs: u64) -> String {
    let days = secs / 86_400;
    let tod = secs % 86_400;
    let (h, m, s) = (tod / 3600, (tod % 3600) / 60, tod % 60);

    // Civil-from-days, Howard Hinnant's algorithm.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{y:04}-{mo:02}-{d:02}T{h:02}-{m:02}-{s:02}Z")
}
