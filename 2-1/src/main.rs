use std::collections::HashMap;
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::io::BufRead;
use std::vec::Vec;

#[derive(Debug)]
enum InputEntry {
    NameOnly(String),
    NameAndNumber(String, u32),
}

impl TryFrom<&str> for InputEntry {
    // https://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/
    // i.e. nothing forbids names from containing colons, or even being an
    // empty string.
    // By explicit instruction, colons are reserved for delimiters, thus names
    // cannot contain colons.
    // I am also deciding on my own initiative that a name must have
    // *something*, no empty strings.
    // 'Name: 12' unambiguous, simple to implement -> Allow it!
    // 'name:12' nothing says a name has to start with an upper-case letter
    // 'Name :12' a valid name, even if the space is probably a typo.
    // ':231' is arguably a valid name, but I'm deciding a name must have *something*.
    // 'fdsfds:' Colons are reserved for delimiters, but there's no number on
    // the right-hand side.
    // 'ufdd::12' too many delimiters, malformed expression.
    // 'Robert'; -- \nDROP TABLE students       ;:12' Bobby Tables is innocent,
    // but make sure \n is expressed literally, not as a new line.
    fn try_from(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // parse string. split by colon, left is string, right is u32.
        let parts: Vec<&str> = value.split(":").collect();
        if parts[0].is_empty() {
            Err(Box::from(format!(
                "'{}' has an empty string in its name section",
                value
            )))
        } else if parts.len() == 2 && parts[1].is_empty() {
            Err(Box::from(format!(
                "'{}' has an empty string in its number section",
                value
            )))
        } else if parts.len() == 1 {
            Ok(InputEntry::NameOnly(parts[0].to_string()))
        } else if parts.len() == 2 {
            Ok(InputEntry::NameAndNumber(
                parts[0].to_string(),
                parts[1].trim().parse()?,
            ))
        } else {
            Err(Box::from(format!(
                "{} was not split by colons into 1 or 2 parts",
                value
            )))
        }
    }

    type Error = Box<dyn std::error::Error>;
}

#[derive(Default, Debug)]
struct ScoreStruct {
    num_attempts: u32,
    total_score: u32,
    missed_tests: u32,
}

impl ScoreStruct {
    pub fn add_score(&mut self, score: u32) {
        self.num_attempts += 1;
        self.total_score += score;
    }
    pub fn miss_test(&mut self) {
        self.missed_tests += 1;
    }
    // trivial accessor functions were removed in commit dfb4432 because they were never used.
}

impl fmt::Display for ScoreStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} tests, with a total score of {}. They missed {} tests",
            self.num_attempts, self.total_score, self.missed_tests,
        )
    }
}

fn load_input_entries(filename: &String) -> Result<Vec<InputEntry>, Box<dyn std::error::Error>> {
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);
    // file.lines is a Result, because it may fail.
    // InputEntry::try_from(str) is a Result because it may fail.
    file.lines()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|x| InputEntry::try_from(x.as_str()))
        .collect::<Result<Vec<_>, _>>()
}

fn calculate_scores(entries: Vec<InputEntry>) -> HashMap<String, ScoreStruct> {
    let mut scores: HashMap<String, ScoreStruct> = HashMap::new();
    for entry in entries.into_iter() {
        match entry {
            InputEntry::NameOnly(name) => scores.entry(name).or_default().miss_test(),
            InputEntry::NameAndNumber(name, number) => {
                scores.entry(name).or_default().add_score(number)
            }
        }
    }
    scores
}

fn print_scores(scores: &HashMap<String, ScoreStruct>) {
    for (name, score) in scores.iter() {
        println!("{name} took {score}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let entries = load_input_entries(&filename)?;
    let scores = calculate_scores(entries);
    print_scores(&scores);
    Ok(())
}
