#[tokio::test]
async fn vkey_snapshot() {
    let plonk = hex::encode(pessimistic_proof::core::PP_SELECTOR_PLONK);
    let groth16 = hex::encode(pessimistic_proof::core::PP_SELECTOR_GROTH16);
    let vkey = pessimistic_proof_test_suite::compute_program_vkey(pessimistic_proof::ELF)
        .await
        .unwrap();

    let message: String = [
        "# If this test fails, it means the PP vkey has changed.",
        "# When that happens, consider updating the selector by bumping the PP version.",
        &format!("| PP_VKEY          | {vkey} |"),
        &format!("| SELECTOR_PLONK   | 0x{plonk:64} |"),
        &format!("| SELECTOR_GROTH16 | 0x{groth16:64} |"),
    ]
    .into_iter()
    .flat_map(|line| [line, "\n"])
    .collect();

    insta::assert_snapshot!(message);
}
