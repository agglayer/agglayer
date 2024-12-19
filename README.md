<div id="top"></div>
<!-- PROJECT LOGO -->
<br />
<div align="center">

  <img src="./.github/assets/agglayer-logo.png#gh-light-mode-only" alt="Logo" width="100">
  <img src="./.github/assets/agglayer-logo.png#gh-dark-mode-only" alt="Logo" width="100">

<br />

<h1>Agglayer</h1>

<p align="center">
The <b>Agglayer</b> (<i>Aggregation layer</i>) provides a common language for secure, atomic, interoperability among heterogeneous chains. (WIP)
</p>
</div>

<br />

<div align="center">

[![Test workflow](https://github.com/agglayer/agglayer/actions/workflows/test.yml/badge.svg)](https://github.com/agglayer/agglayer/actions/workflows/test.yml)
[![Quality workflow](https://github.com/agglayer/agglayer/actions/workflows/quality.yml/badge.svg)](https://github.com/agglayer/agglayer/actions/workflows/quality.yml)
[![codecov](https://codecov.io/gh/agglayer/agglayer/graph/badge.svg?token=5TOBZRZ7Q8)](https://codecov.io/gh/agglayer/agglayer)

<hr />

<img src="./.github/assets/agglayer.png" alt="Logo">

</div>

## Table of Contents

- [Overview](#overview)
- [Repository Structure](#repository-structure)
- [Prerequisites](#prerequisites)
  - [Succinct Prover Network](#succinct-prover-network)
  - [Software Requirements](#software-requirements)
  - [Hardware Requirements](#hardware-recommendations)
- [Installation](#installation)
- [Running the Pessimistic Proof Test Suite](#running-the-pessimistic-proof-test-suite)
- [Development](#development)
- [Support](#support)
- [Resources](#resources)
- [License](#license)

## Overview

Agglayer is the Rust-based service designed to: 
1. Receive updates from Agglayer-connected chains 
2. Verify their validity 
3. Send them to the L1 for final settlement. 

To find out more about Agglayer, please visit [the more detailed documentation.](https://docs.polygon.technology/agglayer/overview/)

> [!WARNING]
>    - Some of the content in this section discusses technology in development and not ready for release. As such, all APIs and configuration are subject to change. The code is still being audited, so please contact the Polygon team if you would like to use it in production.
## Repository Structure

The crates and their functions within the Agglayer repo are as follows:

| Crate                                                                          | Description                                                                                                                                                                                                                                                                                                                                                        |
| ---                                                                            | ---                                                                                                                                                                                                                                                                                                                                                                |
| [agglayer-aggregator-notifier](/crates/agglayer-aggregator-notifier)           | Contains implementations for [Certifier](crates/agglayer-certificate-orchestrator/src/certifier.rs#L29) which applies new [Certificate](crates/agglayer-types/src/lib.rs#245) on top of an existing state and computes the proof, as well as [EpochPacker](crates/agglayer-certificate-orchestrator/src/epoch_packer.rs#14), which handles the packing of an epoch |
| [agglayer-certificate-orchestrator](/crates/agglayer-certificate-orchestrator) | Manages the orchestration and handling of certificates; also handles `current_epoch`, which allows non-orchestrators to push a proven certificate                                                                                                                                                                                                                  |
| [agglayer-clock](/crates/agglayer-clock)                                       | Defines the pace of the Agglayer in terms of epoch with support for two clocks: time (for testing) and block (for listening for L1 blocks)                                                                                                                                                                                                                         |
| [agglayer-config](/crates/agglayer-config)                                     | Manages configuration settings and parameters for Agglayer components                                                                                                                                                                                                                                                                                              |
| [agglayer-contracts](/crates/agglayer-contracts)                               | Contains smart contracts and related logic                                                                                                                                                                                                                                                                                                                         |
| [agglayer-gcp-kms](/crates/agglayer-gcp-kms)                                   | Provides integration with GCP's Key Management Service for secure key handling                                                                                                                                                                                                                                                                                     |
| [agglayer-node](/crates/agglayer-node)                                         | Responsible for spawning and running the different components of the node                                                                                                                                                                                                                                                                                          |
| [agglayer-prover-types](/crates/agglayer-prover-types)                         | Defines data structures and types used by the prover                                                                                                                                                                                                                                                                                                               |
| [agglayer-prover](/crates/agglayer-prover)                                     | Responsible for running everything related to the prover                                                                                                                                                                                                                                                                                                           |
| [agglayer-signer](/crates/agglayer-signer)                                     | Manages signing operations                                                                                                                                                                                                                                                                                                                                         |
| [agglayer-storage](/crates/agglayer-storage)                                   | Contains two layers: a physical layer for abstracting RocksDB and a logic layer for exposing the interface to other crates so that they may interact with the storage                                                                                                                                                                                              |
| [agglayer-telemetry](/crates/agglayer-telemetry)                               | Handles telemetry and monitoring functionalities                                                                                                                                                                                                                                                                                                                   |
| [agglayer-types](/crates/agglayer-types)                                       | Defines common data types and structures                                                                                                                                                                                                                                                                                                                           |
| [agglayer](/crates/agglayer)                                                   | The CLI for interacting with the Agglayer                                                                                                                                                                                                                                                                                                                          |
| [pessimistic-proof-program](/crates/pessimistic-proof-program)                 | Implements the pessimistic proof program                                                                                                                                                                                                                                                                                                                           |
| [pessimistic-proof-test-suite](/crates/pessimistic-proof-test-suite)           | Provides a suite of tests for validating the functionality of the pessimistic proof program                                                                                                                                                                                                                                                                        |
| [pessimistic-proof](/crates/pessimistic-proof)                                 | Contains the core logic and implementation of the pessimistic proof mechanism                                                                                                                                                                                                                                                                                      |

## Prerequisites

Before working with the repository, you’ll need the following:

### Succinct Prover Network

You’ll need to submit a unique Ethereum address to Succinct for access to their proving network. To get access:

1. Follow the instructions [here](https://docs.succinct.xyz/docs/generating-proofs/prover-network/key-setup) to use Foundry to generate a new private key or retrieve an existing one.
2. Apply for access for the public address associated with your private key to Succinct Network [here](https://docs.google.com/forms/d/e/1FAIpQLSd-X9uH7G0bvXH_kjptnQtNil8L4dumrVPpFE4t8Ci1XT1GaQ/viewform).

### Software Requirements
* [Rustup](https://www.rust-lang.org/tools/install) (stable)
* [protoc](https://grpc.io/docs/protoc-installation/)
* [nextest](https://nexte.st/docs/installation/pre-built-binaries/#with-cargo-binstall)
* [cargo-make](https://github.com/sagiegurari/cargo-make#installation)
* [cargo-insta](https://insta.rs/docs/quickstart/)
* [Go](https://go.dev/doc/install)

### Hardware Recommendations
With SP1, you do not need to generate proofs locally on your machine.

However, if you’d like to run a prover locally (not recommended), you’ll need roughly 40-50GB of available RAM.

## Installation

To install the Agglayer repository, please run the following:

```bash
git clone https://github.com/agglayer/agglayer
cd agglayer
```

To build Agglayer locally, please run:
```bash
cargo build
```

## Running the Pessimistic Proof Test Suite
To run the Pessimistic Proof Test Suite with test inputs in Native Rust Execution, please run the following:

```bash
cargo test --package pessimistic-proof-test-suite
```

You can find the test inputs here: [`./agglayer/crates/pessimistic-proof-test-suite`](./crates/pessimistic-proof-test-suite/data/)

## Running SP1 Proof Generation Locally (Not Recommended)

The [Succinct Prover Network](#succinct-prover-network) is the best way to generate Pessimistic Proofs for Agglayer. 

For those with the hardware and know-how, however, you can run the Pessimistic Proof program in a local SP1 Prover with the following commands:

```bash
cargo run --package pessimistic-proof-test-suite --bin ppgen
```

## Development

Contributions are very welcomed, the guidelines are currently not available (WIP)

## Support

Feel free to [open an issue](https://github.com/agglayer/agglayer/issues/new) if you have any feature request or bug report.<br />

## Resources
  
## License
Copyright (c) 2024 PT Services DMCC

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 

The SPDX license identifier for this project is `MIT OR Apache-2.0`.
