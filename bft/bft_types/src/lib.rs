//! Brainfuck types library
//! A description of the brainfuck language model, translated from text into rust data structures.

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::string::String;
use thiserror::Error;

/// An enum of every possible instruction Brainfuck can execute
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, Clone)]
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
/// Instructions that have been processed into a form useful to an interpreter
pub enum DecoratedInstruction {
    /// A loop has been opened. In addition, here is where it will close
    OpenLoop {
        instruction: PositionedInstruction,
        closer: PositionedInstruction,
        closer_index: usize,
    },
    /// A loop has been closed. In addition, here is where it was opened
    CloseLoop {
        instruction: PositionedInstruction,
        opener: PositionedInstruction,
        opener_index: usize,
    },
    /// An ordinary instruction that can be used as-is
    Instruction(PositionedInstruction),
    /// An open-bracket instruction that gets replaced when the loop is closed
    ///
    /// This is expected to only exist as a temporary implementation detail.
    PlaceholderOpenBracket,
}

/// A program that's been processed into a form useful to an interpreter
/// Compared to a Program, this has the additional constraint that the code must be valid Brainfuck.
pub struct DecoratedProgram {
    file: PathBuf,
    decorated_instructions: Vec<DecoratedInstruction>,
}

/// Errors that may occur while parsing a Brainfuck program.
#[derive(Debug, Error)]
pub enum ParseError {
    /// A bracket was opened, but never closed
    UnopenedBracket {
        closer: PositionedInstruction,
        source_file: PathBuf,
    },
    /// A closing bracket was found before an opening bracket
    UnclosedBracket {
        opener: PositionedInstruction,
        source_file: PathBuf,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnopenedBracket {
                closer,
                source_file,
            } => {
                write!(
                    f,
                    "In input file {}, closed a loop with no matching opener at line {}, column {}",
                    source_file.to_string_lossy(),
                    closer.line(),
                    closer.character()
                )
            }
            Self::UnclosedBracket {
                opener,
                source_file,
            } => {
                write!(
                    f,
                    "In input file {}, opened a loop that wasn't closed at line {}, column {}",
                    source_file.to_string_lossy(),
                    opener.line(),
                    opener.character()
                )
            }
        }
    }
}

impl DecoratedProgram {
    /// Parses a Program, returning either a validated Brainfuck program, or the reason why it's invalid
    /// # Examples
    /// ```
    /// # use bft_types;
    /// let code = "[+++.]";
    /// let raw_prog = bft_types::Program::new("<Test program>", code);
    /// assert!(bft_types::DecoratedProgram::from_program(&raw_prog).is_ok());
    /// ```
    /// ```
    /// # use bft_types;
    /// let code = "[+++.";
    /// let raw_prog = bft_types::Program::new("<Test program>", code);
    /// assert!(bft_types::DecoratedProgram::from_program(&raw_prog).is_err());
    /// ```
    /// ```
    /// # use bft_types;
    /// let code = "[+++.]]";
    /// let raw_prog = bft_types::Program::new("<Test program>", code);
    /// assert!(bft_types::DecoratedProgram::from_program(&raw_prog).is_err());
    /// ```
    pub fn from_program(prog: &Program) -> Result<DecoratedProgram, ParseError> {
        let mut bracket_stack = Vec::new();
        let mut decorated_instructions: Vec<DecoratedInstruction> = Vec::new();
        for (index, instruction) in prog.instructions().iter().enumerate() {
            match instruction.instruction() {
                RawInstruction::OpenLoop => {
                    bracket_stack.push((index, instruction));

                    decorated_instructions.push(DecoratedInstruction::PlaceholderOpenBracket);
                }
                RawInstruction::CloseLoop => {
                    let opener = bracket_stack.pop();
                    if opener.is_none() {
                        return Err(ParseError::UnopenedBracket {
                            closer: instruction.clone(),
                            source_file: prog.file().clone(),
                        });
                    };
                    // Now that we've closed the loop, go back and decorate the opener.
                    decorated_instructions[opener.unwrap().0] = DecoratedInstruction::OpenLoop {
                        instruction: opener.unwrap().1.clone(),
                        closer: instruction.clone(),
                        closer_index: index,
                    };

                    decorated_instructions.push(DecoratedInstruction::CloseLoop {
                        instruction: instruction.clone(),
                        opener: opener.unwrap().1.clone(),
                        opener_index: opener.unwrap().0,
                    });
                }
                _ => decorated_instructions
                    .push(DecoratedInstruction::Instruction(instruction.clone())),
            };
        }
        if !bracket_stack.is_empty() {
            return Err(ParseError::UnclosedBracket {
                opener: bracket_stack.pop().unwrap().1.clone(),
                source_file: prog.file().clone(),
            });
        };

        // Double-check I haven't left placeholders lying around
        // NOTE: Used a match for lack of a better way of comparing an enum
        assert!(decorated_instructions
            .iter()
            .all(|i| !matches!(i, DecoratedInstruction::PlaceholderOpenBracket)));

        Ok(DecoratedProgram {
            file: prog.file().clone(),
            decorated_instructions,
        })
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    pub fn decorated_instructions(&self) -> &[DecoratedInstruction] {
        self.decorated_instructions.as_ref()
    }
}

/// A collection of all the brainfuck instructions within a single source file
#[derive(Debug)]
pub struct Program {
    file: PathBuf,
    instructions: Vec<PositionedInstruction>,
}

impl Program {
    /// Reads all the text in a file and converts it into a brainfuck program.
    /// This process is fallible, so returns a Result.
    /// # Examples
    /// ```no_run
    /// # use bft_types;
    /// let filepath = "my_file.bf";
    /// let prog: std::io::Result<bft_types::Program> = bft_types::Program::from_file(&filepath);
    /// ```
    // TODO: Path to AsRef<Path>
    // new<P: AsRef<Path>>(path: P)
    pub fn from_file<T: AsRef<Path> + Copy + Into<PathBuf>>(file: T) -> std::io::Result<Program> {
        // Load the text from the path, pass it into new.
        let mut text = String::new();
        BufReader::new(File::open(file)?).read_to_string(&mut text)?;
        Ok(Self::new(file, &text))
    }

    /// Converts a string into a brainfuck program.
    /// # Examples
    /// ```
    /// # use bft_types;
    /// let filename = "(no file)";
    /// let text = "[,.]";
    /// let prog: bft_types::Program = bft_types::Program::new(&filename, &text);
    /// ```
    // NOTE: I tried to use the generic `U: AsRef<str> + BufRead>` but then text.lines() was fallible.
    pub fn new<T: AsRef<Path> + Into<PathBuf>>(filename: T, text: &str) -> Program {
        let mut instructions: Vec<PositionedInstruction> = Vec::new();
        for (line_index, line) in text.lines().enumerate() {
            for (char_index, byte) in line.bytes().enumerate() {
                if let Some(instruction) = RawInstruction::from_byte(byte) {
                    instructions.push(PositionedInstruction {
                        instruction,
                        line: line_index + 1,
                        character: char_index + 1,
                    });
                }
            }
        }
        // non-utf8 characters may be valid paths to readable files so I oughtn't to reject parsing them.
        // If I can read them I can print the closest I can to its representation.
        Program {
            file: filename.into(),
            instructions,
        }
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    pub fn instructions(&self) -> &[PositionedInstruction] {
        &self.instructions
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.instructions() {
            writeln!(f, "{}:{}", self.file().to_string_lossy(), instruction,)?
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
