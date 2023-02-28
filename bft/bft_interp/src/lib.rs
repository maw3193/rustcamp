use bft_types::Program;

pub struct Machine<T> {
    cells: Vec<T>,
    head: usize,
    may_grow: bool,
}

impl<T> Machine<T>
where
    T: std::clone::Clone + Default,
{
    pub fn new(mut size: usize, may_grow: bool) -> Machine<T> {
        if size == 0 {
            size = 30000;
        }
        let cells = vec![Default::default(); size];
        Machine {
            head: 0,
            cells: cells,
            may_grow: may_grow,
        }
    }

    // TODO: Take this out of the constrained impl block
    // Seems very redundant, will probably be replaced with something else later
    pub fn print_program(&self, prog: &Program) {
        print!("{prog}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
