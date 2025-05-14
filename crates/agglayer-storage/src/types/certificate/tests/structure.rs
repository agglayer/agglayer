use agglayer_types::Digest;

fn trace() -> serde_reflection::Result<serde_reflection::Registry> {
    use pessimistic_proof::unified_bridge::{GlobalIndex, TokenInfo};

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct Address(agglayer_types::Address);

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    enum ProofError {
        InvalidPreviousPessimisticRoot {
            declared: Digest,
            computed_v2: Digest,
            computed_v3: Digest,
        },
        InvalidImportedBridgeExit {
            //source: Error,
            global_index: GlobalIndex,
        },
        InvalidImportedExitsRoot {
            declared: Digest,
            computed: Digest,
        },
        InvalidNullifierPath,
        InvalidBalancePath,
        BalanceOverflowInBridgeExit,
        BalanceUnderflowInBridgeExit,
        CannotExitToSameNetwork,
        InvalidMessageOriginNetwork,
        //InvalidL1TokenInfo(TokenInfo),
        //MissingTokenBalanceProof(TokenInfo),
        //DuplicateTokenBalanceProof(TokenInfo),
        InvalidSignature,
        InconsistentSignedPayload,
        InvalidSigner {
            declared: Address,
            recovered: Address,
        },
        //InvalidLocalExitTreeOperation(LocalExitTreeError),
        Unknown(String),
        HeightOverflow,
    }

    // Take a snapshot of the internal structure of the encoding of Certificate
    // to avoid accidental changes.
    let tracer_config = serde_reflection::TracerConfig::default().is_human_readable(false);
    let mut tracer = serde_reflection::Tracer::new(tracer_config);
    let mut samples = serde_reflection::Samples::new();

    tracer.trace_value::<Address>(
        &mut samples,
        &Address(agglayer_types::Address::new([0xad; 20])),
    )?;
    tracer.trace_value::<Digest>(&mut samples, &Digest([0xd1; 32]))?;

    tracer.trace_simple_type::<agglayer_types::GenerationType>()?;
    tracer.trace_simple_type::<GlobalIndex>()?;
    tracer.trace_type::<TokenInfo>(&samples)?;
    eprintln!("{samples:?}");
    //tracer.trace_simple_type::<agglayer_types::CertificateStatus>().unwrap();
    //tracer.trace_simple_type::<agglayer_types::CertificateStatusError>().unwrap();
    tracer.trace_type::<ProofError>(&samples)?;
    tracer.trace_simple_type::<pessimistic_proof::error::ProofVerificationError>()?;

    //let registry = tracer.registry().unwrap();
    tracer.registry()
}

#[test]
fn structure_snapshot() {
    match trace() {
        Ok(registry) => insta::assert_json_snapshot!(&registry),
        Err(err) => panic!("{err}: {}", err.explanation()),
    }
}
