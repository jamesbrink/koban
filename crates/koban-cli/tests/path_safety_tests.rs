use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn koban() -> Command {
    let mut command = Command::cargo_bin("koban").expect("koban binary");
    command
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", "http://127.0.0.1:9");
    command
}

#[test]
fn path_ids_reject_route_changing_segments_before_network() {
    let tempdir = tempdir().expect("tempdir");
    let upload = tempdir.path().join("upload.txt");
    std::fs::write(&upload, b"upload").expect("upload fixture");
    let output = tempdir.path().join("invoice.pdf");

    let cases = vec![
        vec!["clients", "show", "../statics"],
        vec!["clients", "edit-template", "../statics"],
        vec![
            "products",
            "update",
            "product/one",
            "--name",
            "Consulting",
            "--dry-run",
        ],
        vec!["webhooks", "delete", ".", "--dry-run"],
        vec![
            "tax-rates",
            "bulk",
            "--action",
            "archive",
            "--id",
            "../statics",
            "--dry-run",
        ],
        vec![
            "clients",
            "upload",
            "client/one",
            "--file",
            upload.to_str().expect("upload path"),
            "--dry-run",
        ],
        vec![
            "clients",
            "action",
            "../clients",
            "--action",
            "archive",
            "--dry-run",
        ],
        vec![
            "invoices",
            "update",
            "invoice/one",
            "--public-notes",
            "updated",
            "--dry-run",
        ],
        vec![
            "invoices",
            "bulk",
            "--action",
            "archive",
            "--id",
            "invoice/one",
            "--dry-run",
        ],
        vec![
            "invoices",
            "action",
            "../clients",
            "--action",
            "archive",
            "--dry-run",
        ],
        vec![
            "invoices",
            "download",
            "../statics",
            "--output-file",
            output.to_str().expect("output path"),
        ],
        vec![
            "invoices",
            "delivery-note",
            "../statics",
            "--output-file",
            output.to_str().expect("output path"),
        ],
    ];

    for args in cases {
        koban()
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains("safe single path"));
    }
}
