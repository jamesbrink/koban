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
        .stdout(predicate::str::contains(
            "invoices:List, show, create, update, and manage invoices",
        ))
        .stdout(predicate::str::contains(
            "quotes:List, show, and inspect quotes",
        ))
        .stdout(predicate::str::contains(
            "download:Save an invoice PDF by invitation key",
        ))
        .stdout(predicate::str::contains(
            "delivery-note:Save a delivery note PDF by invoice ID",
        ));
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
        .stdout(predicate::str::contains("quotes"))
        .stdout(predicate::str::contains("credits"))
        .stdout(predicate::str::contains("vendors"))
        .stdout(predicate::str::contains("expenses"))
        .stdout(predicate::str::contains("projects"))
        .stdout(predicate::str::contains("tasks"))
        .stdout(predicate::str::contains("products"))
        .stdout(predicate::str::contains("recurring-invoices"))
        .stdout(predicate::str::contains("purchase-orders"))
        .stdout(predicate::str::contains("webhooks"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("update"))
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

#[test]
fn dynamic_expanded_resource_completions_include_write_commands() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .args(["--", "koban", "products", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("bulk"))
        .stdout(predicate::str::contains("upload"))
        .stdout(predicate::str::contains("action"));
}

#[test]
fn dynamic_inspect_resource_completions_omit_write_commands() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .args(["--", "koban", "imports", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("create").not())
        .stdout(predicate::str::contains("update").not())
        .stdout(predicate::str::contains("delete").not())
        .stdout(predicate::str::contains("bulk").not())
        .stdout(predicate::str::contains("upload").not())
        .stdout(predicate::str::contains("action").not());
}

#[test]
fn dynamic_invoice_completions_include_download_commands() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .args(["--", "koban", "invoices", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("bulk"))
        .stdout(predicate::str::contains("upload"))
        .stdout(predicate::str::contains("action"))
        .stdout(predicate::str::contains("download"))
        .stdout(predicate::str::contains("delivery-note"));
}

#[test]
fn dynamic_invoice_create_completions_include_write_flags() {
    koban()
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "3")
        .args(["--", "koban", "invoices", "create", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("--data-file"))
        .stdout(predicate::str::contains("--client-id"))
        .stdout(predicate::str::contains("--line-item"))
        .stdout(predicate::str::contains("--send-email"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--yes"));
}
