# Settlement Latest Attempt Benchmark

This benchmark compares two read strategies for "latest attempt per job":

- `strategy_prefix_scan`: seek on the attempts CF keyspace
- `strategy_latest_cf`: direct lookup in a dedicated latest-attempt CF

It uses realistic settlement workload assumptions:

- attempts per job in `[1..25]`
- distribution: 70% (1-3), 25% (4-10), 5% (11-25)
- datasets: 1k, 10k, 50k jobs
- workloads: write-only, write-heavy 90/10, read-hit, read-miss

## Run benchmark

```bash
cargo bench -p agglayer-storage --features testutils --bench settlement_latest_attempt_bench
```

Criterion writes HTML and JSON reports under:

- `target/criterion/settlement_latest/**`
- `target/criterion/report/index.html`

Use the report index page to compare medians, confidence intervals, and
distribution charts for each dataset/workload/strategy combination.

## Compare against a baseline

Use Criterion baselines instead of custom extraction scripts:

```bash
# Baseline run (for example on main)
cargo bench -p agglayer-storage --features testutils --bench settlement_latest_attempt_bench -- --save-baseline settlement-latest-main

# Candidate run (current branch)
cargo bench -p agglayer-storage --features testutils --bench settlement_latest_attempt_bench -- --baseline settlement-latest-main
```

Criterion will include relative change and significance directly in its output
and generated report pages.

## Optional artifact layout

Use this folder structure to keep benchmark outputs grouped and easy to compare:

```text
crates/agglayer-storage/benches/artifacts/settlement_latest_attempt/
  raw/
    criterion/                # optional copy of target/criterion subset
```

Benchmark artifacts under `benches/artifacts/` are intentionally git-ignored.
Keep benchmark code and this README in version control.

## Decision threshold

- Prioritize steady-state writes.
- Accept `strategy_latest_cf` only if write penalty is `<= 5%` and read p95 latency gain is meaningful.
