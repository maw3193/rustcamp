use clap::Parser;
use std::{
    io::{stdin, stdout},
    num::NonZeroUsize,
    path::PathBuf,
};

use bft_interp::Machine;
use bft_types::{DecoratedProgram, Program};

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
    let decorated = DecoratedProgram::from_program(&prog)?;
    let mut machine: Machine<u8> = Machine::new(args.cells, args.extensible, &decorated);
    machine.interpret(&mut stdin(), &mut stdout())?;
    Ok(())
}
