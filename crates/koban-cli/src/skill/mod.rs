//! `koban skill` — install the embedded Agent Skills standard `SKILL.md`.
//!
//! The installed file is byte-identical to [`SKILL_MD`]. Installs write only
//! `<skills dir>/koban/SKILL.md`, overwrite that file on rerun, and never touch
//! sibling files. User-wide installs default to detected agents; project installs
//! default to `.claude/skills` + `.agents/skills`, which covers the supported
//! project-level readers.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use koban::{KobanError, Result};
use serde_json::{Value, json};

use crate::cli::{
    OutputFormat, SkillAgent, SkillCommand, SkillGenerateArgs, SkillInstallArgs, SkillListArgs,
    SkillUninstallArgs,
};

/// The embedded skill content, installed verbatim.
pub const SKILL_MD: &str = include_str!("SKILL.md");

const SKILL_DIR_NAME: &str = "koban";

/// Per-agent skill locations. Paths are stored as segments so joins are
/// platform-correct and easy to audit.
struct AgentSpec {
    agent: SkillAgent,
    label: &'static str,
    name: &'static str,
    detect: &'static [&'static str],
    user_skills: &'static [&'static str],
    project_skills: &'static [&'static str],
}

const AGENTS: &[AgentSpec] = &[
    AgentSpec {
        agent: SkillAgent::Claude,
        label: "Claude Code",
        name: "claude",
        detect: &[".claude"],
        user_skills: &[".claude", "skills"],
        project_skills: &[".claude", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Codex,
        label: "OpenAI Codex CLI",
        name: "codex",
        detect: &[".codex"],
        user_skills: &[".codex", "skills"],
        project_skills: &[".agents", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Pi,
        label: "pi coding agent",
        name: "pi",
        detect: &[".pi"],
        user_skills: &[".pi", "agent", "skills"],
        project_skills: &[".pi", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::OpenClaw,
        label: "OpenClaw",
        name: "openclaw",
        detect: &[".openclaw"],
        user_skills: &[".openclaw", "skills"],
        project_skills: &["skills"],
    },
    AgentSpec {
        agent: SkillAgent::Copilot,
        label: "GitHub Copilot CLI",
        name: "copilot",
        detect: &[".copilot"],
        user_skills: &[".copilot", "skills"],
        project_skills: &[".github", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Cursor,
        label: "Cursor",
        name: "cursor",
        detect: &[".cursor"],
        user_skills: &[".cursor", "skills"],
        project_skills: &[".agents", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Gemini,
        label: "Gemini CLI",
        name: "gemini",
        detect: &[".gemini"],
        user_skills: &[".gemini", "skills"],
        project_skills: &[".agents", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Amp,
        label: "Amp",
        name: "amp",
        detect: &[".config", "amp"],
        user_skills: &[".config", "amp", "skills"],
        project_skills: &[".agents", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Goose,
        label: "Goose",
        name: "goose",
        detect: &[".config", "goose"],
        user_skills: &[".config", "goose", "skills"],
        project_skills: &[".agents", "skills"],
    },
    AgentSpec {
        agent: SkillAgent::Agents,
        label: "Agent Skills (generic)",
        name: "agents",
        detect: &[".agents"],
        user_skills: &[".agents", "skills"],
        project_skills: &[".agents", "skills"],
    },
];

pub fn execute(output: OutputFormat, command: SkillCommand) -> Result<String> {
    match command {
        SkillCommand::List(args) => list(output, &args),
        SkillCommand::Install(args) => install(output, &args),
        SkillCommand::Uninstall(args) => uninstall(output, &args),
        SkillCommand::Show => Ok(SKILL_MD.to_string()),
        SkillCommand::Generate(args) => generate(output, &args),
    }
}

impl SkillAgent {
    fn spec(self) -> &'static AgentSpec {
        AGENTS
            .iter()
            .find(|spec| spec.agent == self)
            .expect("every SkillAgent variant has an AgentSpec entry")
    }

    fn user_skill_dir(self, home: &Path) -> PathBuf {
        join_segments(home, self.spec().user_skills).join(SKILL_DIR_NAME)
    }

    fn project_skill_dir(self, root: &Path) -> PathBuf {
        join_segments(root, self.spec().project_skills).join(SKILL_DIR_NAME)
    }

    fn is_detected(self, home: &Path) -> bool {
        join_segments(home, self.spec().detect).exists()
    }
}

enum Scope {
    User(PathBuf),
    Project(PathBuf),
}

fn install(output: OutputFormat, args: &SkillInstallArgs) -> Result<String> {
    let scope = resolve_scope(args.project, &args.dir)?;
    let agents = resolve_install_agents(&scope, args);
    let targets = dedupe_targets(&scope, &agents);

    let mut written = Vec::new();
    let mut summaries = Vec::new();
    for (dir, names) in targets {
        let target = write_skill_md(&dir)?;
        written.push(target.display().to_string());
        summaries.push(json!({
            "path": target.display().to_string(),
            "agents": names,
        }));
    }

    render_write_summary(output, "install", &written, summaries, None)
}

fn uninstall(output: OutputFormat, args: &SkillUninstallArgs) -> Result<String> {
    let scope = resolve_scope(args.project, &args.dir)?;
    let agents: Vec<SkillAgent> = if args.agents.is_empty() {
        AGENTS.iter().map(|spec| spec.agent).collect()
    } else {
        args.agents.clone()
    };

    let mut removed = Vec::new();
    for (dir, _names) in dedupe_targets(&scope, &agents) {
        let skill_md = dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        fs::remove_file(&skill_md).map_err(|source| {
            skill_error(format!("could not remove {}: {source}", skill_md.display()))
        })?;
        let _ = fs::remove_dir(&dir);
        removed.push(skill_md.display().to_string());
    }

    render_write_summary(output, "uninstall", &removed, Vec::new(), None)
}

fn list(output: OutputFormat, args: &SkillListArgs) -> Result<String> {
    let home = home_dir().ok();
    let project_root = std::env::current_dir().ok();

    let rows: Vec<Value> = AGENTS
        .iter()
        .map(|spec| {
            let detected = home
                .as_deref()
                .map(|home| spec.agent.is_detected(home))
                .unwrap_or(false);
            let user_skill = home
                .as_deref()
                .map(|home| status_value(spec.agent.user_skill_dir(home)));
            let project_skill = project_root
                .as_deref()
                .map(|root| status_value(spec.agent.project_skill_dir(root)));
            json!({
                "agent": spec.name,
                "label": spec.label,
                "detected": detected,
                "user_skill": user_skill,
                "project_skill": project_skill,
            })
        })
        .collect();

    match output {
        OutputFormat::Json => to_json(&json!({ "agents": rows })),
        OutputFormat::Table => Ok(render_list_table(&rows, args.ascii)),
    }
}

fn generate(output: OutputFormat, args: &SkillGenerateArgs) -> Result<String> {
    let dir = expand_tilde(&args.dir).join(SKILL_DIR_NAME);
    let target = write_skill_md(&dir)?;
    let hint = "Review copy written. Activate it with `koban skill install`, or inspect the embedded skill with `koban skill show`.";
    render_write_summary(
        output,
        "generate",
        &[target.display().to_string()],
        Vec::new(),
        Some(hint),
    )
}

fn resolve_install_agents(scope: &Scope, args: &SkillInstallArgs) -> Vec<SkillAgent> {
    if !args.agents.is_empty() {
        return args.agents.clone();
    }
    if args.all {
        return AGENTS.iter().map(|spec| spec.agent).collect();
    }

    match scope {
        Scope::User(home) => {
            let detected: Vec<SkillAgent> = AGENTS
                .iter()
                .map(|spec| spec.agent)
                .filter(|agent| *agent != SkillAgent::Agents && agent.is_detected(home))
                .collect();
            if detected.is_empty() {
                vec![SkillAgent::Agents]
            } else {
                detected
            }
        }
        Scope::Project(_) => vec![SkillAgent::Claude, SkillAgent::Agents],
    }
}

fn resolve_scope(project: bool, dir: &Option<PathBuf>) -> Result<Scope> {
    if let Some(dir) = dir {
        Ok(Scope::Project(expand_tilde(dir)))
    } else if project {
        std::env::current_dir()
            .map(Scope::Project)
            .map_err(|source| {
                skill_error(format!("could not determine current directory: {source}"))
            })
    } else {
        home_dir().map(Scope::User)
    }
}

fn dedupe_targets(scope: &Scope, agents: &[SkillAgent]) -> BTreeMap<PathBuf, Vec<&'static str>> {
    let mut targets: BTreeMap<PathBuf, Vec<&'static str>> = BTreeMap::new();
    for agent in agents {
        let dir = match scope {
            Scope::User(home) => agent.user_skill_dir(home),
            Scope::Project(root) => agent.project_skill_dir(root),
        };
        let names = targets.entry(dir).or_default();
        let name = agent.spec().name;
        if !names.contains(&name) {
            names.push(name);
        }
    }
    targets
}

fn write_skill_md(dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dir)
        .map_err(|source| skill_error(format!("could not create {}: {source}", dir.display())))?;

    let target = dir.join("SKILL.md");
    let tmp = dir.join(format!(".SKILL.md.tmp-{}", std::process::id()));
    {
        let mut file = fs::File::create(&tmp).map_err(|source| {
            skill_error(format!("could not create {}: {source}", tmp.display()))
        })?;
        file.write_all(SKILL_MD.as_bytes()).map_err(|source| {
            skill_error(format!("could not write {}: {source}", tmp.display()))
        })?;
        file.sync_all()
            .map_err(|source| skill_error(format!("could not sync {}: {source}", tmp.display())))?;
    }
    if target.exists() {
        fs::remove_file(&target).map_err(|source| {
            let _ = fs::remove_file(&tmp);
            skill_error(format!(
                "could not replace existing {}: {source}",
                target.display()
            ))
        })?;
    }
    fs::rename(&tmp, &target).map_err(|source| {
        let _ = fs::remove_file(&tmp);
        skill_error(format!("could not install {}: {source}", target.display()))
    })?;
    Ok(target)
}

fn render_write_summary(
    output: OutputFormat,
    mode: &str,
    paths: &[String],
    entries: Vec<Value>,
    hint: Option<&str>,
) -> Result<String> {
    match output {
        OutputFormat::Json => {
            let mut payload = json!({
                "mode": mode,
                "paths": paths,
            });
            if !entries.is_empty() {
                payload["entries"] = json!(entries);
            }
            if let Some(hint) = hint {
                payload["hint"] = json!(hint);
            }
            to_json(&payload)
        }
        OutputFormat::Table => {
            let verb = match mode {
                "install" => "Installed",
                "uninstall" => "Removed",
                "generate" => "Wrote",
                _ => "Wrote",
            };
            let mut lines = if paths.is_empty() {
                vec![format!("No koban skill files {mode}ed.")]
            } else {
                vec![format!("{verb} {} koban skill file(s):", paths.len())]
            };
            for path in paths {
                lines.push(format!("  {path}"));
            }
            if let Some(hint) = hint {
                lines.push(String::new());
                lines.push(hint.to_string());
            }
            Ok(lines.join("\n"))
        }
    }
}

fn render_list_table(rows: &[Value], ascii: bool) -> String {
    let sep = if ascii { " | " } else { " │ " };
    let mut lines = Vec::new();
    lines.push(["Agent", "Name", "Detected", "User skill", "Project skill"].join(sep));
    lines.push(["-----", "----", "--------", "----------", "-------------"].join(sep));
    for row in rows {
        lines.push(
            [
                row["label"].as_str().unwrap_or_default().to_string(),
                row["agent"].as_str().unwrap_or_default().to_string(),
                if row["detected"].as_bool().unwrap_or(false) {
                    "yes".to_string()
                } else {
                    "no".to_string()
                },
                status_cell(&row["user_skill"]),
                status_cell(&row["project_skill"]),
            ]
            .join(sep),
        );
    }
    lines.join("\n")
}

fn status_value(dir: PathBuf) -> Value {
    json!({
        "path": display_path(&dir),
        "installed": dir.join("SKILL.md").exists(),
    })
}

fn status_cell(value: &Value) -> String {
    let Some(path) = value.get("path").and_then(Value::as_str) else {
        return "-".to_string();
    };
    if value
        .get("installed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        format!("{path} [installed]")
    } else {
        path.to_string()
    }
}

fn join_segments(base: &Path, segments: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();
    for segment in segments {
        path.push(segment);
    }
    path
}

fn expand_tilde(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str == "~" {
        return home_dir().unwrap_or_else(|_| path.to_path_buf());
    }
    if let Some(rest) = path_str.strip_prefix("~/") {
        return home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|_| path.to_path_buf());
    }
    path.to_path_buf()
}

fn display_path(path: &Path) -> String {
    if let Ok(home) = home_dir()
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}

fn home_dir() -> Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .ok_or_else(|| skill_error("could not determine your home directory"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_path_mapping_is_pinned() {
        let home = Path::new("/home/u");
        let root = Path::new("/repo");
        let cases: &[(SkillAgent, &str, &str)] = &[
            (SkillAgent::Claude, ".claude/skills", ".claude/skills"),
            (SkillAgent::Codex, ".codex/skills", ".agents/skills"),
            (SkillAgent::Pi, ".pi/agent/skills", ".pi/skills"),
            (SkillAgent::OpenClaw, ".openclaw/skills", "skills"),
            (SkillAgent::Copilot, ".copilot/skills", ".github/skills"),
            (SkillAgent::Cursor, ".cursor/skills", ".agents/skills"),
            (SkillAgent::Gemini, ".gemini/skills", ".agents/skills"),
            (SkillAgent::Amp, ".config/amp/skills", ".agents/skills"),
            (SkillAgent::Goose, ".config/goose/skills", ".agents/skills"),
            (SkillAgent::Agents, ".agents/skills", ".agents/skills"),
        ];
        assert_eq!(cases.len(), AGENTS.len());
        for (agent, user, project) in cases {
            assert_eq!(agent.user_skill_dir(home), home.join(user).join("koban"));
            assert_eq!(
                agent.project_skill_dir(root),
                root.join(project).join("koban")
            );
        }
    }

    #[test]
    fn dedupe_shared_project_dirs() {
        let scope = Scope::Project(PathBuf::from("/repo"));
        let targets = dedupe_targets(
            &scope,
            &[SkillAgent::Codex, SkillAgent::Cursor, SkillAgent::Gemini],
        );
        assert_eq!(targets.len(), 1);
        let (dir, names) = targets.iter().next().unwrap();
        assert_eq!(dir, &PathBuf::from("/repo/.agents/skills/koban"));
        assert_eq!(names, &vec!["codex", "cursor", "gemini"]);
    }

    #[test]
    fn embedded_skill_frontmatter_is_standard() {
        let rest = SKILL_MD
            .strip_prefix("---\n")
            .expect("skill starts with frontmatter");
        let (frontmatter, body) = rest.split_once("\n---\n").expect("closing fence");
        assert!(frontmatter.contains("name: koban"));
        assert!(frontmatter.contains("description: Read and write Invoice Ninja"));
        assert!(frontmatter.contains("allowed-tools: Bash(koban:*)"));
        assert!(frontmatter.contains("metadata: {\"openclaw\""));
        assert!(body.contains("koban skill install"));
    }
}
