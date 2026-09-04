# SP1 proving latency for the pessimistic proof

> **These measurements suggest we are mostly bound by the SNARK wrap, not by the
> PP or by how much work it does.**
>
> If that holds, benchmarking effort is probably better spent on the wrap than
> on the PP program.

## Measurements

Rough averages, on SP1 reserved capacity.

| program                        | cycles        | STARK   | Groth16 | PLONK    |
| ------------------------------ | ------------- | ------- | ------- | -------- |
| empty program                  | 4.6k          | ~30s    | ~50s    | ~2min45s |
| PP, from 1 + 1 to 100 + 100    | 1.1M - 32.9M  | ~30-35s | ~45s    | ~2min50s |
| PP in production, 99 proofs    | 0.8M - 6.7M   |         |         | ~3min    |

The first two rows are ours, n=6 to 19 per cell. The last row is 99 real
fulfilled requests over 75 hours, PLONK only, where the load varies on its own
rather than because we set it.

An empty program takes about as long as the PP does, at any load we tried, and
production sits in the same place. Most of the time appears to go into the wrap.

## What that suggests

SP1 shards the execution and proves the shards in parallel, so wall clock
follows the critical path rather than the total work. Succinct gas, which does
track total work, varies much more than latency does.

So if we want to compare proving performance, the number that likely matters is
how long the SNARK wrap takes, not how much the PP computes.

## Caveats

- The first three rows are thin: n=6 to 10 per program, run quickly. Treat those
  figures as rough. The production row is the sturdier of the two.
- This is about the PP, not about guest programs in general.
- This is latency only. Cost, through Succinct gas, does still follow the PP
  load.

## Reproduce

You need an SP1 network key.

```sh
export NETWORK_PRIVATE_KEY=<key>
cargo build --release -p pessimistic-proof-test-suite --bin sp1bench
```

Run the synthetic matrix, or pull a requester's recent production requests:

```sh
target/release/sp1bench --out-dir docs/benchmarks/sp1-proving-latency/data
target/release/sp1bench --survey 0x<address> --survey-limit 100 \
  --out-dir docs/benchmarks/sp1-proving-latency/data
```

Then rebuild the report from whatever is in `data/`:

```sh
cd docs/benchmarks/sp1-proving-latency
python3 sp1_report.py data/*.json > report.md
```

Add `--dry-run` to check the local path without submitting anything. See
`--help` for the workload, mode and repetition flags.

[report.md](report.md) has every request URL and the full per-cell statistics.
Source: `crates/pessimistic-proof-test-suite/src/bin/sp1bench.rs`.
