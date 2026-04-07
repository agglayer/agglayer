# Glossary

This glossary defines the core terms used across Agglayer.
Use it together with [Architecture](architecture.md),
[Pessimistic Proof](pessimistic-proof.md),
[Protobuf and gRPC](protobuf-and-grpc.md), and [Storage](storage.md).

## Aggchain proof

An optional proof payload attached to a certificate by a connected chain.
Agglayer validates it according to the configured proof mode and commitment version.

## Balance tree

A per-network Merkle structure that tracks token balances used by the pessimistic proof.
State transitions in certificates update this tree.
See [Storage](storage.md#state-column-families).

## Bridge exit

A cross-chain transfer intent created on a source network.
It is represented in Merkle commitments and consumed during verification and settlement.

## Certificate

A chain update submitted to Agglayer.
It carries the data needed to advance network state,
including exit roots and optional proof material.
See [Architecture](architecture.md#certificate-lifecycle).

## Certificate header

Metadata and status for a certificate stored by Agglayer.
Headers track lifecycle state and failure information.

## Commitment version

The schema version of the hash commitment signed or proven by a chain.
Versions evolve over time (for example V2 through V5)
to include additional fields and safety checks.

## Epoch

A pacing window used by Agglayer to process and settle certificates.
Epochs can be time-based or block-based depending on node configuration.

## Imported bridge exit

A bridge exit originating from another network,
included as an input to the destination network transition.

## L1 info root

A root commitment from the L1 view used by Agglayer proof and validation logic.
It anchors certificate processing to L1 context.

## Local exit root

A Merkle root summarizing exits for a specific network transition.
Certificates and proof outputs contain old/new local exit roots.

## Network ID

The logical Agglayer identifier for a connected chain.
Network ID is a key dimension for storage, rate limits, and lifecycle state.

## Nullifier tree

A per-network Merkle structure used to prevent replay or double-consumption
of bridge exits.
See [Storage](storage.md#state-column-families).

## Pessimistic proof

The core zero-knowledge proof used by Agglayer to verify state transitions safely.
It is generated and verified through the SP1-based proof pipeline.
See [Pessimistic Proof](pessimistic-proof.md).

## Pessimistic root

A commitment in the pessimistic-proof state that summarizes balance/nullifier state.
Proof outputs include previous and new pessimistic roots.

## Proof mode

The verification mode configured for a network,
for example legacy ECDSA, multisig, or STARK plus multisig.
Mode selection determines which checks run for each certificate.

## Settlement

The process of submitting a proven certificate result to Ethereum L1.
Settlement finalizes cross-chain state transitions.
See [Architecture](architecture.md#settlement-flow).

## SP1 zkVM

The zero-knowledge virtual machine used by the pessimistic-proof program.
Agglayer compiles the guest program to an ELF artifact and verifies proofs on host.

## Verification key (vkey)

The cryptographic key identifying a specific proof program/circuit.
A vkey change is security-sensitive and requires explicit acceptance workflow.
See [Pessimistic Proof](pessimistic-proof.md#development-workflow).

## Vkey selector

A short selector derived from the active verification key version,
used by protocol components to route verification logic.

## Witness

Concrete private and public input data fed into the proof program
to produce a proof for a specific certificate transition.
