use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Loads and immediately runs the memory file,
    /// displaying the CPU state afterwards.
    Run {
        /// Memory file to load
        file: PathBuf,
    },
    /// Loads the file and starts a interactive session.
    Load {
        /// Memory file to load
        file: PathBuf,
    },
    /// Prints a table containing all instructions and its codes.
    ISA,
}
