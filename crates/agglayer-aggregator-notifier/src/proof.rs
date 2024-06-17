/// Proof is a wrapper around all the different types of proofs that can be
/// generated
pub(crate) enum Proof {
    SP1(sp1_sdk::SP1Proof),
}
