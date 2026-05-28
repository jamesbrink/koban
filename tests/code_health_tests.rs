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
