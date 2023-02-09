use std::convert::TryFrom;
use std::vec::Vec;

enum InputEntry {
    NameOnly(String),
    NameAndNumber(String, u32)
}

impl TryFrom<&str> for InputEntry {
    fn try_from(value: &str) -> Result<Self, String> {
        // parse string. split by colon, left is string, right is u32.
        let parts: Vec<&str> = value.split(":").collect();
        if parts.len() == 1 {
            Ok(InputEntry::NameOnly(parts[0].to_string()))
        } else if parts.len() == 2 {
            match parts[1].parse() {
                Ok(number) => Ok(InputEntry::NameAndNumber(parts[0].to_string(), number)),
                Err(_) => Err(format!("Could not parse {} into u32", parts[1])),
            }
        } else {
            Err(format!("{} was not split by colons into 1 or 2 parts", value))
        }
    }

    type Error = String;
}

fn main() {
    println!("Hello, world!");
}
