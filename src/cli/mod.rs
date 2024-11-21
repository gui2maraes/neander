use std::process::ExitCode;

use clap::Parser;

mod args;
mod repl;
mod run;
use args::*;

use crate::cpu::Neander;

pub fn cli() -> std::process::ExitCode {
    let args = args::CliArgs::parse();
    match args.command {
        Commands::Run { file } => run::run_file(&file),
        Commands::Load { file } => repl::run_repl(&file),
        Commands::ISA => {
            crate::cpu::instr::print_instr_table();
            ExitCode::SUCCESS
        }
    }
}
