# Introduction

The Agglayer Knowledge Base is the human-first reference for architecture,
domain concepts,
and contributor workflows in this repository.
It is intended to be useful for both maintainers and AI coding agents,
with clarity and operational correctness as the primary goals.

## What this book covers

- Core terms and protocol language used across Agglayer.
- Crate and domain ownership,
  including certificate lifecycle and settlement flow.
- Safety-sensitive subsystems,
  such as pessimistic proof,
  protobuf boundaries,
  and storage layout.
- Contributor workflows for AI-agent configuration and docs publishing.

For generated Rust API documentation,
see the [rustdoc reference](rustdoc/agglayer/)
(also accessible via the `</>` icon in the toolbar).

## How to use it

1. Start with [Glossary](glossary.md)
   and [Architecture](architecture.md)
   to build shared context.
2. Use domain chapters for deep dives:
   [Pessimistic Proof](pessimistic-proof.md),
   [Protobuf and gRPC](protobuf-and-grpc.md),
   and [Storage](storage.md).
3. Use workflow chapters for contributor operations:
   [AI Agent Configuration](ai-agents.md)
   and [Documentation Publishing](docs-publishing.md).

## Editorial expectations

- Keep content factual,
  concise,
  and linked to concrete code paths where relevant.
- Prefer explicit safety invariants,
  failure modes,
  and verification guidance over vague descriptions.
- Update [Summary](SUMMARY.md)
  when adding or restructuring chapters.
