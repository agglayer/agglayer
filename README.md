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
  - [Succinct Prover Network Access](#succinct-prover-network-access)
  - [Software Requirements](#software-requirements)
  - [Hardware Requirements](#hardware-requirements)
- [Installation](#installation)
- [Running the Pessimistic Proof Test Suite](#running-the-pessimistic-proof-test-suite)
- [Development](#development)
- [Support](#support)
- [Resources](#resources)
- [License](#license)

## Overview

AggLayer-rs is the Rust-based service designed to (1) receive zero-knowledge proofs from AggLayer-connected chains, (2) verify their validity, and (3) send them to the L1 for final settlement. AggLayer-rs replaces the previous Golang implementation. 

To find out more about Polygon, visit the [official website.](https://docs.polygon.technology/cdk/)

WARNING: This is a work in progress, and as such, all APIs and configuration are subject to change. The code is still being audited, so please contact the Polygon team if you would like to use it in production.

## Crate Directory

The crates and their functions within the AggLayer repo are as follows:

| Crate                          | Description                                                                 |
|--------------------------------|-----------------------------------------------------------------------------|
| agglayer-aggregator-notifier   | Notifies aggregators about new events                                       |
| agglayer-certificate-orchestrator | Manages the orchestration and handling of certificates                     |
| agglayer-clock                 | Provides time-related functionalities and synchronization                   |
| agglayer-config                | Manages configuration settings and parameters for Agglayer components       |
| agglayer-contracts             | Contains smart contracts and related logic                                  |
| agglayer-gcp-kms               | Provides integration with GCP’s Key Management Service for secure key handling |
| agglayer-node                  | Provides the core node implementation, handling RPC services and kernel operations |
| agglayer-prover-types          | Defines data structures and types used by the prover                        |
| agglayer-prover                | Responsible for generating zero-knowledge proofs                            |
| agglayer-signer                | Manages signing operations                                                  |

## Prerequisite

Before working with the repository, you’ll need the following:

### Succinct Prover Network

You’ll need to submit a unique Ethereum address to Succinct for access to their proving network. To get access:

1. Follow the instructions [here](https://docs.succinct.xyz/generating-proofs/prover-network/key-setup.html) to use Foundry to generate a new private key or retrieve an existing one
2. Apply for access for the public address associated with your private key to Succinct Network [here.](https://docs.google.com/forms/d/e/1FAIpQLSd-X9uH7G0bvXH_kjptnQtNil8L4dumrVPpFE4t8Ci1XT1GaQ/viewform)

### Software Requirements
#### Rust
AggLayer-rs is built with the stable version of Rust. To install Rust using `rustup` please see instructions [here.](https://www.rust-lang.org/tools/install) 

#### Go
To run certain libraries, you’ll need to have Go installed. Follow instructions to install Go [here.](https://go.dev/doc/install)

#### Protoc
If you don’t have protoc installed, [source it here](https://docs.rs/prost-build/latest/prost_build/#sourcing-protoc) or, for MacOS, you can run `brew install protobuf`

### Hardware Recommendations for Local Prover (Optional)
With SP1, you do not need to generate proofs locally on your machine

However, if you’d like to run a prover locally, you’ll need roughly 40-50GB of available RAM.

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
To run the Pessimistic Proof Test Suite with mock transaction and exit data, please run the following:

```
cd crates
cargo test -p pessimistic-proof-test-suite
```

You can find the dummy data in [`./agglayer/crates/pessimistic-proof-test-suite`](./crates/pessimistic-proof-test-suite/data/)

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
