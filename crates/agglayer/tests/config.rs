use assert_cmd::Command;

#[test]
fn config_display() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("agglayer")?;
    cmd.arg("config");

    let output = cmd.assert().success();

    let result: &str = std::str::from_utf8(&output.get_output().stdout)?;

    insta::assert_snapshot!(result);

    Ok(())
}
