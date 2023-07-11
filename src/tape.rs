use std::{fmt, io};
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::io::Read;

fn default_stdin() -> Box<dyn Iterator<Item=u8>> {
    Box::new(io::stdin().bytes().map(Result::unwrap))
}

fn default_stdout() -> Box<dyn io::Write> {
    Box::new(io::stdout())
}

pub struct Tape {
    pointer: isize,
    values: Vec<u8>,
    /// Index of the value that corresponds to the initial cell.
    origin: isize,
    hex_output: bool,
    stdin: Box<dyn Iterator<Item=u8>>,
    stdout: Box<dyn io::Write>,
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            pointer: 0,
            values: Vec::new(),
            origin: 0,
            hex_output: false,
            stdin: default_stdin(),
            stdout: default_stdout(),
        }
    }
}

impl Tape {
    pub fn new(hex_output: bool) -> Self {
        Self {
            hex_output,
            ..Self::default()
        }
    }

    /// Moves the cell pointer to the right by the specified amount.
    pub fn right_by(&mut self, amount: isize) {
        self.pointer += amount;
    }

    fn first_index(&self) -> isize {
        -self.origin
    }

    fn last_index(&self) -> isize {
        (min(self.values.len(), isize::MAX as usize) as isize) - self.origin - 1
    }

    /// Gets the value of the cell at the specified index.
    fn read_cell(&self, index: isize) -> u8 {
        if (self.first_index()..=self.last_index()).contains(&index) {
            self.values[(self.origin + index) as usize]
        } else {
            0
        }
    }

    /// Returns the value of the cell to the right of the pointer by the specified offset.
    pub fn read_relative(&self, offset: isize) -> u8 {
        self.read_cell(self.pointer + offset)
    }

    /// Returns the value of the current as a `u8`.
    ///
    /// To get the value of the current cell as a `char`, use [`Self::read_char`].
    pub fn read(&self) -> u8 {
        self.read_relative(0)
    }

    /// Extends the tape to make the specified index valid.
    fn extend_to_index(&mut self, index: isize) {
        if index > self.last_index() {
            self.values.resize((self.origin + index + 1) as usize, 0)
        } else if index < self.first_index() {
            let mut new_values: Vec<u8> = Vec::new();
            new_values.resize((-index - self.origin) as usize, 0);
            new_values.append(&mut self.values);
            self.values = new_values;
            self.origin += -index - self.origin
        }
    }

    /// Returns a mutable reference to the cell at the passed index.
    fn get_cell(&mut self, index: isize) -> &mut u8 {
        self.extend_to_index(index);
        &mut self.values[(self.origin + index) as usize]
    }

    /// Returns a mutable reference to the slice from `from` to `to` (both included).
    fn get_slice(&mut self, from: isize, to: isize) -> &mut [u8] {
        assert!(from <= to);
        self.extend_to_index(from);
        self.extend_to_index(to);
        let (i, j) = ((self.origin + from) as usize, (self.origin + to) as usize);
        &mut self.values[i..=j]
    }

    /// Sets the value of the cell to the right of the pointer by the specified offset.
    pub fn write_relative(&mut self, offset: isize, value: u8) {
        let cell = self.get_cell(self.pointer + offset);
        *cell = value;
    }

    /// Sets the value of the current cell.
    pub fn write(&mut self, value: u8) {
        self.write_relative(0, value)
    }

    /// Fills the values of the cells between the current cell and the cell to the right of the
    /// pointer by the specified offset (both included) with the provided value.
    pub fn fill(&mut self, max_offset: isize, value: u8) {
        let from = min(self.pointer, self.pointer + max_offset);
        let to = max(self.pointer, self.pointer + max_offset);
        let slice = self.get_slice(from, to);
        slice.fill(value)
    }

    /// Adds the passed amount to the value of the cell to the right of the pointer by the specified
    /// offset.
    pub fn add(&mut self, offset: isize, amount: u8) {
        let cell = self.get_cell(self.pointer + offset);
        *cell = cell.wrapping_add(amount);
    }

    /// Outputs the value of the current cell to this tape's `stdout`.
    pub fn output(&mut self) {
        if self.hex_output {
            writeln!(self.stdout, "0x{:02x}", self.read()).unwrap()
        } else {
            write!(self.stdout, "{}", self.read() as char).unwrap()
        }
    }

    /// Sets the value of the current cell from this tape's `stdin`.
    pub fn input(&mut self) {
        let value = self.stdin.next().unwrap();
        self.write_relative(0, value)
    }
}

impl Display for Tape {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Print cell values
        for i in self.first_index()..self.last_index() {
            write!(f, "0x{:>02x} | ", self.read_cell(i))?;
        }
        writeln!(f, "0x{:>02x}", self.read_cell(self.last_index()))?;
        // Print cell indices
        for i in -self.first_index()..=self.last_index() {
            write!(f, "{:>4}   ", i)?;
        }
        Ok(())
    }
}
