use bft_interp::Machine;
use bft_types::Program;
use clap::Parser;
mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Cli::parse();
    let prog = Program::from_file(&args.program)?;
    let machine: Machine<u8> = Machine::new(args.cells, args.extensible);
    machine.print_program(&prog);
    Ok(())
}
