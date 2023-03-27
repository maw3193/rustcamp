mod cli;

fn main() {
    match cli::run_bft() {
        Err(e) => {
            println!("{}: Error: {}", std::env::args().nth(0).unwrap(), e.to_string())
        },
        Ok(_) => (),
    }
}
