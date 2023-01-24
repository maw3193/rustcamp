fn main() {
    let sum = 0;
    // Read the filename from args
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;

    // read the file
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    let contents = std::string::String::new();
    for line in reader.lines() {
        // parse line as a single number
        let num = line.parse().ok_or("{num} can't be parsed as a number")?;
        // sum up number
        sum += num;
    }

    // return the sum
    println!("Total of all lines is {sum}");
}
