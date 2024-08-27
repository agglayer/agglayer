use assert_cmd::Command;

#[test]
fn config_display() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("agglayer")?;
    cmd.arg("config");

    let output = cmd.assert().success();

    let result: &str = std::str::from_utf8(&output.get_output().stdout)?;

    insta::assert_snapshot!(sanitize_config_folder_path(result));

    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
pub fn sanitize_config_folder_path(cmd_out: &str) -> String {
    let dir = dirs::config_dir().unwrap().join("agglayer");
    let pattern =
        regex::Regex::new(dir.to_str().expect("Unable to create regex dir exclusion")).unwrap();
    pattern
        .replace_all(cmd_out, "/tmp/agglayer-test")
        .to_string()
}
