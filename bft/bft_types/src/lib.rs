use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{self, Path};
use std::string::String;

#[derive(Debug)]
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
    fn from_byte(byte: u8) -> Option<RawInstruction> {
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

#[derive(Debug)]
pub struct Program {
    file: String,
    instructions: Vec<PositionedInstruction>,
}

impl Program {
    pub fn from_file(file: &Path) -> std::io::Result<Program> {
        // Load the text from the path, pass it into new.
        let mut text = String::new();
        BufReader::new(File::open(file)?).read_to_string(&mut text)?;
        Ok(Self::new(file, text))
    }
    pub fn new(filename: &Path, text: String) -> Program {
        let mut instructions: Vec<PositionedInstruction> = Vec::new();
        // Split the string into lines, split the line into characters, as iteration.
        for (line_index, line) in text.lines().enumerate() {
            for (char_index, byte) in line.bytes().enumerate() {
                match RawInstruction::from_byte(byte) {
                    None => (),
                    Some(instruction) => instructions.push(PositionedInstruction {
                        instruction: instruction,
                        line: line_index + 1,
                        character: char_index + 1,
                    }),
                }
            }
        }
        // non-utf8 characters may be valid paths to readable files so I oughtn't to reject parsing them.
        // If I can read them I can print the closest I can to its representation.
        Program {
            file: String::from(filename.to_string_lossy()),
            instructions: instructions,
        }
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn instructions(&self) -> &[PositionedInstruction] {
        &&self.instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
