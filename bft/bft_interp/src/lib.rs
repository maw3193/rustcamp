//! Brainfuck interpreter library
//! An implementation of the brainfuck virtual machine

use std::num::NonZeroUsize;

use bft_types::{DecoratedProgram, ParseError, Program};

/// A brainfuck virtual machine
/// The type T is the type that all brainfuck cells will be.
/// The machine is initialised with a specific number of cells, and may
/// allocate more cells when the head extends beyond the end of memory if
/// configured to do so.
pub struct Machine<T> {
    cells: Vec<T>,
    head: usize,
    may_grow: bool,
}

impl<T> Machine<T> {
    pub fn print_program(&self, prog: &Program) {
        print!("{prog}")
    }

    pub fn cells(&self) -> &[T] {
        self.cells.as_ref()
    }

    pub fn head(&self) -> usize {
        self.head
    }

    pub fn may_grow(&self) -> bool {
        self.may_grow
    }

    /// Checks whether a provided program is valid Brainfuck code
    ///
    /// As it's a very thin wrapper around DecoratedProgram::from_program,
    /// see that for examples of how this is used.
    pub fn validate(&self, prog: &Program) -> Result<(), ParseError> {
        // Reuse existing parsing logic, throwing away the success
        // and propagating any errors
        DecoratedProgram::from_program(prog)?;
        Ok(())
    }
}

impl<T> Machine<T>
where
    T: std::clone::Clone + Default,
{
    /// Creates a new virtual machine of the specified size, type, and whether it can grow.
    /// If `size` is set to 0, it will choose the default, 30000.
    /// # Examples
    /// ```
    /// # use bft_interp;
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(None, false);
    /// ```
    pub fn new(size: Option<NonZeroUsize>, may_grow: bool) -> Machine<T> {
        let size = match size {
            None => 30000,
            Some(sz) => sz.into(),
        };
        let cells = vec![Default::default(); size];
        Machine {
            head: 0,
            cells,
            may_grow,
        }
    }
}
