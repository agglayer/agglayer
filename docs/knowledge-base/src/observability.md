# Observability

Agglayer exposes Prometheus metrics over the `/metrics` HTTP endpoint served by
the `agglayer-telemetry` crate (OpenTelemetry → `opentelemetry-prometheus`). The
listen address is configured under `[telemetry]` (`prometheus-addr`, default
`0.0.0.0:3000`).

This page documents Agglayer's certificate bridging-time metrics: the end-to-end
duration histogram and a per-stage breakdown (the project's first histograms).
Per-rollup pending/proven/settled height gauges and certificate status/error
counters are tracked separately (issues #1352 and #1655).

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
