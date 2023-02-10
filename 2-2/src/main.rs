use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

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

#[derive(Debug)]
struct PositionedInstruction {
    instruction: RawInstruction,
    line: usize,
    character: usize,
}

fn read_brainfuck_instructions<R>(file: BufReader<R>) -> Result<Vec<PositionedInstruction>, Box<dyn std::error::Error>> where R: std::io::Read {
    let mut instructions: Vec<PositionedInstruction> = Vec::new();
    for (line_index, line) in file.lines().enumerate() {
        for (char_index, byte) in line?.into_bytes().into_iter().enumerate() {
            let raw = RawInstruction::from_byte(byte);
            if raw.is_some() {
                instructions.push(PositionedInstruction{
                    instruction: raw.unwrap(),
                    line: line_index + 1,
                    character: char_index + 1
                })
            }
        }
    }
    Ok(instructions)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("Expected filename")?;
    let file = std::io::BufReader::new(std::fs::File::open(filename)?);

    let prog = read_brainfuck_instructions(file)?;
    for instruction in prog {
        println!("{:?}", instruction);
    }
    Ok(())
}
