use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Spec-driven development for humans and AI
#[derive(Parser, Debug)]
#[command(name = "spox")]
#[command(version)]
#[command(about = "Spec-driven development for humans and AI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Spox project
    Init {
        /// Path to initialize the project in (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Manage specs
    #[command(subcommand)]
    Spec(SpecCommands),

    /// Manage changes
    #[command(subcommand)]
    Change(ChangeCommands),

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand, Debug)]
pub enum SpecCommands {
    /// List all specs
    List,

    /// Show a spec
    Show {
        /// Spec ID to show
        id: Option<String>,
    },

    /// Validate specs
    Validate {
        /// Spec ID to validate (validates all if not specified)
        id: Option<String>,

        /// Strict mode - warnings also cause failure
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ChangeCommands {
    /// Initialize a new change
    Init {
        /// Change ID (required)
        id: String,
    },

    /// List all changes
    List,

    /// Show a change
    Show {
        /// Change ID to show
        id: Option<String>,
    },

    /// Validate changes
    Validate {
        /// Change ID to validate (validates all if not specified)
        id: Option<String>,

        /// Strict mode - warnings also cause failure
        #[arg(long)]
        strict: bool,
    },

    /// Approve a change
    Approve,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show configuration
    Show,
}
