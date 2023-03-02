//! Brainfuck interpreter library
//! An implementation of the brainfuck virtual machine

use bft_types::Program;

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
    /// let mut interp: bft_interp::Machine<u8> = bft_interp::Machine::new(0, false);
    /// ```
    pub fn new(mut size: usize, may_grow: bool) -> Machine<T> {
        if size == 0 {
            size = 30000;
        }
        let cells = vec![Default::default(); size];
        Machine {
            head: 0,
            cells,
            may_grow,
        }
    }
}
