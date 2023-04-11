//! Brainfuck interpreter library
//! An implementation of the brainfuck virtual machine

use std::io::{Read, Write};
use std::num::NonZeroUsize;

use bft_types::{DecoratedInstruction, DecoratedProgram, PositionedInstruction};

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
    pub fn seek_left(&mut self) -> Result<(), VMError> {
        if self.head == 0 {
            Err(VMError::SeekTooLow(
                self.current_instruction().instruction(),
            ))
        } else {
            self.head -= 1;
            Ok(())
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
    pub fn seek_right(&mut self) -> Result<(), VMError> {
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
        Ok(())
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
    pub fn increment_cell(&mut self) {
        self.cells[self.head].increment()
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
    pub fn decrement_cell(&mut self) {
        self.cells[self.head].decrement()
    }

    /// Read a value from `file` into memory at the memory pointer
    ///
    /// If an I/O Error occurs while trying to read the file, it returns that error wrapped inside a [VMError].
    ///
    /// Because I couldn't think of what brainfuck should do if it ever runs out of bytes reading stdin, it also
    /// errors if zero bytes are read.
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
    pub fn read_value(&mut self, file: &mut impl Read) -> Result<(), VMError> {
        let mut buffer: [u8; 1] = [0; 1];
        match file.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // This *could* just mean "end of file", but I don't know what brainfuck should do in that case
                    Err(VMError::IOError {
                        instruction: self.current_instruction().instruction(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unexpected zero bytes read",
                        ),
                    })
                } else {
                    self.cells[self.head].set_value(buffer[0]);
                    Ok(())
                }
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
    /// Because I can't think of how brainfuck should respond to the file not accepting any more bytes, it also
    /// errors if zero bytes are written.
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
    pub fn write_value(&mut self, file: &mut impl Write) -> Result<(), VMError> {
        let mut buffer: [u8; 1] = [0; 1];
        buffer[0] = self.cells[self.head].get_value();
        match file.write(&buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // This *could* mean that the output file couldn't accept any more bytes.
                    // I don't know what brainfuck should do in this case, other than error.
                    Err(VMError::IOError {
                        instruction: self.current_instruction().instruction(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unexpected zero bytes written",
                        ),
                    })
                } else {
                    Ok(())
                }
            }
            Err(ioerror) => Err(VMError::IOError {
                instruction: self.current_instruction().instruction(),
                source: ioerror,
            }),
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
