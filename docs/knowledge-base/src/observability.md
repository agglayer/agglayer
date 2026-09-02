# Observability

Agglayer exposes Prometheus metrics over the `/metrics` HTTP endpoint served by
the `agglayer-telemetry` crate (OpenTelemetry → `opentelemetry-prometheus`). The
listen address is configured under `[telemetry]` (`prometheus-addr`, default
`0.0.0.0:3000`).

This page documents two metric families:
certificate bridging times, and the backup subsystem.
Per-rollup pending/proven/settled height gauges and certificate status/error
counters are tracked separately (issues #1352 and #1655),
as are the settlement metrics.

## Certificate bridging-time metrics

Both metrics use the OpenTelemetry meter scope `agglayer_node_certificate` and are
labeled by `network_id`; the per-stage histogram also carries a `stage` label.

| Metric | Type | Labels | Meaning |
| --- | --- | --- | --- |
| `agglayer_certificate_duration_seconds` | histogram | `network_id` | Total end-to-end bridging time of a certificate (`Pending` → `Settled`). |
| `agglayer_certificate_stage_duration_seconds` | histogram | `network_id`, `stage` | Time spent in each lifecycle stage. |

### Stages

The `stage` label on `agglayer_certificate_stage_duration_seconds` is the name of
the non-terminal certificate status being timed — the time a certificate spends
in that state before its next transition:

| `stage` | State timed | Ends at | Covers |
| --- | --- | --- | --- |
| `pending` | `Pending` | `Proven` | Proof generation (certification). |
| `proven` | `Proven` | `Candidate` | Building the calldata and submitting the settlement job (includes L1 `estimateGas`). |
| `candidate` | `Candidate` | `Settled` | L1 inclusion and confirmation wait. |

The three stages are contiguous, so their durations sum to
`agglayer_certificate_duration_seconds` for a given certificate.

### Histogram buckets

Both histograms share one bucket set (seconds), covering sub-second stages
through multi-minute settlement waits:

```text
0.5, 1, 2.5, 5, 10, 30, 60, 120, 300, 600, 900, 1800
```

## Semantics and caveats

- **In-process, no persistence.** Durations are measured with in-memory timers on
  the certificate task. They are **not** persisted, so counts reset when the node
  restarts.
- **Fresh certificates only.** The two duration histograms are recorded only for
  certificates the task observes from `Pending` through `Settled` within a single
  process lifetime. Certificates resumed after a restart (entering as `Proven` or
  `Candidate`) contribute no durations, which keeps end-to-end and per-stage
  distributions honest.
- **Each stage records on completion.** A certificate that errors mid-lifecycle
  still contributes the stages that finished; only the total requires reaching
  `Settled`.
- **Queue wait excluded.** Timing starts when the certificate task begins
  processing, not at RPC receipt. Time spent waiting in the pending queue before
  pickup (usually small) is not included. A true wall-clock receipt→settled metric
  that survives restarts is a possible follow-up.

## Example PromQL

End-to-end p95 bridging time, per network:

```promql
histogram_quantile(
  0.95,
  sum by (le, network_id) (
    rate(agglayer_certificate_duration_seconds_bucket[$__rate_interval])
  )
)
```

Median time per stage:

```promql
histogram_quantile(
  0.5,
  sum by (le, stage) (
    rate(agglayer_certificate_stage_duration_seconds_bucket[$__rate_interval])
  )
)
```

## Backup metrics

The backup subsystem copies three RocksDB databases off-box.
All series use the OpenTelemetry meter scope `agglayer_node_backup`.

| Metric | Type | Labels | Meaning |
| --- | --- | --- | --- |
| `agglayer_node_backup_requests_total` | counter | `queue`, `disposition` | Requests raised. |
| `agglayer_node_backup_queue_wait_seconds` | histogram | `queue` | Wait before a request was picked up. |
| `agglayer_node_backup_duration_seconds` | histogram | `queue`, `outcome` | Time the backup itself took. |
| `agglayer_node_backup_outstanding_age_seconds` | gauge | `queue` | Age of the request being served; `0` when idle. |
| `agglayer_node_backup_last_success_timestamp_seconds` | gauge | `queue` | Unix time of the last successful backup. |
| `agglayer_node_backup_files` | gauge | `db` | Files in the last successful backup. |

| Label | Values |
| --- | --- |
| `queue` | `state`, `epoch` |
| `db` | `state`, `pending`, `epoch` |
| `disposition` | `queued`, `coalesced`, `rejected` |
| `outcome` | `success`, `failure` |

A `queue` is a unit of work, a `db` is a database.
One `queue="state"` request backs up two databases,
`db="state"` and `db="pending"`. Each epoch has its own.

Both histograms use these buckets, in seconds.
They are finer at the bottom than the certificate ones
because an idle engine serves a request almost immediately:

```text
0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10, 30, 60, 120, 300, 600, 1800
```

### Reading them

- `coalesced` is normal, not a loss.
  The queued backup also covers the write whose request was coalesced.
  A high rate means the engine is not keeping up.
- `rejected` means the queue is closed.
  It fires for the whole shutdown drain, which can last minutes,
  so alert only when it persists outside a restart.
- No backup series at all means backups are disabled.
  An engine that runs and fails still exports, with `outcome="failure"` rising.
- A backup that starts and never finishes shows only on
  `outstanding_age_seconds`, which climbs while `duration_seconds` goes quiet.
  Alert on the gauge, not on missing durations.
- That gauge covers the request being *served*, not queued ones.
  Epoch backlog is `requests_total{queue="epoch"}` minus
  `duration_seconds_count{queue="epoch"}`.
- `files` is why a backup is slow: the engine checks every file it references
  against the backup filesystem before copying anything.
- A failed `purge_old_backups` has no series.
  It leaves the new backup valid, so watch the
  `Failed to purge old backup` log line for backups piling up.
- Nothing here survives a restart.

### Queries

```promql
# p95 backup time and p95 queue wait, per queue
histogram_quantile(0.95, sum by (le, queue) (
  rate(agglayer_node_backup_duration_seconds_bucket[$__rate_interval])))
histogram_quantile(0.95, sum by (le, queue) (
  rate(agglayer_node_backup_queue_wait_seconds_bucket[$__rate_interval])))

# how stale the newest backup is
time() - agglayer_node_backup_last_success_timestamp_seconds

# epoch requests queued or running, since that queue is unbounded
sum(agglayer_node_backup_requests_total{queue="epoch", disposition="queued"})
- sum(agglayer_node_backup_duration_seconds_count{queue="epoch"})
```

## Configuration

The metrics endpoint address is configured under `[telemetry]`
(`prometheus-addr`, default `0.0.0.0:3000`). Deployment-level labels such as
`environment` or `cluster` are expected to be added at scrape time via Prometheus
`external_labels` rather than emitted by the node.

## Extending

Certificate metrics are defined in `crates/agglayer-telemetry/src/certificate.rs`
and emitted only through its `record_*` helpers, which build the shared label set.
Adding a metric (for example an RPC-path latency histogram) is one instrument plus
one helper there and its call site in `agglayer-certificate-orchestrator`; adding
or splitting a stage is a new `stage` constant plus a record call at the transition.
Bucket boundaries and stage names are constants at the top of that module and can
be tuned once real distributions are observed.

Backup metrics live entirely in `crates/agglayer-telemetry/src/backup.rs`;
`agglayer-storage` holds an `Arc<BackupMetrics>` and reports lifecycle events
through it.
