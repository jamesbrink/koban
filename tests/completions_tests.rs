use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_koban");

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn completions_bash_outputs_script() {
    let output = Command::new(BIN)
        .args(["completions", "bash"])
        .output()
        .expect("run koban completions bash");
    assert!(output.status.success(), "command failed: {output:?}");

    let stdout = stdout(&output);
    assert!(stdout.contains("complete"), "got: {stdout}");
}

#[test]
fn completions_zsh_outputs_script() {
    let output = Command::new(BIN)
        .args(["completions", "zsh"])
        .output()
        .expect("run koban completions zsh");
    assert!(output.status.success(), "command failed: {output:?}");

    let stdout = stdout(&output);
    assert!(stdout.contains("#compdef koban"), "got: {stdout}");
}

#[test]
fn dynamic_root_completions_include_completions() {
    let output = Command::new(BIN)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .args(["--", "koban", ""])
        .output()
        .expect("run dynamic completion");
    assert!(output.status.success(), "completion failed: {output:?}");

    let stdout = stdout(&output);
    assert!(stdout.contains("completions"), "got: {stdout}");
}
