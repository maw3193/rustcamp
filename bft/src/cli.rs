use std::{num::NonZeroUsize, path::PathBuf};
use clap::Parser;

use bft_interp::Machine;
use bft_types::Program;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    pub(crate) program: PathBuf,
    #[arg(short, long)]
    pub(crate) cells: Option<NonZeroUsize>,
    #[arg(short, long)]
    pub(crate) extensible: bool,
}

pub(crate) fn run_bft() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let prog = Program::from_file(&args.program)?;
    let machine: Machine<u8> = Machine::new(args.cells, args.extensible);
    machine.validate(&prog)?;
    Ok(())
}