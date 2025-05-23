use agglayer_types::{CertificateStatus, CertificateStatusError, Digest, NetworkId};
use alloy_primitives::Bytes;
use pessimistic_proof::{error::ProofVerificationError, ProofError};

use crate::columns::bincode_codec;

type Cse = CertificateStatusError;
type Pve = ProofVerificationError;

fn err(error: CertificateStatusError) -> CertificateStatus {
    CertificateStatus::InError { error }
}

fn proof_err(source: ProofError) -> CertificateStatus {
    err(CertificateStatusError::ProofGenerationError {
        generation_type: agglayer_types::GenerationType::Prover,
        source,
    })
}

fn ver_err(error: ProofVerificationError) -> CertificateStatus {
    err(CertificateStatusError::ProofVerificationFailed(error))
}

#[rstest::rstest]
#[case("pending", CertificateStatus::Pending)]
#[case("candidate", CertificateStatus::Candidate)]
#[case("proven", CertificateStatus::Proven)]
#[case("settled", CertificateStatus::Settled)]
#[case("err-prf-ho", proof_err(ProofError::HeightOverflow))]
#[case("err-prf-inp", proof_err(ProofError::InvalidNullifierPath))]
#[case("err-prf-is", proof_err(ProofError::InvalidSignature))]
#[case("err-prf-iier", proof_err(ProofError::InvalidImportedExitsRoot {
    declared: Digest([0x44; 32]),
    computed: Digest([0x55; 32]),
}))]
#[case("err-ver-vm", ver_err(Pve::VersionMismatch("vm".into())))]
#[case("err-ver-core", ver_err(Pve::Core("core".into())))]
#[case("err-ver-rec", ver_err(Pve::Recursion("rec".into())))]
#[case("err-ver-pl", ver_err(Pve::Plonk("plonk".into())))]
#[case("err-ver-gr", ver_err(Pve::Groth16("groth16".into())))]
#[case("err-ver-pv", ver_err(Pve::InvalidPublicValues))]
#[case("err-ts", err(Cse::TrustedSequencerNotFound(NetworkId::new(6))))]
#[case("err-pr", err(Cse::LastPessimisticRootNotFound(NetworkId::new(5))))]
#[case("err-ie", err(Cse::InternalError("internal".into())))]
#[case("err-se", err(Cse::SettlementError("settlement".into())))]
#[case("err-pce", err(Cse::PreCertificationError("precert".into())))]
#[case("err-ce", err(Cse::CertificationError("cert".into())))]
#[case("err-l1", err(Cse::L1InfoRootNotFound(0xabcd)))]
fn encoding(#[case] name: &'static str, #[case] status: CertificateStatus) {
    // Check for changes in encoding of certificate status.
    // Reordering arms in the status enum causes the storage encoding to change, causing
    // compatibility issues.

    let bytes = Bytes::from(bincode_codec().serialize(&status).unwrap());
    insta::assert_snapshot!(name, bytes);
}
