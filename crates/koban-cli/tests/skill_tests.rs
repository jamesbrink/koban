use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn koban() -> Command {
    Command::cargo_bin("koban").expect("koban binary")
}

fn embedded_skill() -> &'static str {
    include_str!("../src/skill/SKILL.md")
}

#[test]
fn show_prints_the_embedded_skill() {
    koban()
        .args(["skill", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: koban"))
        .stdout(predicate::str::contains("koban skill install"));
}

#[test]
fn project_install_defaults_to_claude_and_agents_verbatim() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "install", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed 2 koban skill file"));

    let claude = dir.path().join(".claude/skills/koban/SKILL.md");
    let agents = dir.path().join(".agents/skills/koban/SKILL.md");
    assert_eq!(
        std::fs::read_to_string(claude).expect("claude"),
        embedded_skill()
    );
    assert_eq!(
        std::fs::read_to_string(agents).expect("agents"),
        embedded_skill()
    );
}

#[test]
fn user_install_without_detected_agents_falls_back_to_generic_agents_dir() {
    let home = tempdir().expect("home");
    koban()
        .env("HOME", home.path())
        .args(["skill", "install"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".agents/skills/koban/SKILL.md"));

    assert_eq!(
        std::fs::read_to_string(home.path().join(".agents/skills/koban/SKILL.md"))
            .expect("generic skill"),
        embedded_skill()
    );
}

#[test]
fn user_install_targets_detected_agents() {
    let home = tempdir().expect("home");
    std::fs::create_dir(home.path().join(".codex")).expect("codex config");
    std::fs::create_dir(home.path().join(".cursor")).expect("cursor config");

    koban()
        .env("HOME", home.path())
        .args(["skill", "install"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".codex/skills/koban/SKILL.md"))
        .stdout(predicate::str::contains(".cursor/skills/koban/SKILL.md"));

    assert!(home.path().join(".codex/skills/koban/SKILL.md").exists());
    assert!(home.path().join(".cursor/skills/koban/SKILL.md").exists());
    assert!(!home.path().join(".agents/skills/koban/SKILL.md").exists());
}

#[test]
fn project_install_specific_agents_dedupes_shared_directory() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "install", "codex", "cursor", "gemini", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed 1 koban skill file"));

    assert!(dir.path().join(".agents/skills/koban/SKILL.md").exists());
}

#[test]
fn explicit_openclaw_project_install_uses_workspace_skills_dir() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "install", "openclaw", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(dir.path().join("skills/koban/SKILL.md")).expect("openclaw skill"),
        embedded_skill()
    );
    assert!(!dir.path().join(".agents/skills/koban/SKILL.md").exists());
}

#[test]
fn install_overwrites_existing_skill_file_but_not_siblings() {
    let dir = tempdir().expect("tempdir");
    let skill_dir = dir.path().join(".claude/skills/koban");
    let skill_path = skill_dir.join("SKILL.md");
    let sibling = skill_dir.join("notes.md");
    std::fs::create_dir_all(&skill_dir).expect("mkdir");
    std::fs::write(&skill_path, "stale").expect("seed");
    std::fs::write(&sibling, "keep me").expect("sibling");

    koban()
        .args(["skill", "install", "claude", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(skill_path).expect("skill"),
        embedded_skill()
    );
    assert_eq!(
        std::fs::read_to_string(sibling).expect("sibling"),
        "keep me"
    );
}

#[test]
fn uninstall_removes_only_skill_file_and_empty_skill_dir() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "install", "claude", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    koban()
        .args(["skill", "uninstall", "claude", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 1 koban skill file"));

    assert!(!dir.path().join(".claude/skills/koban/SKILL.md").exists());
    assert!(!dir.path().join(".claude/skills/koban").exists());
}

#[test]
fn list_reports_supported_agents_and_status() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "install", "claude", "--dir"])
        .arg(dir.path())
        .assert()
        .success();

    let mut cmd = koban();
    cmd.current_dir(dir.path());
    cmd.args(["skill", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Code"))
        .stdout(predicate::str::contains("[installed]"));
}

#[test]
fn json_install_lists_written_paths() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["--output", "json", "skill", "install", "claude", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"install\""))
        .stdout(predicate::str::contains("SKILL.md"));
}

#[test]
fn hidden_generate_writes_review_copy_for_compatibility() {
    let dir = tempdir().expect("tempdir");
    koban()
        .args(["skill", "generate", "--dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Review copy written"));

    assert_eq!(
        std::fs::read_to_string(dir.path().join("koban/SKILL.md")).expect("review copy"),
        embedded_skill()
    );
}

#[test]
fn checked_in_openclaw_skill_copy_matches_embedded_skill() {
    assert_eq!(
        include_str!("../../../skills/koban/SKILL.md"),
        embedded_skill()
    );
}
