//! Brainfuck interpreter library
//! An implementation of the brainfuck virtual machine

use std::num::NonZeroUsize;

use bft_types::{DecoratedInstruction, DecoratedProgram, PositionedInstruction};

use thiserror::Error;

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
    T: std::clone::Clone + Default,
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
}

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Instruction {0} tried to seek to a negative head position")]
    SeekTooLow(PositionedInstruction),
    #[error("Instruction {0} tried to seek beyond the end of the cells and the cells aren't permitted to grow")]
    SeekTooHigh(PositionedInstruction),
}
