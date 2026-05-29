//! Shell selection for `koban completions`.

use std::fmt;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
    Zsh,
}

impl fmt::Display for CompletionShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Bash => "bash",
            Self::Elvish => "elvish",
            Self::Fish => "fish",
            Self::Nushell => "nushell",
            Self::PowerShell => "powershell",
            Self::Zsh => "zsh",
        };
        f.write_str(value)
    }
}
