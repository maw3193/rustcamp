use bft_interp::Machine;
use bft_types::Program;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let prog = Program::from_file(&filename)?;
    let machine: Machine<u8> = Machine::new(None, false);
    machine.print_program(&prog);
    Ok(())
}
