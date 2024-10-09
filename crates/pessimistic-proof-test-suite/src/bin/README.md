# Pessimistic Proof Generator

This utility generates pessimistic proofs on sample data.

```
cargo run -r -p pessimistic-proof-test-suite --bin ppgen -- --help
```

## Local mode

The following command will generate one plonk proof with the default mode which is local.

```
RUST_LOG=info cargo run -r -p pessimistic-proof-test-suite --bin ppgen -- --proof-dir ./data/proofs/ --n-exits 10
```

The local mode requires a large machine with ~128GB RAM.

### Bridge exits and Imported bridge exits

The transition is given by:

- The number of bridge exits: `--n-exits <number>`
- The number of imported bridge exits: `--n-imported-exits <number>`

It will generate one Certificate with `--n-exits` brige exits and `--n-imported-exits` imported bridge exits.

The events are cyclically taken from the optional sample file given by `--sample-path`.
If not provided, this default sample file of 200 events is used: [withdrawals.json](../../data/withdrawals.json)

## Network mode

The succinct infrastructure generates the proof upon request. This requires to define two environment variables:

```
SP1_PROVER=network
SP1_PRIVATE_KEY=...
```

Then, the command remains the same:

```
RUST_LOG=info cargo run -r -p pessimistic-proof-test-suite --bin ppgen -- --proof-dir ./data/proofs/ --n-exits 10
```

Expected logs:

```
2024-10-01T10:30:41.580433Z  INFO Certificate: [{"network_id":1,"height":0,"prev_local_exit_root":[39,174,91,160,141,114, ...
2024-10-01T10:30:41.580860Z  INFO Generating the proof for 1 bridge exits
2024-10-01T10:30:41.580881Z  INFO Client circuit version: v2.0.0
2024-10-01T10:30:56.775226Z  WARN network_prover: close time.busy=72.9ms time.idle=3.57µs
2024-10-01T10:30:56.776023Z  INFO Skipping simulation
2024-10-01T10:31:05.958647Z  INFO Created proofrequest_01j93t47b4e8dv7kgcm0a02dra
2024-10-01T10:31:05.958692Z  INFO View in explorer: https://explorer.succinct.xyz/proofrequest_01j93t47b4e8dv7kgcm0a02dra
2024-10-01T10:31:10.517512Z  INFO Proof request claimed, proving...
2024-10-01T10:34:23.797319Z  INFO Proof request fulfilled
2024-10-01T10:34:24.767493Z  INFO Successfully generated the plonk proof with a latency of 223.186621825s
2024-10-01T10:34:24.767799Z  INFO Writing the proof to "./data/proofs/1-exits-v0x00c745-b99b2bf1-c58c-4808-8b3a-7548d13a151d.json"
```

The first line in the logs is the full Certificate which is trimmed in this example.

The execution outputs a json file which contains the following:

- The certificate
- The signer address
- The public inputs of the pessimistic proof in clear and serialized for the verifier
- The SNARK pessimistic proof
- The verifier key

## Proof output

Use `--proof-dir` to save the proof as a JSON file in the specified directory. If not set, the proof will be logged instead.
