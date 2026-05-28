use assert_cmd::Command;
use predicates::prelude::*;

fn koban() -> Command {
    Command::cargo_bin("koban").expect("koban binary")
}

#[test]
fn completions_bash_outputs_script() {
    koban()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("clients"));
}

#[test]
fn completions_zsh_outputs_script() {
    koban()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef koban"))
        .stdout(predicate::str::contains("invoices"));
}

#[test]
fn completions_fish_outputs_script() {
    koban()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("payments"));
}

#[test]
fn completions_powershell_outputs_script() {
    koban()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"))
        .stdout(predicate::str::contains("statics"));
}

#[test]
fn completions_elvish_outputs_script() {
    koban()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "edit:completion:arg-completer[koban]",
        ))
        .stdout(predicate::str::contains("clients"));
}

#[test]
fn completions_nushell_outputs_script() {
    koban()
        .args(["completions", "nushell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export extern koban"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn dynamic_root_completions_include_resources() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .args(["--", "koban", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("clients"))
        .stdout(predicate::str::contains("invoices"))
        .stdout(predicate::str::contains("payments"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn dynamic_resource_completions_include_list_and_show() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .args(["--", "koban", "clients", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("\ntemplate\n"))
        .stdout(predicate::str::contains("edit-template"));
}
