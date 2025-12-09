mod config;
mod config_cmd;
mod core;
mod error;
mod index_cmd;
mod init;
mod list_cmd;
mod mcp;
mod mcp_cmd;
mod show;
mod show_cmd;
mod validate;
mod validate_cmd;

// cli module is now under src/cli/
mod cli;

use clap::Parser;
use cli::{ChangeCommands, Cli, Commands, ConfigCommands, McpCommands, SpecCommands};
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            println!();
        }
        Some(cmd) => {
            if let Err(e) = run(cmd) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run(cmd: Commands) -> error::Result<()> {
    match cmd {
        Commands::Init { path } => {
            let target_path = path.unwrap_or_else(|| PathBuf::from("."));
            init::run(&target_path)
        }
        Commands::Show => show_cmd::run_dashboard(),
        Commands::Spec(action) => match action {
            SpecCommands::List => list_cmd::run_spec_list(),
            SpecCommands::Show { id } => show_cmd::run_spec_show(id),
            SpecCommands::Validate { id, strict } => validate_cmd::run_spec_validate(id, strict),
        },
        Commands::Change(action) => match action {
            ChangeCommands::Init { .. } => not_implemented("change init"),
            ChangeCommands::List => list_cmd::run_change_list(),
            ChangeCommands::Show { id, deltas_only } => show_cmd::run_change_show(id, deltas_only),
            ChangeCommands::Validate { id, strict } => {
                validate_cmd::run_change_validate(id, strict)
            }
            ChangeCommands::Approve => not_implemented("change approve"),
        },
        Commands::Config(action) => match action {
            ConfigCommands::Show { paths } => config_cmd::run_show(paths),
        },
        Commands::Index => index_cmd::run(),
        Commands::Mcp(action) => match action {
            McpCommands::Serve => mcp_cmd::serve(),
        },
    }
}

fn not_implemented(name: &str) -> error::Result<()> {
    Err(error::Error::NotImplemented(name.to_string()))
}
