use std::io::BufRead;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sum: i32 = 0;
    // Read the filename from args
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;

    // read the file
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        // parse line as a single number
        //println!("{}", line?);
        let num: i32 = line?.parse().unwrap();
        // sum up number
        sum += num;
    }

    // return the sum
    println!("Total of all lines is {sum}");
    Ok(())
}
