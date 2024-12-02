#[test]
fn trycmd() {
    trycmd::TestCases::new()
        .pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/trycmd/*.md"))
        .run();
}
