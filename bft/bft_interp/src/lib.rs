use std::default;

pub struct Machine<T> {
    cells: Vec<T>,
    head: usize,
    may_grow: bool,
}

impl<T> Machine<T>
where T: std::clone::Clone + Default
{
    pub fn new(mut size: usize, may_grow: bool) -> Machine<T> {
        if size == 0 {
            size = 30000;
        }
        let mut cells = vec![Default::default(); size];
        Machine {
            head: 0,
            cells: cells,
            may_grow: may_grow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}