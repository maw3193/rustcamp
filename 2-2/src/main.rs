use std::fmt;
use std::io::BufRead;

#[derive(Debug)]
enum RawInstruction {
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

impl fmt::Display for RawInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IncrementDataPointer => "Increment current location",
                Self::DecrementDataPointer => "Decrement current location",
                Self::IncrementByte => "Increment the byte at the current location",
                Self::DecrementByte => "Decrement the byte at the current location",
                Self::PutByte => "Output the byte at the current location",
                Self::GetByte => "Store a byte of input at the current location",
                Self::OpenLoop => "Start looping",
                Self::CloseLoop => "Stop looping",
            }
        )
    }
}

#[derive(Debug)]
struct PositionedInstruction {
    file: String,
    instruction: RawInstruction,
    line: usize,
    character: usize,
}

impl fmt::Display for PositionedInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}:{}] {}",
            self.file, self.line, self.character, self.instruction
        )
    }
}

fn read_brainfuck_instructions(
    filename: &String,
) -> Result<Vec<PositionedInstruction>, Box<dyn std::error::Error>> {
    let mut instructions: Vec<PositionedInstruction> = Vec::new();
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);
    for (line_index, line) in file.lines().enumerate() {
        for (char_index, byte) in line?.bytes().enumerate() {
            let raw = RawInstruction::from_byte(byte);
            if raw.is_some() {
                instructions.push(PositionedInstruction {
                    file: filename.clone(),
                    instruction: raw.unwrap(),
                    line: line_index + 1,
                    character: char_index + 1,
                })
            }
        }
    }
    Ok(instructions)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;

    let prog = read_brainfuck_instructions(&filename)?;
    for instruction in prog {
        println!("{instruction}");
    }
    Ok(())
}
