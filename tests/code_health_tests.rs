use std::{fs, path::Path};

const MAX_SOURCE_LINES: usize = 900;
const MAX_TEST_LINES: usize = 1_100;

#[test]
fn rust_source_files_stay_small_enough_to_review() {
    let mut failures = Vec::new();

    collect_rust_file_failures(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut failures,
    );
    collect_rust_file_failures(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        &mut failures,
    );

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn ci_keeps_coverage_gate_enforced() {
    let ci =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml"))
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
    let workflow = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-please.yml"),
    )
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

        let max_lines = if path.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"))
            || path
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
                path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .unwrap_or(&path)
                    .display()
            ));
        }
    }
}
