use std::{fs, path::Path, path::PathBuf};

const MAX_SOURCE_LINES: usize = 900;
const MAX_TEST_LINES: usize = 1_100;

/// Workspace root, derived from this crate's manifest dir
/// (`<root>/crates/koban-cli`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root above crates/koban-cli")
        .to_path_buf()
}

#[test]
fn rust_source_files_stay_small_enough_to_review() {
    let mut failures = Vec::new();

    // Scan every workspace member's source and tests.
    collect_rust_file_failures(&workspace_root().join("crates"), &mut failures);

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn ci_keeps_coverage_gate_enforced() {
    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("read CI workflow");

    assert!(
        !ci.contains("continue-on-error: true"),
        "coverage should not be advisory in CI"
    );
    assert!(
        ci.contains("fail_ci_if_error: true"),
        "Codecov upload failures should fail CI"
    );
}

#[test]
fn release_publish_crate_is_gated_on_registry_token() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release-please.yml"))
            .expect("read release workflow");

    assert!(
        !workflow.contains("secrets.CARGO_REGISTRY_TOKEN != ''"),
        "GitHub Actions does not support secrets directly in if conditionals"
    );
    assert!(
        workflow.contains("if: env.CARGO_REGISTRY_TOKEN != ''"),
        "crates.io publish step must be explicitly gated on CARGO_REGISTRY_TOKEN"
    );
}

#[test]
fn release_workflow_supports_first_release_dispatch() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release-please.yml"))
            .expect("read release workflow");

    assert!(
        workflow.contains("description: \"Release tag to build and publish"),
        "manual release dispatch should accept a tag to create or reuse"
    );
    assert!(
        workflow.contains("git tag \"$TAG\" \"$GITHUB_SHA\""),
        "manual release dispatch should create the tag when it does not exist"
    );
    assert!(
        workflow.contains("git push origin \"refs/tags/$TAG\""),
        "manual release dispatch should push the created release tag"
    );
}

#[test]
fn release_publish_waits_for_library_before_cli_crate() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release-please.yml"))
            .expect("read release workflow");

    assert!(
        workflow.contains("for attempt in 1 2 3 4 5 6"),
        "CLI crate publish should retry while crates.io indexes the freshly published library"
    );
    assert!(
        workflow.contains("cargo search koban --limit 1"),
        "CLI crate publish should refresh the crates.io index between publish attempts"
    );
    assert!(
        workflow.contains("publish_with_retry koban-cli"),
        "release workflow must still publish the CLI crate"
    );
}

fn collect_rust_file_failures(dir: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
    {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rust_file_failures(&path, failures);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let max_lines = if path
            .components()
            .any(|component| component.as_os_str() == "tests")
        {
            MAX_TEST_LINES
        } else {
            MAX_SOURCE_LINES
        };
        let contents = fs::read_to_string(&path).expect("read source file");
        let line_count = contents.lines().count();
        if line_count > max_lines {
            failures.push(format!(
                "{} has {line_count} lines, above the {max_lines} line budget",
                path.strip_prefix(workspace_root())
                    .unwrap_or(&path)
                    .display()
            ));
        }
    }
}
