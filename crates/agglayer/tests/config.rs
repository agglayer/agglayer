use assert_cmd::Command;

#[test]
fn config_display() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("agglayer")?;
    cmd.args(["config", "--base-dir", "/tmp/agglayer-test"]);

    let output = cmd.assert().success();

    let result: &str = std::str::from_utf8(&output.get_output().stdout)?;

    insta::assert_snapshot!(sanitize_config_folder_path(result));

    Ok(())
}

#[test]
fn prover_config_display() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("agglayer")?;
    cmd.args(["prover-config"]);

    let output = cmd.assert().success();

    let result: &str = std::str::from_utf8(&output.get_output().stdout)?;

    insta::assert_snapshot!(sanitize_config_folder_path(result));

    Ok(())
}

pub fn sanitize_config_folder_path(cmd_out: &str) -> String {
    let dir = dirs::config_dir().unwrap().join("agglayer");
    cmd_out.replace(
        dir.to_str().expect("Unable to transform config dir"),
        "/tmp/agglayer-test",
    )
}
