mod cli;
mod config;
mod config_cmd;
mod error;
mod init;

use clap::Parser;
use cli::{ChangeCommands, Cli, Commands, ConfigCommands, SpecCommands};
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
        Commands::Spec(action) => match action {
            SpecCommands::List => not_implemented("spec list"),
            SpecCommands::Show { .. } => not_implemented("spec show"),
            SpecCommands::Validate => not_implemented("spec validate"),
        },
        Commands::Change(action) => match action {
            ChangeCommands::Init { .. } => not_implemented("change init"),
            ChangeCommands::List => not_implemented("change list"),
            ChangeCommands::Show { .. } => not_implemented("change show"),
            ChangeCommands::Validate => not_implemented("change validate"),
            ChangeCommands::Approve => not_implemented("change approve"),
        },
        Commands::Config(action) => match action {
            ConfigCommands::Show => config_cmd::run_show(),
        },
    }
}

fn not_implemented(name: &str) -> error::Result<()> {
    Err(error::Error::NotImplemented(name.to_string()))
}
