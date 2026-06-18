//! CLI argument types for the agent-facing commands (`auth` and `skill`).
//!
//! Kept out of `cli.rs` so that module stays focused on the Invoice Ninja
//! resource surface and within the per-file size budget.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

/// Subcommands for managing stored credentials.
#[derive(Debug, Subcommand)]
#[command(after_help = "\
Examples:
  koban auth login --token TOKEN
  koban auth login --keychain
  koban auth status
  koban auth logout")]
pub enum AuthCommand {
    /// Store an API token (verified against the API unless --no-verify)
    Login(AuthLoginArgs),

    /// Remove the stored token (the OS keychain entry too)
    Logout,

    /// Show which credential source is active (never prints the token)
    Status,
}

#[derive(Debug, Args)]
pub struct AuthLoginArgs {
    /// API token. If omitted, koban reads it from stdin (pipe) or prompts on a TTY
    #[arg(long)]
    pub token: Option<String>,

    /// Invoice Ninja base URL to store alongside the token
    #[arg(long, value_name = "URL")]
    pub base_url: Option<String>,

    /// Store the token in the OS keychain instead of the config file
    #[arg(long)]
    pub keychain: bool,

    /// Skip the live API check and save the token as-is
    #[arg(long)]
    pub no_verify: bool,
}

/// Subcommands for managing the koban agent skill.
#[derive(Debug, Subcommand)]
#[command(after_help = "\
Examples:
  koban skill install
  koban skill install --project
  koban skill install claude codex
  koban skill list
  koban skill show")]
pub enum SkillCommand {
    /// List supported agents, their skill paths, and install status
    List(SkillListArgs),

    /// Install the koban skill (user-wide by default, --project for project-level)
    Install(SkillInstallArgs),

    /// Remove installed koban skills
    Uninstall(SkillUninstallArgs),

    /// Print the embedded SKILL.md to stdout
    Show,

    /// Deprecated alias for `show`/`install`: write a review copy under a directory
    #[command(hide = true)]
    Generate(SkillGenerateArgs),
}

#[derive(Debug, Args)]
pub struct SkillListArgs {
    /// Use plain ASCII separators instead of Unicode table borders
    #[arg(long)]
    pub ascii: bool,
}

#[derive(Debug, Args)]
pub struct SkillInstallArgs {
    /// Agents to install for (default: detected agents, or claude + agents for project installs)
    #[arg(value_enum)]
    pub agents: Vec<SkillAgent>,

    /// Install into project-level skill directories under the current directory
    #[arg(long)]
    pub project: bool,

    /// Project directory to install into (implies --project)
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Install for every supported agent regardless of detection
    #[arg(long, conflicts_with = "agents")]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct SkillUninstallArgs {
    /// Agents to uninstall from (default: every known path in scope)
    #[arg(value_enum)]
    pub agents: Vec<SkillAgent>,

    /// Remove from project-level skill directories under the current directory
    #[arg(long)]
    pub project: bool,

    /// Project directory to uninstall from (implies --project)
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SkillGenerateArgs {
    /// Output root for the generated review copy
    #[arg(long, value_name = "PATH", default_value = "koban-skills")]
    pub dir: PathBuf,
}

/// AI coding agents with known Agent Skills directories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SkillAgent {
    /// Claude Code (~/.claude/skills, .claude/skills)
    Claude,
    /// OpenAI Codex CLI (~/.codex/skills, .agents/skills)
    Codex,
    /// pi coding agent (~/.pi/agent/skills, .pi/skills)
    Pi,
    /// OpenClaw (~/.openclaw/skills, skills/)
    #[value(name = "openclaw", alias = "open-claw")]
    OpenClaw,
    /// GitHub Copilot CLI (~/.copilot/skills, .github/skills)
    Copilot,
    /// Cursor (~/.cursor/skills, .agents/skills)
    Cursor,
    /// Gemini CLI (~/.gemini/skills, .agents/skills)
    Gemini,
    /// Amp (~/.config/amp/skills, .agents/skills)
    Amp,
    /// Goose (~/.config/goose/skills, .agents/skills)
    Goose,
    /// Generic cross-agent directory (~/.agents/skills, .agents/skills)
    Agents,
}
