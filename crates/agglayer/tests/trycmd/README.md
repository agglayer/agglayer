# Literate test cases for the agglayer CLI

This file contains UI tests in code blocks.

## Quickstart
```bash
cargo test --test cli # run the tests
TRYCMD=overwrite cargo test --test cli # overwrite the snapshots
TRYCMD=dump cargo test --test cli # dump the snapshots
```

Trycmd can additionally check exit statuses, clean local paths, assert file contents etc.
See the [crate documentation](https://docs.rs/trycmd) for more.

```console
$ agglayer --help
Agglayer command line interface

Usage: agglayer <COMMAND>

Commands:
  run              
  config           
  validate-config  
  prover-config    
  prover           
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

```

```console
$ agglayer config --base-dir .
prover-entrypoint = "http://127.0.0.1:8080"

[full-node-rpcs]

[l2]
rpc-timeout = "45s"

[proof-signers]

[log]
level = "info"
outputs = []
format = "pretty"

[rpc]
port = 9090
host = "0.0.0.0"
request-timeout = "3m"

[rate-limiting]
send-tx = "unlimited"

[rate-limiting.network]

[outbound.rpc.settle]
max-retries = 3
retry-interval = "7s"
confirmations = 1
settlement-timeout = "20m"

[l1]
chain-id = 1337
node-url = "http://zkevm-mock-l1-network:8545/"
ws-node-url = "ws://zkevm-mock-l1-network:8546/"
max-reconnection-elapsed-time = "10s"
rollup-manager-contract = "0xb7f8bc63bbcad18155201308c8f3540b07f84f5e"
polygon-zkevm-global-exit-root-v2-contract = "0xb7f8bc63bbcad18155201308c8f3540b07f84f5e"
rpc-timeout = "45s"

[auth.local]
private-keys = []

[telemetry]
prometheus-addr = "0.0.0.0:3000"

[epoch.block-clock]
epoch-duration = 6
genesis-block = 0

[shutdown]
runtime-timeout = "5s"

[certificate-orchestrator]
input-backpressure-buffer-size = 1000

[certificate-orchestrator.prover.sp1-local]

[storage]
db-path = "./storage"


```
