# Pessimistic Proof Generator

This utility generates pessimistic proofs on sample data.

```
cargo run -r -p pessimistic-proof-test-suite --bin ppgen -- --help
```

## Local mode

The following command will generate one plonk proof for 10 bridge exits, with the default mode which is local.

```
RUST_LOG=info ./target/release/ppgen --proof-dir ./data/proofs/ --n-exits 10
```

The local mode requires a large machine with ~128GB RAM.

## Network mode

Necessary environment variables to generate with the succinct infrastructure:

```
SP1_PROVER=network
SP1_PRIVATE_KEY=...
```

Then, the command remains the same:

```
RUST_LOG=info ./target/release/ppgen --proof-dir ./data/proofs/ --n-exits 10
```

Expected logs:

```
2024-08-30T13:02:29.261953Z  INFO Generating the proof for 10 bridge exits
2024-08-30T13:02:29.261973Z  INFO Client circuit version: v1.1.0
2024-08-30T13:02:37.100401Z  WARN network_prover: close time.busy=3.75µs time.idle=2.72µs
2024-08-30T13:02:37.111616Z  INFO execute: clk = 0 pc = 0x209758
2024-08-30T13:02:37.277199Z  INFO execute: close time.busy=176ms time.idle=1.57µs
2024-08-30T13:02:37.277218Z  INFO Simulation complete, cycles: 1264128
2024-08-30T13:02:45.374985Z  INFO Created proofrequest_01j6hp2xm7fpjtc6depe2sfa9s
2024-08-30T13:02:45.375028Z  INFO View in explorer: https://explorer.succinct.xyz/proofrequest_01j6hp2xm7fpjtc6depe2sfa9s
2024-08-30T13:02:48.651517Z  INFO Proof request claimed, proving...
2024-08-30T13:05:57.296735Z  INFO Proof request fulfilled
2024-08-30T13:05:58.374793Z  INFO Successfully generated the plonk proof
2024-08-30T13:05:58.374934Z  INFO Writing the proof to "./agglayer/crates/pessimistic-proof-test-suite/./data/proofs/10-exits-v0x0072d7-4c049331-cde2-4a82-84f9-c720d89b1752.json"
```
