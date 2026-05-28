//! Library crate for `koban`.
//!
//! The public surface is intentionally small while the CLI and Invoice Ninja
//! client model settle.

mod api;
mod cli;
mod commands;
mod config;
mod error;
mod invoice;
mod render;
mod resource;
mod update;
mod util;

pub use cli::{Cli, Commands, CompletionShell, OutputFormat};
pub use commands::{execute, execute_with_config};
pub use config::{API_TOKEN_ENV, BASE_URL_ENV, Config, DEFAULT_BASE_URL};
pub use error::{KobanError, Result};
pub use render::render_value;
pub use resource::Resource;
pub use util::redact;

pub fn command() -> clap::Command {
    <Cli as clap::CommandFactory>::command()
}

#[cfg(test)]
mod tests;
