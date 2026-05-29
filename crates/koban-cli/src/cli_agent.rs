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

/// Subcommands for emitting the koban agent skill.
#[derive(Debug, Subcommand)]
#[command(after_help = "\
Examples:
  koban skill generate
  koban skill install --target claude-code
  koban skill install --global --target all")]
pub enum SkillCommand {
    /// Write skill files to a directory for review (default ./koban-skills)
    Generate(SkillArgs),

    /// Write skill files into live harness configuration locations
    Install(SkillArgs),
}

#[derive(Debug, Args)]
pub struct SkillArgs {
    /// Harness targets to emit (repeatable). Defaults to `all`
    #[arg(long, value_enum, default_value = "all", action = clap::ArgAction::Append)]
    pub target: Vec<SkillTarget>,

    /// Output root (generate) or base directory override (install)
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Install into the user-level config (home) instead of the current project
    #[arg(long)]
    pub global: bool,

    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,
}

/// AI harness targets the skill generator knows how to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SkillTarget {
    /// Claude Code (`.claude/skills/koban/SKILL.md`)
    ClaudeCode,
    /// OpenAI Codex CLI / pi (`.agents/skills/koban/SKILL.md`)
    Codex,
    /// pi coding agent (`.pi/skills/koban/SKILL.md`)
    Pi,
    /// AGENTS.md block (Cursor, Windsurf, Gemini, Aider, Copilot, Zed, ...)
    AgentsMd,
    /// Claude Desktop upload bundle (`koban.zip`)
    ClaudeDesktop,
    /// Cursor project rule (`.cursor/rules/koban.mdc`)
    Cursor,
    /// Claude Code plugin (`.claude-plugin/plugin.json` + skill)
    Plugin,
    /// claude-code + codex + agents-md (the practical default bundle)
    All,
}
