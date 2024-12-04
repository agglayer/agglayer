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
- [Crate Directory](#crate-directory)
- [Prerequisites](#prerequisites)
  - [Succinct Prover Network](#succinct-prover-network)
  - [Software Requirements](#software-requirements)
  - [Hardware Requirements](#hardware-requirements)
- [Installation](#installation)
- [Running the Pessimistic Proof Test Suite](#running-the-pessimistic-proof-test-suite)
- [Development](#development)
- [Support](#support)
- [Resources](#resources)
- [License](#license)

## Overview

AggLayer is the Rust-based service designed to (1) receive zero-knowledge proofs from AggLayer-connected chains, (2) verify their validity, and (3) send them to the L1 for final settlement. AggLayer replaces the previous Golang implementation. 

To find out more about Polygon, visit the [official website.](https://docs.polygon.technology/cdk/)

WARNING: This is a work in progress, and as such, all APIs and configuration are subject to change. The code is still being audited, so please contact the Polygon team if you would like to use it in production.

## Crate Directory

The crates and their functions within the AggLayer repo are as follows:

| Crate                          | Description                                                                                                  |
|--------------------------------|--------------------------------------------------------------------------------------------------------------|
| agglayer-aggregator-notifier   | Contains certificate implementations for `certify` and `settled`, as well as `packer`, which handles the packing of an epoch |
| agglayer-certificate-orchestrator | Manages the orchestration and handling of certificates; also handles `current_epoch`, which allows non-orchestrators to push a proven certificate |
| agglayer-clock                 | Defines the pace of the Agglayer in terms of epoch with support for two clocks: time (for testing) and block (for listening for L1 blocks) |
| agglayer-config                | Manages configuration settings and parameters for Agglayer components                                       |
| agglayer-contracts             | Contains smart contracts and related logic                                                                  |
| agglayer-gcp-kms               | Provides integration with GCP's Key Management Service for secure key handling                              |
| agglayer-node                  | Responsible for spawning and running the different components of the node                                   |
| agglayer-prover-types          | Defines data structures and types used by the prover                                                        |
| agglayer-prover                | Responsible for running everything related to the prover                                                    |
| agglayer-signer                | Manages signing operations                                                                                  |
| agglayer-storage               | Contains two layers: a physical layer for abstracting RocksDB and a logic layer for exposing the interface to other crates so that they may interact with the storage |
| agglayer-telemetry             | Handles telemetry and monitoring functionalities                                                            |
| agglayer-types                 | Defines common data types and structures                                                                    |
| agglayer                       | The CLI for interacting with the Agglayer                                                                   |
| pessimistic-proof-program      | Implements the pessimistic proof program                                                                    |
| pessimistic-proof-test-suite   | Provides a suite of tests for validating the functionality of the pessimistic proof program                  |
| pessimistic-proof              | Contains the core logic and implementation of the pessimistic proof mechanism                               |

## Prerequisite

Before working with the repository, you’ll need the following:

### Succinct Prover Network

You’ll need to submit a unique Ethereum address to Succinct for access to their proving network. To get access:

1. Follow the instructions [here](https://docs.succinct.xyz/docs/generating-proofs/prover-network/key-setup) to use Foundry to generate a new private key or retrieve an existing one
2. Apply for access for the public address associated with your private key to Succinct Network [here.](https://docs.google.com/forms/d/e/1FAIpQLSd-X9uH7G0bvXH_kjptnQtNil8L4dumrVPpFE4t8Ci1XT1GaQ/viewform)

### Software Requirements
#### Rust
AggLayer is built with the stable version of Rust. To install Rust using `rustup` please see instructions [here.](https://www.rust-lang.org/tools/install) 

#### Go
To run certain libraries, you’ll need to have Go installed. Follow instructions to install Go [here.](https://go.dev/doc/install)

#### Protoc
If you don’t have protoc installed, [source it here](https://docs.rs/prost-build/latest/prost_build/#sourcing-protoc) or, for MacOS, you can run `brew install protobuf`

### Hardware Recommendations
With SP1, you do not need to generate proofs locally on your machine

However, if you’d like to run a prover locally (not recommended), you’ll need roughly 40-50GB of available RAM.

## Installation

To install the AggLayer repository, please run the following:

```bash
git clone https://github.com/agglayer/agglayer
cd agglayer
```

To build AggLayer locally, please run:
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

The [Succinct Prover Network](#succinct-prover-network) is the best way to generate Pessimistic Proofs for AggLayer. 

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
