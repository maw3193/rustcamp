mod cli;
use std::process;

fn main() {
    if let Err(e) = cli::run_bft() {
        println!("{}: Error: {}", std::env::args().next().unwrap(), e,);
        process::exit(1);
    }
}
