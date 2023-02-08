use std::io::BufRead;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;

    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);

    let sum = reader.lines().map({
        |line| match line {
            Ok(text) => {
                text.parse::<i32>()?
            }
            Err(e) => e
        }
    }).fold(0, |sum, i| sum + i);

    println!("Total of all lines is {sum}");
    Ok(())
}
