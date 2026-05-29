use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn koban() -> Command {
    Command::cargo_bin("koban").expect("koban binary")
}

#[test]
fn generate_all_emits_claude_codex_and_agents_with_correct_frontmatter() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "generate", "--target", "all", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 file(s)"));

    // Claude Code: name + description + allowed-tools.
    let claude =
        std::fs::read_to_string(dir.path().join(".claude/skills/koban/SKILL.md")).expect("claude");
    assert!(claude.starts_with("---\n"));
    assert!(claude.contains("name: koban"));
    assert!(claude.contains("description: Read and write Invoice Ninja"));
    assert!(claude.contains("allowed-tools: Bash(koban:*)"));

    // Codex: minimal Agent Skills subset (no Claude-specific keys).
    let codex =
        std::fs::read_to_string(dir.path().join(".agents/skills/koban/SKILL.md")).expect("codex");
    assert!(codex.contains("name: koban"));
    assert!(codex.contains("description: Read and write Invoice Ninja"));
    assert!(
        !codex.contains("allowed-tools"),
        "codex frontmatter should stay the minimal subset"
    );

    // AGENTS.md: no frontmatter, wrapped in idempotency markers.
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).expect("agents");
    assert!(!agents.starts_with("---"));
    assert!(agents.contains("<!-- koban:start -->"));
    assert!(agents.contains("<!-- koban:end -->"));
}

#[test]
fn generate_optional_targets_emit_their_own_formats() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args([
            "skill",
            "generate",
            "--target",
            "cursor",
            "--target",
            "plugin",
            "--target",
            "claude-desktop",
            "--dir",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // Cursor: its own .mdc frontmatter schema.
    let cursor =
        std::fs::read_to_string(dir.path().join(".cursor/rules/koban.mdc")).expect("cursor");
    assert!(cursor.contains("alwaysApply: false"));
    assert!(cursor.contains("globs:"));

    // Plugin: valid JSON manifest naming the koban plugin.
    let plugin = std::fs::read_to_string(dir.path().join("koban/.claude-plugin/plugin.json"))
        .expect("plugin");
    let manifest: serde_json::Value = serde_json::from_str(&plugin).expect("valid plugin json");
    assert_eq!(manifest["name"], "koban");
    assert!(dir.path().join("koban/skills/koban/SKILL.md").exists());

    // Claude Desktop: a real zip bundle (PK magic bytes).
    let zip = std::fs::read(dir.path().join("koban.zip")).expect("zip");
    assert_eq!(&zip[..2], b"PK");
}

#[test]
fn install_agents_md_is_idempotent() {
    let dir = tempdir().expect("tempdir");

    for _ in 0..2 {
        koban()
            .args(["skill", "install", "--target", "agents-md", "--dir"])
            .arg(dir.path())
            .assert()
            .success();
    }

    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).expect("agents");
    assert_eq!(agents.matches("<!-- koban:start -->").count(), 1);
    assert_eq!(agents.matches("<!-- koban:end -->").count(), 1);
}

#[test]
fn install_agents_md_preserves_surrounding_content() {
    let dir = tempdir().expect("tempdir");
    let agents_path = dir.path().join("AGENTS.md");
    std::fs::write(&agents_path, "# My project\n\nExisting guidance.\n").expect("seed");

    koban()
        .args(["skill", "install", "--target", "agents-md", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    let agents = std::fs::read_to_string(&agents_path).expect("agents");
    assert!(agents.contains("# My project"));
    assert!(agents.contains("Existing guidance."));
    assert!(agents.contains("<!-- koban:start -->"));
}

#[test]
fn install_refuses_to_overwrite_without_force() {
    let dir = tempdir().expect("tempdir");
    let skill_path = dir.path().join(".claude/skills/koban/SKILL.md");
    std::fs::create_dir_all(skill_path.parent().unwrap()).expect("mkdir");
    std::fs::write(&skill_path, "existing").expect("seed");

    koban()
        .args(["skill", "install", "--target", "claude-code", "--dir"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // --force overwrites.
    koban()
        .args([
            "skill",
            "install",
            "--target",
            "claude-code",
            "--force",
            "--dir",
        ])
        .arg(dir.path())
        .assert()
        .success();
    assert!(std::fs::read_to_string(&skill_path).expect("rewritten") != "existing");
}

#[test]
fn generate_json_output_lists_written_paths() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args([
            "--output",
            "json",
            "skill",
            "generate",
            "--target",
            "claude-code",
            "--dir",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"generate\""))
        .stdout(predicate::str::contains("SKILL.md"));
}
