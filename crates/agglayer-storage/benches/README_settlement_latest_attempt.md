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

## Result artifact layout

Use this folder structure to keep benchmark outputs grouped and easy to compare:

```text
crates/agglayer-storage/benches/artifacts/settlement_latest_attempt/
  raw/
    criterion/                # copy target/criterion subset here
  reports/
    results.csv               # fill from extracted benchmark metrics
    throughput_ops_per_s.png
    latency_panel_us.png
    db_size_mb.png
    db_size_delta_pct.png
    executive_summary.png
```

You can start from the template CSV:

`crates/agglayer-storage/benches/settlement_latest_attempt_results_template.csv`

## Generate graphs from CSV

Create a local virtualenv for plotting dependencies:

```bash
python3 -m venv .venv-bench
.venv-bench/bin/pip install matplotlib
```

Extract CSV from Criterion outputs:

```bash
python3 crates/agglayer-storage/benches/extract_settlement_latest_attempt_results.py \
  --criterion-root target/criterion \
  --out-csv crates/agglayer-storage/benches/artifacts/settlement_latest_attempt/reports/results.csv
```

Generate plots from the CSV:

```bash
.venv-bench/bin/python crates/agglayer-storage/benches/plot_settlement_latest_attempt.py \
  --csv crates/agglayer-storage/benches/artifacts/settlement_latest_attempt/reports/results.csv \
  --out-dir crates/agglayer-storage/benches/artifacts/settlement_latest_attempt/reports
```

## Decision threshold

- Prioritize steady-state writes.
- Accept `strategy_latest_cf` only if write penalty is `<= 5%` and read p95 latency gain is meaningful.
