use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::io::BufRead;
use std::vec::Vec;
use std::default::Default;

#[derive(Debug)]
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

#[derive(Default)]
#[derive(Debug)]
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
    pub fn get_attempts(&self) -> u32 {
        self.num_attempts
    }
    pub fn get_score(&self) -> u32 {
        self.total_score
    }
    pub fn get_missed(&self) -> u32 {
        self.missed_tests
    }
}

impl fmt::Display for ScoreStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ???: Is two spaces after a full stop a requirement?
        write!(f, "{} tests, with a total score of {}. They missed {} tests",
            self.num_attempts, self.total_score, self.missed_tests,
        )
    }
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

fn calculate_scores(entries: Vec<InputEntry>) -> HashMap<String, ScoreStruct> {
    let mut scores: HashMap<String, ScoreStruct> = HashMap::new();
    for entry in entries.into_iter() {
        match entry {
            InputEntry::NameOnly(name) => {
                scores.entry(name).or_insert(Default::default()).miss_test()
            }
            InputEntry::NameAndNumber(name, number) => {
                scores.entry(name).or_insert(Default::default()).add_score(number)
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
