use std::convert::TryFrom;
use std::io::BufRead;
use std::vec::Vec;

enum InputEntry {
    NameOnly(String),
    NameAndNumber(String, u32)
}

impl TryFrom<&str> for InputEntry {
    fn try_from(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // parse string. split by colon, left is string, right is u32.
        let parts: Vec<&str> = value.split(":").collect();
        if parts.len() == 1 {
            Ok(InputEntry::NameOnly(parts[0].to_string()))
        } else if parts.len() == 2 {
            Ok(InputEntry::NameAndNumber(parts[0].to_string(), parts[1].parse()?))
        } else {
            Err(Box::from(format!("{} was not split by colons into 1 or 2 parts", value)))
        }
    }

    type Error = Box<dyn std::error::Error>;
}

fn load_input_entries(filename: &String) -> Result<Vec<InputEntry>, Box<dyn std::error::Error>> {
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);
    // file.lines is a Result, because it may fail.
    // InputEntry::try_from(str) is a Result because it may fail.
    file.lines()
        .collect::<Result<Vec<_>,_>>()?
        .into_iter()
        .map(|x| InputEntry::try_from(x.as_str()))
        .collect::<Result<Vec<_>, _>>()
}

fn main() {
    println!("Hello, world!");
}
