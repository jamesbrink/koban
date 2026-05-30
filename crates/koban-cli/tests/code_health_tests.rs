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
fn release_crate_publish_is_wired_to_registry_token() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/release-plz.yml"))
        .expect("read release workflow");

    assert!(
        !workflow.contains("secrets.CARGO_REGISTRY_TOKEN != ''"),
        "GitHub Actions does not support secrets directly in if conditionals"
    );
    assert!(
        workflow.contains("CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}"),
        "release-plz must receive CARGO_REGISTRY_TOKEN to publish crates to crates.io"
    );
}

#[test]
fn release_workflow_supports_first_release_dispatch() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/release-plz.yml"))
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
fn release_plz_publishes_both_crates_with_stable_tag_scheme() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/release-plz.yml"))
        .expect("read release workflow");
    // release-plz drives crates.io publishing (library before CLI, ordered
    // automatically) and opens the release PR.
    assert!(
        workflow.contains("command: release-pr"),
        "release-plz must open the release PR"
    );
    assert!(
        workflow.contains("command: release"),
        "release-plz must run the release command to publish crates and create tags"
    );

    let config = fs::read_to_string(workspace_root().join("release-plz.toml"))
        .expect("read release-plz config");
    // install.sh and `koban update` depend on the prefix-free CLI tag carrying
    // the binaries, and the library on the `koban-v*` tag — protect that contract.
    assert!(
        config.contains("name = \"koban-cli\"")
            && config.contains("git_tag_name = \"v{{ version }}\""),
        "koban-cli must own the prefix-free vX.Y.Z tag that carries binary assets"
    );
    assert!(
        config.contains("name = \"koban\"")
            && config.contains("git_tag_name = \"koban-v{{ version }}\""),
        "the koban library must use the koban-v* tag"
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
