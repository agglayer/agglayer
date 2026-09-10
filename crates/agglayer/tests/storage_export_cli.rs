use assert_cmd::Command;

const L1_RPC_ENV: &str = "AGGLAYER_L1_RPC_URL";

#[test]
fn tree_export_help_hides_l1_rpc_env_value() -> eyre::Result<()> {
    let secret_url = "https://rpc.example.invalid/api-key/secret-value";
    let mut command = Command::cargo_bin("agglayer")?;
    command
        .env(L1_RPC_ENV, secret_url)
        .args(["storage", "export-trees", "--help"]);

    let output = command.assert().success().get_output().stdout.clone();
    let help = std::str::from_utf8(&output)?;

    assert!(help.contains("[env: AGGLAYER_L1_RPC_URL]"));
    assert!(!help.contains(secret_url));

    Ok(())
}

#[test]
fn tree_export_reads_l1_rpc_url_from_environment() -> eyre::Result<()> {
    const SECRET: &str = "credential-which-must-not-be-echoed";
    let mut command = Command::cargo_bin("agglayer")?;
    command
        .env(
            L1_RPC_ENV,
            format!("ftp://user:{SECRET}@not-an-http-endpoint.invalid"),
        )
        .args([
            "storage",
            "export-trees",
            "--storage-path",
            "/unused/storage",
            "--output-path",
            "/unused/output",
        ]);

    let output = command.assert().failure().get_output().stderr.clone();
    let stderr = std::str::from_utf8(&output)?;
    assert!(stderr.contains("L1 RPC URL must use http or https"));
    assert!(!stderr.contains(SECRET));

    Ok(())
}

#[test]
fn tree_export_cli_l1_rpc_url_takes_precedence_over_environment() -> eyre::Result<()> {
    let storage = tempfile::tempdir()?;
    let output = storage.path().join("output");
    let mut command = Command::cargo_bin("agglayer")?;
    command
        .env(L1_RPC_ENV, "file:///not-an-http-endpoint")
        .args([
            "storage",
            "export-trees",
            "--storage-path",
            storage.path().to_str().expect("temporary path is UTF-8"),
            "--output-path",
            output.to_str().expect("temporary path is UTF-8"),
            "--l1-rpc-url",
            "http://127.0.0.1:8545",
        ]);

    let output = command.assert().failure().get_output().stderr.clone();
    let stderr = std::str::from_utf8(&output)?;
    assert!(stderr.contains("has no state directory"));
    assert!(!stderr.contains("L1 RPC URL must use http or https"));

    Ok(())
}
