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
fn skill_documents_filter_traps_and_status_codes() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "generate", "--target", "all", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    let claude =
        std::fs::read_to_string(dir.path().join(".claude/skills/koban/SKILL.md")).expect("claude");
    // The silent-ignore filter trap and the canonical outstanding recipe.
    assert!(claude.contains("silently ignored"));
    assert!(claude.contains("client_status=unpaid"));
    // The status_id mapping that statics does not provide.
    assert!(claude.contains("status_id"));
    assert!(claude.contains("4         | paid"));
    // Reporting runners are accurately documented as POST/confirmation-gated.
    assert!(claude.contains("treated as mutations"));

    // The compact AGENTS.md block carries the same filter warning.
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).expect("agents");
    assert!(agents.contains("client_status=unpaid"));
    assert!(agents.contains("silently ignored"));
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

#[test]
fn generate_hints_how_to_install_the_files() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "generate", "--target", "claude-code", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        // Names the install command and the manual-copy escape hatch.
        .stdout(predicate::str::contains("koban skill install"))
        .stdout(predicate::str::contains("manually"));
}

#[test]
fn generate_json_includes_the_install_hint() {
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
        .stdout(predicate::str::contains("\"hint\""))
        .stdout(predicate::str::contains("koban skill install"));
}

#[test]
fn install_does_not_print_the_generate_hint() {
    let dir = tempdir().expect("tempdir");
    // The hint is for review output only; install writes to live locations and
    // should not nudge the user to run install again.
    koban()
        .args(["skill", "install", "--target", "claude-code", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("koban skill install").not());
}

#[test]
fn generate_openclaw_emits_workspace_skill_with_single_line_metadata_gate() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "generate", "--target", "openclaw", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    // OpenClaw workspace layout: <root>/skills/koban/SKILL.md (no dot-prefix).
    let skill =
        std::fs::read_to_string(dir.path().join("skills/koban/SKILL.md")).expect("openclaw skill");
    assert!(skill.starts_with("---\n"));
    assert!(skill.contains("name: koban"));
    assert!(skill.contains("description: Read and write Invoice Ninja"));

    // OpenClaw's parser only accepts single-line frontmatter keys, so the
    // metadata gate must be a single-line JSON object, and it must gate on the
    // koban binary being present on PATH.
    let metadata_line = skill
        .lines()
        .find(|line| line.starts_with("metadata:"))
        .expect("single-line metadata key");
    assert!(metadata_line.contains("\"openclaw\""));
    assert!(metadata_line.contains("\"bins\""));
    assert!(metadata_line.contains("\"koban\""));

    // It is a SKILL.md flavor, not a Claude Code one: no allowed-tools key.
    assert!(!skill.contains("allowed-tools"));
}

#[test]
fn target_all_excludes_openclaw() {
    let dir = tempdir().expect("tempdir");
    // OpenClaw stays opt-in; `all` keeps its documented three-target contract.
    koban()
        .args(["skill", "generate", "--target", "all", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 file(s)"));
    assert!(!dir.path().join("skills/koban/SKILL.md").exists());
}
