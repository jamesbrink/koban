//! `koban skill` — generate or install an agent skill describing koban.
//!
//! `generate` writes files into a directory for review; `install` writes them
//! into live harness configuration locations (project by default, or the
//! user-level config with `--global`). One shared skill body is wrapped in
//! target-appropriate frontmatter (see [`templates`]).

mod templates;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use koban::{KobanError, Result};
use serde_json::{Value, json};

use crate::cli::{OutputFormat, SkillArgs, SkillCommand, SkillTarget};
use templates::Flavor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Generate,
    Install,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::Generate => "generate",
            Mode::Install => "install",
        }
    }
}

/// A single file to write.
struct Artifact {
    path: PathBuf,
    content: Content,
}

enum Content {
    /// A text file written verbatim (honors `--force` for overwrites).
    Text(String),
    /// Binary bytes, e.g. a zip bundle (honors `--force`).
    Binary(Vec<u8>),
    /// A markdown block spliced idempotently into a shared `AGENTS.md`.
    AgentsBlock(String),
}

pub fn execute(output: OutputFormat, command: SkillCommand) -> Result<String> {
    let (mode, args) = match command {
        SkillCommand::Generate(args) => (Mode::Generate, args),
        SkillCommand::Install(args) => (Mode::Install, args),
    };

    let targets = expand_targets(&args.target);
    let command_list = templates::command_list();

    let mut artifacts = Vec::new();
    for target in targets {
        artifacts.extend(plan_target(target, mode, &args, &command_list)?);
    }

    let mut written = Vec::new();
    for artifact in &artifacts {
        write_artifact(artifact, args.force)?;
        written.push(artifact.path.display().to_string());
    }

    render_summary(output, mode, &written)
}

/// Expand `all` and de-duplicate while preserving order.
fn expand_targets(requested: &[SkillTarget]) -> Vec<SkillTarget> {
    let mut targets = Vec::new();
    let push = |target: SkillTarget, targets: &mut Vec<SkillTarget>| {
        if !targets.contains(&target) {
            targets.push(target);
        }
    };

    for &target in requested {
        match target {
            SkillTarget::All => {
                push(SkillTarget::ClaudeCode, &mut targets);
                push(SkillTarget::Codex, &mut targets);
                push(SkillTarget::AgentsMd, &mut targets);
            }
            other => push(other, &mut targets),
        }
    }
    targets
}

/// Build the artifacts for one target.
fn plan_target(
    target: SkillTarget,
    mode: Mode,
    args: &SkillArgs,
    command_list: &str,
) -> Result<Vec<Artifact>> {
    match target {
        SkillTarget::ClaudeCode => {
            let base = base_dir(mode, args, ".claude", ".claude")?;
            Ok(vec![Artifact {
                path: base.join("skills").join("koban").join("SKILL.md"),
                content: Content::Text(templates::skill_md(Flavor::ClaudeCode, command_list)),
            }])
        }
        SkillTarget::Codex => {
            let base = base_dir(mode, args, ".agents", ".agents")?;
            Ok(vec![Artifact {
                path: base.join("skills").join("koban").join("SKILL.md"),
                content: Content::Text(templates::skill_md(Flavor::Codex, command_list)),
            }])
        }
        SkillTarget::Pi => {
            let base = base_dir(mode, args, ".pi", ".pi/agent")?;
            Ok(vec![Artifact {
                path: base.join("skills").join("koban").join("SKILL.md"),
                content: Content::Text(templates::skill_md(Flavor::Pi, command_list)),
            }])
        }
        SkillTarget::AgentsMd => {
            let base = base_dir(mode, args, "", ".codex")?;
            Ok(vec![Artifact {
                path: base.join("AGENTS.md"),
                content: Content::AgentsBlock(templates::agents_block(command_list)),
            }])
        }
        SkillTarget::Cursor => {
            let base = base_dir(mode, args, ".cursor", ".cursor")?;
            Ok(vec![Artifact {
                path: base.join("rules").join("koban.mdc"),
                content: Content::Text(templates::cursor_mdc(command_list)),
            }])
        }
        SkillTarget::ClaudeDesktop => {
            // The upload bundle and the plugin are project artifacts; --global
            // does not relocate them.
            let base = bundle_base(mode, args)?;
            let skill = templates::skill_md(Flavor::ClaudeCode, command_list);
            Ok(vec![Artifact {
                path: base.join("koban.zip"),
                content: Content::Binary(zip_skill(&skill)?),
            }])
        }
        SkillTarget::Plugin => {
            let base = bundle_base(mode, args)?.join("koban");
            Ok(vec![
                Artifact {
                    path: base.join(".claude-plugin").join("plugin.json"),
                    content: Content::Text(templates::plugin_json()),
                },
                Artifact {
                    path: base.join("skills").join("koban").join("SKILL.md"),
                    content: Content::Text(templates::skill_md(Flavor::ClaudeCode, command_list)),
                },
            ])
        }
        SkillTarget::All => unreachable!("expand_targets removes All"),
    }
}

/// Directory that contains a target's `skills/` (or holds `AGENTS.md`).
fn base_dir(
    mode: Mode,
    args: &SkillArgs,
    project_prefix: &str,
    global_prefix: &str,
) -> Result<PathBuf> {
    match mode {
        Mode::Generate => {
            let root = args
                .dir
                .clone()
                .unwrap_or_else(|| PathBuf::from("koban-skills"));
            Ok(join_prefix(root, project_prefix))
        }
        Mode::Install if args.global => {
            let root = match &args.dir {
                Some(dir) => dir.clone(),
                None => home_dir()?,
            };
            Ok(join_prefix(root, global_prefix))
        }
        Mode::Install => {
            let root = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
            Ok(join_prefix(root, project_prefix))
        }
    }
}

/// Root for project-shaped bundles (zip, plugin) that `--global` does not move.
fn bundle_base(mode: Mode, args: &SkillArgs) -> Result<PathBuf> {
    let default = match mode {
        Mode::Generate => PathBuf::from("koban-skills"),
        Mode::Install => PathBuf::from("."),
    };
    Ok(args.dir.clone().unwrap_or(default))
}

fn join_prefix(root: PathBuf, prefix: &str) -> PathBuf {
    if prefix.is_empty() {
        root
    } else {
        root.join(prefix)
    }
}

fn home_dir() -> Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .ok_or_else(|| skill_error("could not determine your home directory for --global install"))
}

fn write_artifact(artifact: &Artifact, force: bool) -> Result<()> {
    if let Some(parent) = artifact.path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|source| {
            skill_error(format!("could not create {}: {source}", parent.display()))
        })?;
    }

    match &artifact.content {
        Content::Text(text) => write_new_file(&artifact.path, text.as_bytes(), force),
        Content::Binary(bytes) => write_new_file(&artifact.path, bytes, force),
        Content::AgentsBlock(block) => splice_agents(&artifact.path, block),
    }
}

/// Write a file, refusing to clobber an existing one unless `force` is set.
fn write_new_file(path: &Path, bytes: &[u8], force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(skill_error(format!(
            "{} already exists (use --force to overwrite)",
            path.display()
        )));
    }
    fs::write(path, bytes)
        .map_err(|source| skill_error(format!("could not write {}: {source}", path.display())))
}

/// Splice the koban block into `AGENTS.md`, replacing an existing marked block
/// in place so re-running never duplicates it.
fn splice_agents(path: &Path, block: &str) -> Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();

    let updated = match (
        existing.find(templates::AGENTS_START),
        existing.find(templates::AGENTS_END),
    ) {
        (Some(start), Some(end)) if end > start => {
            let end = end + templates::AGENTS_END.len();
            format!("{}{}{}", &existing[..start], block, &existing[end..])
        }
        _ if existing.trim().is_empty() => format!("{block}\n"),
        _ => format!("{}\n\n{block}\n", existing.trim_end()),
    };

    fs::write(path, updated)
        .map_err(|source| skill_error(format!("could not write {}: {source}", path.display())))
}

/// Build a Claude Desktop upload bundle: a zip containing `koban/SKILL.md`.
fn zip_skill(skill: &str) -> Result<Vec<u8>> {
    use zip::write::SimpleFileOptions;

    let mut buffer = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.add_directory("koban/", options)
            .map_err(|source| skill_error(format!("zip: {source}")))?;
        zip.start_file("koban/SKILL.md", options)
            .map_err(|source| skill_error(format!("zip: {source}")))?;
        zip.write_all(skill.as_bytes())
            .map_err(|source| skill_error(format!("zip: {source}")))?;
        zip.finish()
            .map_err(|source| skill_error(format!("zip: {source}")))?;
    }
    Ok(buffer)
}

fn render_summary(output: OutputFormat, mode: Mode, written: &[String]) -> Result<String> {
    match output {
        OutputFormat::Json => to_json(&json!({
            "mode": mode.label(),
            "written": written,
        })),
        OutputFormat::Table => {
            let mut lines = vec![format!(
                "Wrote {} file(s) ({}):",
                written.len(),
                mode.label()
            )];
            for path in written {
                lines.push(format!("  {path}"));
            }
            if written.iter().any(|path| path.ends_with("koban.zip")) {
                lines.push(
                    "Upload koban.zip in Claude Desktop via Settings > Capabilities > Skills."
                        .to_string(),
                );
            }
            Ok(lines.join("\n"))
        }
    }
}

fn to_json(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map_err(|source| skill_error(format!("could not render JSON: {source}")))
}

fn skill_error(message: impl Into<String>) -> KobanError {
    KobanError::File {
        message: message.into(),
    }
}
