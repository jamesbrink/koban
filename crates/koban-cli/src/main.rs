mod cli;
mod commands;
mod invoice;
mod payload;
mod render;
mod update;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use clap_complete_nushell::Nushell;

use cli::{Cli, Commands, CompletionShell};

/// Build the clap command tree for the CLI (used for shell completions).
fn command() -> clap::Command {
    Cli::command()
}

#[tokio::main]
async fn main() {
    clap_complete::CompleteEnv::with_factory(command).complete();

    let cli = Cli::parse();

    if let Some(Commands::Completions { shell }) = &cli.command {
        print_completions(shell);
        return;
    }

    match commands::execute(cli).await {
        Ok(output) if output.is_empty() => {}
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{:?}", miette::Report::new(error));
            std::process::exit(1);
        }
    }
}

fn print_completions(shell: &CompletionShell) {
    let mut command = command();
    let bin_name = command.get_name().to_string();

    match shell {
        CompletionShell::Bash => {
            generate(Shell::Bash, &mut command, bin_name, &mut std::io::stdout())
        }
        CompletionShell::Elvish => generate(
            Shell::Elvish,
            &mut command,
            bin_name,
            &mut std::io::stdout(),
        ),
        CompletionShell::Fish => {
            generate(Shell::Fish, &mut command, bin_name, &mut std::io::stdout())
        }
        CompletionShell::PowerShell => {
            generate(
                Shell::PowerShell,
                &mut command,
                bin_name,
                &mut std::io::stdout(),
            );
        }
        CompletionShell::Zsh => {
            generate(Shell::Zsh, &mut command, bin_name, &mut std::io::stdout())
        }
        CompletionShell::Nushell => {
            generate(Nushell, &mut command, bin_name, &mut std::io::stdout());
        }
    }
}

#[cfg(test)]
mod tests;
