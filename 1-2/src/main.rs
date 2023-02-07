use std::io::Read;
const BRAINFUCK_CHARS: &[u8; 8] = b",.<>[]-+";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);
    // BufReader.bytes() returns an iterator of Results. First, handle Error, then filter.
    let prog = file.bytes()
        .collect::<Result<std::vec::Vec<_>,_>>()?
        .into_iter()
        .filter(|x| BRAINFUCK_CHARS.contains(x))
        .collect();
    println!("{}", std::string::String::from_utf8(prog)?);
    Ok(())
}
