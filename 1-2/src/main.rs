// Project specification: Load a source file
// convert it to a sequence of bytes
// strip out any characters that aren't valid brainfuck
use std::io::Read;
//const brainfuck_chars: std::vec::Vec<char> = std::vec!['>', '<', '+', '-', '.', ',', '[', ']'];
const BRAINFUCK_CHARS: &str = "><+-.,[]";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);
    let mut prog: std::vec::Vec<char> = std::vec::Vec::new();
    for byte in file.bytes() {
    //    println!("{}", byte.unwrap());
        let byte = byte.unwrap();
        if BRAINFUCK_CHARS.contains(byte as char) {
            prog.push(byte as char); // byte is a Result?
        }
    }

    println!("{}", prog.into_iter().collect::<std::string::String>());
    Ok(())
}
