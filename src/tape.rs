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

#[derive(Debug, Clone, Copy)]
enum OutputMode {
    Ascii,
    Hex,
    Silent,
}

/// A Brainfuck tape.
///
/// It is infinitely expandable in both directions, and each cell contains a `u8`.
pub struct Tape {
    /// The current position of the pointer.
    pointer: isize,
    /// The values of the cells.
    values: Vec<u8>,
    /// Index of the value that corresponds to the initial cell.
    origin: isize,
    /// The output mode.
    output_mode: OutputMode,
    /// The iterator [`Tape::input`]  should read from.
    stdin: Box<dyn Iterator<Item=u8>>,
    /// The file [`Tape::output`]  should write to.
    stdout: Box<dyn io::Write>,
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            pointer: 0,
            values: Vec::new(),
            origin: 0,
            output_mode: OutputMode::Ascii,
            stdin: default_stdin(),
            stdout: default_stdout(),
        }
    }
}

impl Tape {
    pub fn new(hex_output: bool, silent: bool) -> Self {
        let output_mode =
            if silent {
                OutputMode::Silent
            } else if hex_output {
                OutputMode::Hex
            } else {
                OutputMode::Ascii
            };
        Self {
            output_mode,
            ..Self::default()
        }
    }

    /// Moves the cell pointer to the right by a specific amount.
    pub fn right_by(&mut self, amount: isize) {
        self.pointer += amount;
    }

    fn first_index(&self) -> isize {
        -self.origin
    }

    fn last_index(&self) -> isize {
        (min(self.values.len(), isize::MAX as usize) as isize) - self.origin - 1
    }

    /// Gets the value of a cell.
    fn read_cell(&self, index: isize) -> u8 {
        if (self.first_index()..=self.last_index()).contains(&index) {
            self.values[(self.origin + index) as usize]
        } else {
            0
        }
    }

    /// Returns the value of the cell to the right of the pointer by a specific offset.
    pub fn read_relative(&self, offset: isize) -> u8 {
        self.read_cell(self.pointer + offset)
    }

    /// Returns the value of the current cell as a `u8`.
    pub fn read(&self) -> u8 {
        self.read_relative(0)
    }

    /// Extends the tape to make the specified index valid in the underlying vector.
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

    /// Returns a mutable reference to a cell.
    fn get_cell(&mut self, index: isize) -> &mut u8 {
        self.extend_to_index(index);
        &mut self.values[(self.origin + index) as usize]
    }

    /// Returns a mutable reference to the slice from `from` to `to` (both included).
    fn get_slice(&mut self, from: isize, to: isize) -> &mut [u8] {
        assert!(from <= to);
        self.extend_to_index(to);
        self.extend_to_index(from);
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
    /// pointer by the specified offset (both included) with a specific value.
    pub fn fill(&mut self, max_offset: isize, value: u8) {
        let from = min(self.pointer, self.pointer + max_offset);
        let to = max(self.pointer, self.pointer + max_offset);
        let slice = self.get_slice(from, to);
        slice.fill(value)
    }

    /// Adds a specific amount to the value of the cell to the right of the pointer by the specified
    /// offset.
    pub fn add(&mut self, offset: isize, amount: u8) {
        let cell = self.get_cell(self.pointer + offset);
        *cell = cell.wrapping_add(amount);
    }

    /// Outputs the value of the current cell to this tape's `stdout`.
    pub fn output(&mut self) {
        match self.output_mode {
            OutputMode::Ascii => write!(self.stdout, "{}", self.read() as char).unwrap(),
            OutputMode::Hex => writeln!(self.stdout, "0x{:02x}", self.read()).unwrap(),
            _ => {}
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
        let range = if self.values.is_empty() {
            0..=0
        } else {
            self.first_index()..=self.last_index()
        };
        // Print cell values
        for i in range.clone() {
            write!(f, "| 0x{:>02x} ", self.read_cell(i))?;
        }
        writeln!(f, "|")?;
        // Print cell indices
        for i in range {
            write!(f, "  {:>4} ", i)?;
        }
        Ok(())
    }
}
