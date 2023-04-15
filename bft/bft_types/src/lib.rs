//! Brainfuck types library
//! A description of the brainfuck language model, translated from text into rust data structures.

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::string::String;
use thiserror::Error;

/// An enum of every possible instruction Brainfuck can execute
#[derive(Debug, PartialEq, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
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
#[derive(Clone, Copy)]
pub enum DecoratedInstruction {
    /// A loop has been opened. In addition, here is where it will close
    OpenLoop {
        instruction: PositionedInstruction,
        closer: PositionedInstruction,
    },
    /// A loop has been closed. In addition, here is where it was opened
    CloseLoop {
        instruction: PositionedInstruction,
        opener: PositionedInstruction,
    },
    /// An ordinary instruction that can be used as-is
    Instruction(PositionedInstruction),
    /// An open-bracket instruction that gets replaced when the loop is closed
    ///
    /// This is expected to only exist as a temporary implementation detail.
    PlaceholderOpenBracket,
}

impl DecoratedInstruction {
    pub fn instruction(&self) -> PositionedInstruction {
        // :-/ That placeholder variant sure is a nuisance!
        assert!(!matches!(self, Self::PlaceholderOpenBracket));

        match self {
            Self::OpenLoop { instruction, .. } => *instruction,
            Self::CloseLoop { instruction, .. } => *instruction,
            Self::Instruction(instruction) => *instruction,
            Self::PlaceholderOpenBracket => unreachable!(),
        }
    }

    pub fn line(&self) -> usize {
        self.instruction().line()
    }

    pub fn character(&self) -> usize {
        self.instruction().character()
    }
}

impl fmt::Display for DecoratedInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        assert!(!matches!(self, Self::PlaceholderOpenBracket));
        write!(f, "{}", self.instruction())
    }
}

/// A program that's been processed into a form useful to an interpreter
/// Compared to a Program, this has the additional constraint that the code must be valid Brainfuck.
pub struct DecoratedProgram {
    file: PathBuf,
    decorated_instructions: Vec<DecoratedInstruction>,
}
impl DecoratedProgram {
    pub fn position_to_index(&self, line: usize, character: usize) -> usize {
        self.decorated_instructions
            .binary_search_by(|instruction| {
                instruction
                    .line()
                    .cmp(&line)
                    .then(instruction.character().cmp(&character))
            })
            .unwrap()
        // >:| I can't just search for the DecoratedInstruction because I don't store it
        //
    }
}
impl fmt::Display for DecoratedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.decorated_instructions() {
            writeln!(f, "{}:{}", self.file().display(), instruction,)?
        }
        Ok(())
    }
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
                            closer: *instruction,
                            source_file: prog.file().to_path_buf(),
                        });
                    };
                    // Now that we've closed the loop, go back and decorate the opener.
                    decorated_instructions[opener.unwrap().0] = DecoratedInstruction::OpenLoop {
                        instruction: *(opener.unwrap().1),
                        closer: *instruction,
                    };

                    decorated_instructions.push(DecoratedInstruction::CloseLoop {
                        instruction: *instruction,
                        opener: *(opener.unwrap().1),
                    });
                }
                _ => decorated_instructions.push(DecoratedInstruction::Instruction(*instruction)),
            };
        }
        if !bracket_stack.is_empty() {
            return Err(ParseError::UnclosedBracket {
                opener: *(bracket_stack.pop().unwrap().1),
                source_file: prog.file().to_path_buf(),
            });
        };

        // Double-check I haven't left placeholders lying around
        assert!(decorated_instructions
            .iter()
            .all(|i| !matches!(i, DecoratedInstruction::PlaceholderOpenBracket)));

        Ok(DecoratedProgram {
            file: prog.file().to_path_buf(),
            decorated_instructions,
        })
    }

    pub fn file(&self) -> &Path {
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
    pub fn from_file<T: AsRef<Path>>(file: T) -> std::io::Result<Program> {
        let file: PathBuf = file.as_ref().to_path_buf();
        // Load the text from the path, pass it into new.
        let mut text = String::new();
        BufReader::new(File::open(&file)?).read_to_string(&mut text)?;
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
    pub fn new<T: AsRef<Path>>(filename: T, text: &str) -> Program {
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
        Program {
            file: filename.as_ref().to_path_buf(),
            instructions,
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn instructions(&self) -> &[PositionedInstruction] {
        &self.instructions
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.instructions() {
            writeln!(f, "{}:{}", self.file().display(), instruction,)?
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
