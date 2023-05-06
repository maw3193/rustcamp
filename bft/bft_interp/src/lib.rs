//! Brainfuck interpreter library
//! An implementation of the brainfuck virtual machine

use std::io::{Read, Write};
use std::num::NonZeroUsize;

use bft_types::{DecoratedInstruction, DecoratedProgram, PositionedInstruction, RawInstruction};

use thiserror::Error;

pub trait CellKind: std::clone::Clone + Default {
    /// Increase the value of the cell by 1
    fn increment(&mut self);
    /// Decrease the value of the cell by 1
    fn decrement(&mut self);
    /// Sets the cell's value to the given value
    ///
    /// Note that the value is a u8 because brainfuck only reads single bytes from stdin
    fn set_value(&mut self, value: u8);
    /// Gets the cell's value as a single byte
    fn get_value(&self) -> u8;
    /// Returns whether the cell's value is equal to zero
    fn is_zero(&self) -> bool;
}

impl CellKind for u8 {
    fn increment(&mut self) {
        *self = self.wrapping_add(1)
    }
    fn decrement(&mut self) {
        *self = self.wrapping_sub(1)
    }

    fn set_value(&mut self, value: u8) {
        *self = value
    }
    fn get_value(&self) -> u8 {
        *self
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }
}
/// A brainfuck virtual machine
///
/// The type T is the type that all brainfuck cells will be.
///
/// The machine is initialised with a specific number of cells, and may
/// allocate more cells when the head extends beyond the end of memory if
/// configured to do so.
pub struct Machine<'a, T> {
    /// The Machine's internal memory
    cells: Vec<T>,
    /// The memory pointer
    ///
    /// i.e. the point in memory where memory read/write/increment/decrement instructions are applied
    head: usize,
    /// The Instruction Pointer
    ///
    /// i.e. an index into the list of instructions inside the program
    instruction_pointer: usize,
    /// Whether the cells can be extended if the memory pointer extends past the end
    may_grow: bool,
    /// The program the Machine will run
    prog: &'a DecoratedProgram,
}

impl<'a, T> Machine<'a, T> {
    /// Writes the program this Machine was initialised with to standard output
    pub fn print_program(&self) {
        print!("{}", self.prog)
    }

    /// Returns a reference to the Machine's cells
    pub fn cells(&self) -> &[T] {
        self.cells.as_ref()
    }

    /// Returns a reference to the Machine's head
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns whether the Machine may grow
    pub fn may_grow(&self) -> bool {
        self.may_grow
    }

    /// Returns a reference to the program inside the Machine
    pub fn prog(&self) -> &'a DecoratedProgram {
        self.prog
    }

    /// Returns the instruction at the instruction pointer
    pub fn current_instruction(&self) -> DecoratedInstruction {
        self.prog().decorated_instructions()[self.instruction_pointer]
    }

    fn next_instruction(&self) -> usize {
        self.instruction_pointer + 1
    }

    /// Decrements the memory pointer
    ///
    /// If doing so would cause the memory pointer to become negative, it instead returns a [VMError::SeekTooLow]
    ///
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// assert!(interp.seek_left().is_err());
    /// ```
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// interp.seek_right();
    /// assert!(interp.seek_left().is_ok());
    /// assert_eq!(interp.head(), 0);
    /// ```
    /// TODO! Come back here when moving the head is more useful
    /// TODO! Once I can run programs, decide whether I want to allow external mutation of program state
    pub fn seek_left(&mut self) -> Result<usize, VMError> {
        if self.head == 0 {
            Err(VMError::SeekTooLow(
                self.current_instruction().instruction(),
            ))
        } else {
            self.head -= 1;
            Ok(self.next_instruction())
        }
    }
}

impl<'a, T> Machine<'a, T>
where
    T: CellKind,
{
    /// Creates a new virtual machine of the specified size, type, and whether it can grow.
    /// If `size` is set to 0, it will choose the default, 30000.
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// ```
    pub fn new(
        size: Option<NonZeroUsize>,
        may_grow: bool,
        prog: &'a DecoratedProgram,
    ) -> Machine<'a, T> {
        let size = match size {
            None => 30000,
            Some(sz) => sz.into(),
        };
        let cells = vec![Default::default(); size];
        Machine {
            head: 0,
            instruction_pointer: 0,
            cells,
            may_grow,
            prog,
        }
    }

    /// Increments the memory pointer
    ///
    /// If doing so would cause the memory pointer to exceed the allotted cells, it will either allocate more cells (if may_grow is set), or return a [VMError::SeekTooHigh]
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// interp.seek_right();
    /// assert_eq!(interp.head(), 1);
    /// ```
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// # use std::num::NonZeroUsize;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let cell_size = NonZeroUsize::new(1).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(Some(cell_size), false, &prog);
    /// assert!(interp.seek_right().is_err());
    /// ```
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// # use std::num::NonZeroUsize;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let cell_size = NonZeroUsize::new(1).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(Some(cell_size), true, &prog);
    /// assert!(interp.seek_right().is_ok());
    /// assert_eq!(interp.cells().len(), 2);
    /// ```
    /// TODO! Come back here when moving the head is more useful
    /// TODO! Once I can run programs, decide whether I want to allow external mutation of program state
    pub fn seek_right(&mut self) -> Result<usize, VMError> {
        if self.head + 1 == self.cells.len() {
            if !self.may_grow {
                return Err(VMError::SeekTooHigh(
                    self.current_instruction().instruction(),
                ));
            } else {
                self.cells.push(Default::default());
            }
        }
        self.head += 1;
        Ok(self.next_instruction())
    }

    pub fn current_cell(&mut self) -> &mut T {
        &mut self.cells[self.head]
    }

    /// Increase the value of the cell at the data pointer
    ///
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// assert_eq!(interp.cells()[0], 0);
    /// interp.increment_cell();
    /// assert_eq!(interp.cells()[0], 1);
    /// ```
    pub fn increment_cell(&mut self) -> Result<usize, VMError> {
        self.current_cell().increment();
        Ok(self.next_instruction())
    }

    /// Decrease the value of the cell at the data pointer
    ///
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// assert_eq!(interp.cells()[0], 0);
    /// interp.decrement_cell();
    /// assert_eq!(interp.cells()[0], 255);
    /// ```
    pub fn decrement_cell(&mut self) -> Result<usize, VMError> {
        self.current_cell().decrement();
        Ok(self.next_instruction())
    }

    /// Read a value from `file` into memory at the memory pointer
    ///
    /// If an I/O Error occurs while trying to read the file, it returns that error wrapped inside a [VMError].
    ///
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// let mut data = std::io::Cursor::new(vec![7]);
    /// interp.read_value(&mut data);
    /// assert_eq!(interp.cells()[0], 7);
    /// ```
    /// TODO: More examples?
    pub fn read_value(&mut self, file: &mut impl Read) -> Result<usize, VMError> {
        let mut buffer: [u8; 1] = [0; 1];
        match file.read_exact(&mut buffer) {
            Ok(()) => {
                self.current_cell().set_value(buffer[0]);
                Ok(self.next_instruction())
            }
            Err(ioerror) => Err(VMError::IOError {
                instruction: self.current_instruction().instruction(),
                source: ioerror,
            }),
        }
    }

    /// Writes the value at the memory pointer into `file`
    ///
    /// If an I/O Error occurs while trying to write the file, it returns that error wrapped inside a [VMError].
    ///
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// # use bft_types;
    /// // Setup
    /// let prog: bft_types::DecoratedProgram = bft_types::DecoratedProgram::from_program(
    ///     &bft_types::Program::new("<None>", "[,.]")
    /// ).unwrap();
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false, &prog);
    /// let mut data = std::io::Cursor::new(vec![7]);
    /// // Preload data into the Machine
    /// interp.read_value(&mut data);
    ///
    /// // Actually write a value to file
    /// interp.write_value(&mut data);
    /// assert_eq!(data.get_ref()[1], 7);
    /// ```
    pub fn write_value(&mut self, file: &mut impl Write) -> Result<usize, VMError> {
        let mut buffer: [u8; 1] = [0; 1];
        buffer[0] = self.current_cell().get_value();
        file.write_all(&buffer)
            .and_then(|_| file.flush())
            .map_err(|e| VMError::IOError {
                instruction: self.current_instruction().instruction(),
                source: e,
            })
            .and(Ok(self.next_instruction()))
    }

    pub fn open_loop(&mut self) -> Result<usize, VMError> {
        if self.current_cell().is_zero() {
            assert!(matches!(
                self.current_instruction(),
                DecoratedInstruction::OpenLoop { .. }
            ));
            match self.current_instruction() {
                DecoratedInstruction::OpenLoop {
                    instruction: _,
                    closer,
                } => Ok(self
                    .prog
                    .position_to_index(closer.line(), closer.character())
                    + 1),
                _ => unreachable!(),
            }
        } else {
            Ok(self.next_instruction())
        }
    }

    pub fn close_loop(&mut self) -> Result<usize, VMError> {
        assert!(matches!(
            self.current_instruction(),
            DecoratedInstruction::CloseLoop { .. }
        ));

        match self.current_instruction() {
            DecoratedInstruction::CloseLoop {
                instruction: _,
                opener,
            } => Ok(self
                .prog()
                .position_to_index(opener.line(), opener.character())),
            _ => unreachable!(),
        }
    }

    pub fn interpret(
        &mut self,
        input: &mut impl Read,
        output: &mut impl Write,
    ) -> Result<(), VMError> {
        while self.instruction_pointer < self.prog().decorated_instructions().len() {
            self.instruction_pointer = self.interpret_current_instruction(input, output)?
        }
        Ok(())
    }

    fn interpret_current_instruction(
        &mut self,
        input: &mut impl Read,
        output: &mut impl Write,
    ) -> Result<usize, VMError> {
        match self.current_instruction().instruction().instruction() {
            RawInstruction::OpenLoop => self.open_loop(),
            RawInstruction::CloseLoop => self.close_loop(),
            RawInstruction::DecrementDataPointer => self.seek_left(),
            RawInstruction::IncrementDataPointer => self.seek_right(),
            RawInstruction::IncrementByte => self.increment_cell(),
            RawInstruction::DecrementByte => self.decrement_cell(),
            RawInstruction::GetByte => self.read_value(input),
            RawInstruction::PutByte => self.write_value(output),
        }
    }
}

/// Runtime errors in the interpreter
#[derive(Error, Debug)]
pub enum VMError {
    #[error("Instruction {0} tried to seek to a negative head position")]
    SeekTooLow(PositionedInstruction),
    #[error("Instruction {0} tried to seek beyond the end of the cells and the cells aren't permitted to grow")]
    SeekTooHigh(PositionedInstruction),
    #[error("An I/O Error occurred while processing instruction {instruction}")]
    IOError {
        instruction: PositionedInstruction,
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that a simple "hello world" program works
    #[test]
    fn test_hello_world() {
        use bft_types::{DecoratedProgram, Program};
        let hello_world_text =
            ">++++++++[<+++++++++>-]<.>++++[<+++++++>-]<+.+++++++..+++.>>++++++[<+++++++>-]<+
        +.------------.>++++++[<+++++++++>-]<+.<.+++.------.--------.>>>++++[<++++++++>-
        ]<+.";
        let mut input = std::io::Cursor::new(Vec::new());
        let mut output = std::io::Cursor::new(Vec::new());
        let prog = Program::new("<no program>", &hello_world_text);
        let decorated = DecoratedProgram::from_program(&prog).unwrap();
        let mut machine: Machine<u8> = Machine::new(None, false, &decorated);
        let result = machine.interpret(&mut input, &mut output);
        assert!(result.is_ok());
        assert_eq!(output.into_inner(), "Hello, World!".as_bytes());
    }
}
