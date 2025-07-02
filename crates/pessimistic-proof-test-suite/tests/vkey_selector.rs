#[test]
fn vkey_snapshot() {
    let selector = hex::encode(pessimistic_proof::core::PESSIMISTIC_PROOF_PROGRAM_SELECTOR);
    let vkey = agglayer_prover::compute_program_vkey(pessimistic_proof::ELF);

    let message: String = [
        "# If this test fails, it means the PP vkey has changed.",
        "# When that happens, consider updating the selector by bumping the PP version.",
        &format!("| PP_VKEY          | {vkey} |"),
        &format!("| PP_VKEY_SELECTOR | 0x{selector:64} |"),
    ]
    .into_iter()
    .flat_map(|line| [line, "\n"])
    .collect();

    insta::assert_snapshot!(message);
}
