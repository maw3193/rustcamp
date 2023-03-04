//! Brainfuck types library
//! A description of the brainfuck language model, translated from text into rust data structures.

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::string::String;

/// An enum of every possible instruction Brainfuck can execute
#[derive(Debug, PartialEq)]
pub enum RawInstruction {
    IncrementDataPointer,
    DecrementDataPointer,
    IncrementByte,
    DecrementByte,
    PutByte,
    GetByte,
    OpenLoop,
    CloseLoop,
}

impl RawInstruction {
    /// Constructs a RawInstruction from a byte
    /// Returns an Option as we expect brainfuck code to contain bytes that aren't instructions.
    /// # Examples
    /// ```
    /// # use bft_types::RawInstruction;
    /// let instruction = RawInstruction::from_byte(b'>');
    /// assert_eq!(instruction, Some(RawInstruction::IncrementDataPointer));
    ///
    /// let not_instruction = RawInstruction::from_byte(b'w');
    /// assert_eq!(not_instruction, None);
    /// ```
    pub fn from_byte(byte: u8) -> Option<RawInstruction> {
        match byte {
            b'>' => Some(RawInstruction::IncrementDataPointer),
            b'<' => Some(RawInstruction::DecrementDataPointer),
            b'+' => Some(RawInstruction::IncrementByte),
            b'-' => Some(RawInstruction::DecrementByte),
            b'.' => Some(RawInstruction::PutByte),
            b',' => Some(RawInstruction::GetByte),
            b'[' => Some(RawInstruction::OpenLoop),
            b']' => Some(RawInstruction::CloseLoop),
            _ => None,
        }
    }
}

impl fmt::Display for RawInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::IncrementDataPointer => "Increment current location",
            Self::DecrementDataPointer => "Decrement current location",
            Self::IncrementByte => "Increment the byte at the current location",
            Self::DecrementByte => "Decrement the byte at the current location",
            Self::PutByte => "Output the byte at the current location",
            Self::GetByte => "Store a byte of input at the current location",
            Self::OpenLoop => "Start looping",
            Self::CloseLoop => "Stop looping",
        })
    }
}

/// A brainfuck instruction with added context of where it exists within the codebase
#[derive(Debug)]
pub struct PositionedInstruction {
    instruction: RawInstruction,
    line: usize,
    character: usize,
}

impl PositionedInstruction {
    pub fn instruction(&self) -> &RawInstruction {
        &self.instruction
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn character(&self) -> usize {
        self.character
    }
}

impl fmt::Display for PositionedInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.character, self.instruction)
    }
}
/// A collection of all the brainfuck instructions within a single source file
#[derive(Debug)]
pub struct Program {
    file: String,
    instructions: Vec<PositionedInstruction>,
}

impl Program {
    /// Reads all the text in a file and converts it into a brainfuck program.
    /// This process is fallible, so returns a Result.
    /// # Examples
    /// ```no_run
    /// # use bft_types;
    /// let filepath = std::path::Path::new("my_file.bf");
    /// let prog: std::io::Result<bft_types::Program> = bft_types::Program::from_file(&filepath);
    /// ```
    // TODO: Path to AsRef<Path>
    // new<P: AsRef<Path>>(path: P)
    pub fn from_file<T: AsRef<Path> + ToString + Copy>(file: T) -> std::io::Result<Program> {
        // Load the text from the path, pass it into new.
        let mut text = String::new();
        BufReader::new(File::open(file)?).read_to_string(&mut text)?;
        Ok(Self::new(file, &text))
    }

    /// Converts a string into a brainfuck program.
    /// # Examples
    /// ```
    /// # use bft_types;
    /// let filename = std::path::Path::new("(no file)");
    /// let text = "[,.]".to_string();
    /// let prog: bft_types::Program = bft_types::Program::new(filename, text);
    /// ```
    // NOTE: I tried to use the generic `U: AsRef<str> + BufRead>` but then text.lines() was fallible.
    pub fn new<T: AsRef<Path> + ToString>(filename: T, text: &str) -> Program {
        let mut instructions: Vec<PositionedInstruction> = Vec::new();
        for (line_index, line) in text.lines().enumerate() {
            for (char_index, byte) in line.bytes().enumerate() {
                match RawInstruction::from_byte(byte) {
                    None => (),
                    Some(instruction) => instructions.push(PositionedInstruction {
                        instruction,
                        line: line_index + 1,
                        character: char_index + 1,
                    }),
                }
            }
        }
        // non-utf8 characters may be valid paths to readable files so I oughtn't to reject parsing them.
        // If I can read them I can print the closest I can to its representation.
        Program {
            file: filename.to_string(),
            instructions,
        }
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn instructions(&self) -> &[PositionedInstruction] {
        &self.instructions
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.instructions() {
            writeln!(f, "{}:{}", self.file(), instruction,)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructions_from_byte() {
        let test_data = [
            (b'<', Some(RawInstruction::DecrementDataPointer)),
            (b'>', Some(RawInstruction::IncrementDataPointer)),
            (b'+', Some(RawInstruction::IncrementByte)),
            (b'-', Some(RawInstruction::DecrementByte)),
            (b',', Some(RawInstruction::GetByte)),
            (b'.', Some(RawInstruction::PutByte)),
            (b'[', Some(RawInstruction::OpenLoop)),
            (b']', Some(RawInstruction::CloseLoop)),
            (b'*', None),
        ];
        for (input, output) in test_data {
            assert_eq!(output, RawInstruction::from_byte(input));
        }
    }

    #[test]
    fn correct_position() {
        #[rustfmt::skip]
        let text = [
            "[asdf",
            " . +-",
            "]"
        ].join("\n");
        let results = [(1, 1), (2, 2), (2, 4), (2, 5), (3, 1)];
        let prog = Program::new("irrelevant_path", &text);
        print!("{prog}");
        for (index, instruction) in prog.instructions().iter().enumerate() {
            assert_eq!(instruction.line(), results[index].0);
            assert_eq!(instruction.character(), results[index].1);
        }
    }
}
