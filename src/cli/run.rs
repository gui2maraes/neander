use crate::cpu::{Neander, NeanderException};
use crate::memfile::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run_file(file: &Path) -> ExitCode {
    let mut cpu = Neander::new();
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = parse_memfile(cpu.memory_mut(), &source) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = cpu.run() {
        eprintln!("exception: {e}");
    }
    cpu.print_mem();
    println!("{cpu}");
    ExitCode::SUCCESS
}
