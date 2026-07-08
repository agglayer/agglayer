# Architecture

This chapter maps the Agglayer workspace to functional domains,
then describes the certificate lifecycle and settlement path.
Use this file as the canonical "which crate owns what" index,
including README references to crate/domain ownership.

## Crate map

### Node entrypoints and runtime wiring

| Crate | Primary responsibility |
|---|---|
| `agglayer` | CLI binary and user-facing subcommands (`run`, config, backups, vkey tools) |
| `agglayer-node` | Node bootstrap and component wiring |
| `agglayer-config` | TOML configuration schema, parsing, validation, path contextualization |
| `agglayer-clock` | Epoch pacing (time clock and block clock) |
| `agglayer-telemetry` | Metrics export and tracing integration |
| `agglayer-utils` | Shared helpers used across crates |

### Certificate processing and settlement

| Crate | Primary responsibility |
|---|---|
| `agglayer-certificate-orchestrator` | Certificate lifecycle orchestration and task scheduling |
| `agglayer-aggregator-notifier` | Certifier client and epoch packing pipeline |
| `agglayer-settlement-service` | Settlement request handling and L1 transaction workflow |
| `agglayer-signer` | Signing abstraction used for settlement/proof flows |
| `agglayer-gcp-kms` | GCP KMS-backed key management |
| `agglayer-contracts` | Contract bindings and contract-facing settlement logic |

### API and transport

| Crate | Primary responsibility |
|---|---|
| `agglayer-jsonrpc-api` | JSON-RPC API surface and handlers |
| `agglayer-rpc` | Internal RPC service implementation |
| `agglayer-grpc-api` | gRPC API traits and service definitions |
| `agglayer-grpc-server` | gRPC server implementation |
| `agglayer-grpc-client` | gRPC client implementation |
| `agglayer-grpc-types` | Protobuf/generated conversion layer |

### State, types, and testing

| Crate | Primary responsibility |
|---|---|
| `agglayer-storage` | RocksDB physical/logical storage layers and migrations |
| `agglayer-types` | Core domain types and shared error/status types |
| `agglayer-test-suite` | Shared test fixtures and test helpers |

### Pessimistic-proof pipeline

| Crate | Primary responsibility |
|---|---|
| `pessimistic-proof-core` | Proof primitives and transition logic |
| `pessimistic-proof` | Host-side SP1 integration and verification helpers |
| `pessimistic-proof-program` | SP1 zkVM guest program (`no_main`) |
| `pessimistic-proof-test-suite` | Proof-focused integration and compatibility tests |

Notes:

- `pessimistic-proof-program` is intentionally excluded from the default Cargo
  workspace build graph because it is cross-compiled for SP1.
- Several crates expose `testutils` features for test-only helpers.
  Prefer those helpers over ad hoc mocks when extending tests.

## Dependency tiers

Use this dependency mental model when scoping changes:

1. **Foundations:** `agglayer-types`, `agglayer-storage`, proof crates.
2. **Domain services:** orchestrator, settlement, notifier, signer, contracts.
3. **Transport surfaces:** JSON-RPC and gRPC crates.
4. **Runtime composition:** `agglayer-node` and the `agglayer` binary.

Changes in lower tiers have larger blast radius.
For example, edits in `agglayer-types` are likely to affect all upper tiers.

## Certificate lifecycle

Certificates move through a deterministic pipeline.

```text
Client submit (JSON-RPC / gRPC)
  -> Pending persistence
  -> Orchestrator scheduling
  -> Proof/certification execution
  -> Proven header update
  -> Settlement request creation
  -> L1 transaction submission
  -> Settled header/state update
```

Ownership by phase:

- Submission and request validation: `agglayer-jsonrpc-api`, `agglayer-rpc`,
  `agglayer-grpc-server`.
- Pending/proven/settled state transitions: `agglayer-storage` and
  `agglayer-certificate-orchestrator`.
- Proof generation and verification: `agglayer-aggregator-notifier`,
  `pessimistic-proof*` crates.
- L1 settlement: `agglayer-settlement-service`, `agglayer-contracts`,
  `agglayer-signer`.

## Settlement flow

Settlement finalizes proven certificates on Ethereum L1.

1. The orchestrator marks a certificate as ready for settlement.
2. The settlement service constructs the contract call payload.
3. The signer produces transaction signatures (local key or GCP KMS).
4. Contract adapters submit via Alloy providers.
5. Storage updates to settled status after confirmation criteria are met.

Safety expectations:

- Never bypass proof-validation preconditions before settlement.
- Keep retries idempotent and bounded by configuration.
- Treat signer and contract changes as security-sensitive,
  with explicit blast-radius analysis.
