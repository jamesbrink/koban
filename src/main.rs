use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Completions { shell }) => {
            let mut command = Cli::command();
            let bin_name = command.get_name().to_string();
            clap_complete::generate(shell, &mut command, bin_name, &mut std::io::stdout());
        }
        None => {
            Cli::command().print_help().expect("write help");
            println!();
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "koban",
    version,
    about = "Invoice Ninja from the terminal",
    long_about = "koban is a Rust CLI for Invoice Ninja, designed for both humans and AI agents.",
    arg_required_else_help = true
)]
struct Cli {
    /// Output format for commands that return data
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Human-readable tables
    Table,
    /// Machine-readable JSON
    Json,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate shell completions
    #[command(after_long_help = "\
Setup examples:

  zsh:
    source <(koban completions zsh)

  bash:
    source <(koban completions bash)

  fish:
    koban completions fish | source")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
