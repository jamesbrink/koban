use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_koban");

fn koban(args: &[&str]) -> std::process::Output {
    Command::new(BIN).args(args).output().expect("run koban")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn help_mentions_invoice_ninja() {
    let output = koban(&["--help"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains("Invoice Ninja"), "got: {stdout}");
    assert!(stdout.contains("completions"), "got: {stdout}");
}

#[test]
fn version_reports_package_version() {
    let output = koban(&["--version"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "got: {stdout}");
}

#[test]
fn no_args_prints_help() {
    let output = koban(&[]);
    assert!(!output.status.success(), "empty command should fail");

    let stderr = stderr(&output);
    assert!(stderr.contains("Usage:"), "got: {stderr}");
}
