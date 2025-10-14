This documents gathers several pointers to a prototype on the program implementation for aggregation with preconfirmation.

The main approach of the prototype is the approach 2 described there: https://github.com/agglayer/protocol-research/issues/183#issuecomment-3350834366

# Pessimistic proof program

## Gathering of the imported LER out of the imported bridge exits

https://github.com/agglayer/agglayer/blob/426d10712014eb967c7e8e7f0674a0a1f852c338/crates/pessimistic-proof-core/src/proof.rs#L257-L274

Note:

- Imported LER aren't hashed yet in the public inputs, they are listed in clear as public inputs of the PP.

# Aggregation program

The structure `AggregationWitness` gathers all the prover inputs, defined there:

https://github.com/agglayer/agglayer/blob/426d10712014eb967c7e8e7f0674a0a1f852c338/crates/aggregation-proof-core/src/lib.rs#L215-L248

The function `AggregationWitness::verify()` is the top level function of the aggregation program gathering all the verification steps, defined there:

https://github.com/agglayer/agglayer/blob/733d46e8a7755c6e517c0b35a6abd2fb682b6806/crates/aggregation-proof-core/src/lib.rs#L276-L313

# Test

The prototype introduced the need and implementation of a certificate graph builder to generate test data.

Example of a simple aggregation:

https://github.com/agglayer/agglayer/blob/530d0621f177165477e71104e9f32db78a54add0/crates/pessimistic-proof-test-suite/tests/aggregation.rs#L5-L41

Can be run with:

```
cargo test --package pessimistic-proof-test-suite --test aggregation -- simple_aggregation --exact --show-output
```

The aggregation test data is defined from one graph described as follow:

```rust
let cert_graph = {
    let mut dag = CertGraphBuilder::default();
    let a1 = dag.add_cert('A');
    let a2 = dag.add_cert('A');
    let a3 = dag.add_cert('A');
    let b1 = dag.add_cert('B');
    let b2 = dag.add_cert('B');

    b1.claims_from(a1, &mut dag);
    b2.claims_from(a3, &mut dag);

    dag.build()
};
```

It returns:

```
---- simple_aggregation stdout ----
Graph representation:
┌────┐  ClaimedBy   ┌────────────┐
│ B1 │ ◀─────────── │     A1     │
└────┘              └────────────┘
  │                   │
  │                   │ Next
  │                   ▼
  │                 ┌────────────┐
  │                 │     A2     │
  │                 └────────────┘
  │                   │
  │                   │ Next
  │                   ▼
  │                 ┌────────────┐
  │                 │     A3     │
  │                 └────────────┘
  │                   │
  │                   │ ClaimedBy
  │                   ▼
  │    Next         ┌────────────┐
  └───────────────▶ │     B2     │
                    └────────────┘

aggregation public values: AggregationPublicValues {
    hash_chain_pp_inputs: 9ff232f0af68162b09ee4bc03ff0c4a71f540e03b6b342f421f142d79f7662f8,
    pp_vkey: [
        1245470243,
        1286210985,
        1043803508,
        760809471,
        6583742,
        1577009209,
        84592754,
        212580814,
    ],
    l1_info_root: 0000000000000000000000000000000000000000000000000000000000000000,
    prev_arer: 4a8656e9ec38198afb4db94058f5923a5fe98282ceb5b64967d5e0fb6a0306cb,
    new_arer: a8bb74aa4707c3003a083d006ce79c90dcbc0f31e19dbb0a226401f775fa2e27,
}
```

- Each node of the certificate graph corresponds to one PP and for which we can generate one PP
- `A1 ClaimedBy B1` means two things:
  - A1 is a preconfirmed LER
  - B1 is a preconf certificate which has a claim from the preconf of A1
- `A2 Next A3` simply means that `A3` is the next certificate/preconf from network `A`, after `A2`.
