use agglayer_types::{Address, Digest, NetworkId};
use pessimistic_proof::unified_bridge;

fn trace() -> serde_reflection::Result<serde_reflection::Registry> {
    let tracer_config = serde_reflection::TracerConfig::default().is_human_readable(false);
    let mut tracer = serde_reflection::Tracer::new(tracer_config);
    let mut samples = serde_reflection::Samples::new();

    // Support types with custom values.
    dbg!(tracer.trace_value::<Address>(&mut samples, &Address::new([0xad; 20]))?);
    dbg!(tracer.trace_value::<Digest>(&mut samples, &Digest([0xd1; 32]))?);
    dbg!(tracer.trace_value::<NetworkId>(&mut samples, &NetworkId::new(42))?);

    // Remaining support types.
    dbg!(tracer.trace_type::<agglayer_types::GenerationType>(&samples)?);
    dbg!(tracer.trace_type::<unified_bridge::GlobalIndex>(&samples)?);
    dbg!(tracer.trace_type::<unified_bridge::TokenInfo>(&samples)?);

    // Error enumerations.
    dbg!(tracer.trace_type::<unified_bridge::LocalExitTreeError>(&samples)?);
    dbg!(tracer.trace_type::<unified_bridge::Error>(&samples)?);
    dbg!(tracer.trace_type::<pessimistic_proof::core::MultisigError>(&samples)?);
    dbg!(tracer.trace_type::<pessimistic_proof::ProofError>(&samples)?);
    dbg!(tracer.trace_type::<pessimistic_proof::error::ProofVerificationError>(&samples)?);
    dbg!(tracer.trace_type::<agglayer_tries::error::SmtError>(&samples)?);
    dbg!(tracer.trace_type::<agglayer_types::aggchain_data::MultisigError>(&samples)?);
    dbg!(tracer.trace_type::<agglayer_types::aggchain_data::AggchainDataError>(&samples)?);
    dbg!(tracer.trace_type::<agglayer_types::Error>(&samples)?);
    dbg!(tracer.trace_type::<agglayer_types::CertificateStatusError>(&samples)?);

    // Certificate status.
    dbg!(tracer.trace_type::<agglayer_types::CertificateStatus>(&samples)?);

    tracer.registry()
}

#[test]
fn structure_snapshot() {
    // Take a snapshot of the internal structure of the encoding of
    // Certificate-related types to avoid accidental changes.
    //
    // Note that if this changes, backwards compatibility of the change has to
    // be considered. As a rule of thumb:
    // * It is OK to add an enum arm at the end
    // * It is NOT OK to remove, reorder or change existing enum arms.
    // * It is NOT OK to alter structs in any way, even just by adding a field at
    //   the end.
    match trace() {
        Ok(registry) => insta::assert_json_snapshot!(&registry),
        Err(err) => panic!("{err}: {}", err.explanation()),
    }
}
