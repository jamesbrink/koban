use std::{fs, path::Path};

const MAX_SOURCE_LINES: usize = 900;
const MAX_TEST_LINES: usize = 1_300;

#[test]
fn rust_source_files_stay_small_enough_to_review() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut failures = Vec::new();

    for entry in fs::read_dir(&src_dir).expect("read src dir") {
        let path = entry.expect("src entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let max_lines = if path.file_name().and_then(|name| name.to_str()) == Some("tests.rs") {
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

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
