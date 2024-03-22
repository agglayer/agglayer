# `agglayer-rs`

Rust implementation of the Agglayer.

Agglayer is a web service that receives ZKPs from different CDK chains and checks the soundness of them before sending the ZKP to L1 for verification.

To find out more about Polygon, visit the [official website](https://docs.polygon.technology/cdk/).

WARNING: This is a work in progress, and as such, all APIs and configuration are subject to change. The code is still being audited, so please contact the Polygon team if you would like to use it in production.

- [`agglayer-rs`](#agglayer-rs)
- [Prerequisites](#prerequisites)


# Prerequisites

* For each CDK chain it's necessary to configure its corresponding RPC node, synced with the target CDK. This node will be used for checking the state root after executions of L2 batches.
