#[test]
fn vkey_snapshot() {
    let selector = hex::encode(pessimistic_proof::core::PESSIMISTIC_PROOF_PROGRAM_SELECTOR);
    let vkey = agglayer_prover::compute_program_vkey(pessimistic_proof::ELF);
    let col1 = "SELECTOR";
    let col2 = "VKEY";

    let message = format!(
        "# If this test fails, it means the PP vkey has changed.\n# When that happens, consider \
         updating the selector by bumping the PP version.\n| {col1:10} | {col2:66} |\n| \
         0x{selector} | {vkey} |\n",
    );

    insta::assert_snapshot!(message);
}
