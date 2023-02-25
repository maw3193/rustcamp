use bft_interp::Machine;
use bft_types::Program;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let prog = Program::from_file(filename.as_ref())?;
    let machine: Machine<u8> = Machine::new(0, false);
    machine.print_program(&prog);
    Ok(())
}
