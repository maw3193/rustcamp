mod cli;

fn main() {
    if let Err(e) = cli::run_bft() {
        println!("{}: Error: {}", std::env::args().next().unwrap(), e,)
    }
}
