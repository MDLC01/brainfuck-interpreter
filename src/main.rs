#![warn(missing_debug_implementations)]

use std::fmt::Display;
use std::fs;
use std::time::SystemTime;

use clap::Parser;

use crate::args::Args;
use crate::tape::Tape;

mod tape;
mod args;


/// Commands represent higher level concepts than regular Brainfuck instructions. The goal is that a
/// specific command can be executed in less time than it would take for it to be executed if it was
/// made up of multiple regular Brainfuck instructions with the same effect.
#[derive(Debug)]
enum Command {
    /// Moves the pointer to the right by a specific amount (to the left if negative).
    Right(isize),
    /// Adds a specific amount to the current cell.
    Add(u8),
    /// Repeats commands until the current cell reaches 0.
    Loop(Vec<Command>),
    /// Sets the value of the current cell to a byte read from `stdin`.
    Input,
    /// Outputs the value of the current cell to `stdout`.
    Output,
    /// Resets the value of the current cell to 0.
    Reset,
    /// Resets the values of the cells between the current cell and a specific cell (both included)
    /// to 0.
    ResetChunk(isize),
    /// Moves the value of the current cell to the cells at a specific position (relative to the
    /// current cell).
    ///
    /// Specifically:
    /// - For each element `(i, n)` in the vector, adds `n` times the value of the current cell to
    /// the value of the cell `i` cells to the right of the current cell;
    /// - Resets the current cell.
    ///
    /// The pointer is *not* moved.
    Move(Vec<(isize, u8)>),
}

impl Command {
    /// Tests if this command is useful.
    ///
    /// A command is useful if it is not functionally equivalent to doing nothing.
    fn is_useful(&self) -> bool {
        match self {
            Self::Right(0) | Self::Add(0) => false,
            _ => true,
        }
    }

    /// Tests if this comment increments the current cell by an odd amount.
    fn is_odd_increment(&self) -> bool {
        match self {
            Self::Add(n) => n % 2 == 1,
            _ => false,
        }
    }
}


/// Loads Brainfuck instructions from an iterator and pushes them to a vector as [commands](Command)
/// until the iterator yields `end`. Returns the constructed vector.
fn load(instructions: &mut impl Iterator<Item=char>, end: Option<char>) -> Vec<Command> {
    let mut commands = Vec::new();
    loop {
        match instructions.next() {
            Some('<') => {
                match commands.last_mut() {
                    Some(Command::Right(amount)) => *amount -= 1,
                    _ => commands.push(Command::Right(-1)),
                }
            }
            Some('>') => {
                match commands.last_mut() {
                    Some(Command::Right(amount)) => *amount += 1,
                    _ => commands.push(Command::Right(1)),
                }
            }
            Some('+') => {
                match commands.last_mut() {
                    Some(Command::Add(amount)) => *amount = amount.wrapping_add(1),
                    _ => commands.push(Command::Add(1)),
                }
            }
            Some('-') => {
                match commands.last_mut() {
                    Some(Command::Add(amount)) => *amount = amount.wrapping_sub(1),
                    _ => commands.push(Command::Add(u8::MAX)),
                }
            }
            Some('[') => {
                let loop_content = load(instructions, Some(']'));
                commands.push(Command::Loop(loop_content))
            }
            Some('.') => {
                commands.push(Command::Output)
            }
            Some(',') => {
                commands.push(Command::Input)
            }
            c if c == end => {
                return commands;
            }
            Some(']') => {
                panic!("Unexpected `]`")
            }
            None => {
                panic!("Unexpected EOF")
            }
            _ => {}
        }
    }
}


/// Returns a command that is functionally equivalent to a loop containing the passed commands.
fn optimize_loop(commands: Vec<Command>) -> Command {
    /// Tries to optimize a loop with the passed body as a move.
    ///
    /// If possible, returns [`Some(result)`], where `result` is a vector that can be used to
    /// construct [`Command::Move`]. Otherwise, returns [`None`].
    fn try_optimize_as_move(commands: &Vec<Command>) -> Option<Vec<(isize, u8)>> {
        let mut is_origin_decremented = false;
        // Note that, if a cell is incremented multiple times, at different places within the loop,
        // this will result in the vector containing multiple entries for this cell. Using a HashMap
        // to solve this "problem" results in much higher optimization times, though. So Vec it is.
        let mut increments = Vec::new();
        let mut offset = 0;
        for command in commands {
            match command {
                Command::Add(u8::MAX) if offset == 0 => {
                    if is_origin_decremented {
                        return None;
                    } else {
                        is_origin_decremented = true
                    }
                }
                Command::Add(amount) => {
                    increments.push((offset, *amount))
                }
                Command::Right(amount) => {
                    offset += amount
                }
                _ => {
                    return None;
                }
            }
        }
        if is_origin_decremented && offset == 0 {
            Some(increments)
        } else {
            None
        }
    }

    if commands.len() == 1 && commands[0].is_odd_increment() {
        Command::Reset
    } else if let Some(increments) = try_optimize_as_move(&commands) {
        Command::Move(increments)
    } else {
        Command::Loop(commands)
    }
}


/// Recursively optimizes chunk resets in the passed commands (including in nested loops).
fn optimize_chunk_resets(commands: impl Iterator<Item=Command>) -> impl Iterator<Item=Command> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        OutsideChunk,
        /// Associated value is the current offset from initial position.
        ExpectReset(isize),
        /// Associated value is the current offset from initial position.
        ExpectRight(isize),
    }
    let mut optimized_commands = Vec::new();
    let mut state = State::OutsideChunk;
    for command in commands {
        match (state, command) {
            (State::OutsideChunk, Command::Reset) => {
                state = State::ExpectRight(0)
            }
            (State::OutsideChunk, command) => {
                optimized_commands.push(command)
            }
            (State::ExpectRight(0), Command::Right(direction)) if direction.abs() == 1 => {
                state = State::ExpectReset(direction)
            }
            (State::ExpectRight(0), command) => {
                // This is actually a special case of a later case, but this one returns `Reset`
                // instead of `ResetChunk(0)`, which I guess is more optimized (though I have not
                // ran any test)...
                optimized_commands.push(Command::Reset);
                state = State::OutsideChunk;
                optimized_commands.push(command)
            }
            (State::ExpectReset(current_offset), Command::Reset) => {
                state = State::ExpectRight(current_offset)
            }
            (State::ExpectReset(current_offset), command) => {
                let extreme_cell_offset = current_offset - current_offset.signum();
                optimized_commands.push(Command::ResetChunk(extreme_cell_offset));
                optimized_commands.push(Command::Right(current_offset));
                state = State::OutsideChunk;
                optimized_commands.push(command)
            }
            (State::ExpectRight(current_offset), Command::Right(amount)) if amount == current_offset.signum() => {
                state = State::ExpectReset(current_offset + amount)
            }
            (State::ExpectRight(current_offset), command) => {
                optimized_commands.push(Command::ResetChunk(current_offset));
                optimized_commands.push(Command::Right(current_offset));
                state = State::OutsideChunk;
                optimized_commands.push(command)
            }
        }
    }
    match state {
        State::ExpectReset(current_offset) => {
            let extreme_cell_offset = current_offset - current_offset.signum();
            optimized_commands.push(Command::ResetChunk(extreme_cell_offset));
            optimized_commands.push(Command::Right(current_offset))
        }
        State::ExpectRight(current_offset) => {
            optimized_commands.push(Command::ResetChunk(current_offset));
            optimized_commands.push(Command::Right(current_offset))
        }
        _ => {}
    }
    optimized_commands.into_iter()
}

/// Returns a vector of commands that is functionally equivalent to the passed commands.
fn optimize(commands: Vec<Command>, args: &Args) -> Vec<Command> {
    let commands_iter = commands.into_iter()
        // Recursive call
        .map(|command| match command {
            Command::Loop(content) => Command::Loop(optimize(content, args)),
            _ => command
        })
        // Optimize trivial loops
        .map(|command| {
            if args.optimize_loops {
                match command {
                    Command::Loop(content) => optimize_loop(content),
                    _ => command
                }
            } else {
                command
            }
        })
        .filter(Command::is_useful);
    if args.optimize_chunk_resets {
        optimize_chunk_resets(commands_iter).collect()
    } else {
        commands_iter.collect()
    }
}


fn execute(commands: &[Command], tape: &mut Tape) {
    for command in commands {
        match command {
            Command::Right(amount) => {
                tape.right_by(*amount)
            }
            Command::Add(amount) => {
                tape.add(0, *amount)
            }
            Command::Loop(loop_commands) => {
                while tape.read() != 0 {
                    execute(loop_commands, tape)
                }
            }
            Command::Input => {
                tape.input()
            }
            Command::Output => {
                tape.output()
            }
            &Command::Reset => {
                tape.write(0)
            }
            &Command::ResetChunk(max_offset) => {
                tape.fill(max_offset, 0)
            }
            Command::Move(cells) => {
                let value = tape.read();
                for &(cell_offset, multiplier) in cells {
                    tape.add(cell_offset, value.wrapping_mul(multiplier))
                }
                tape.write(0)
            }
        }
    }
}


fn time<T>(description: impl Display, do_time: bool, f: impl FnOnce() -> T) -> T {
    if do_time {
        let start = SystemTime::now();
        let result = f();
        let end = SystemTime::now();
        let duration = end.duration_since(start).unwrap();
        eprintln!("{:16}\t{:>10.3} ms", description, duration.as_secs_f64() * 1000.0);
        result
    } else {
        f()
    }
}


fn main() {
    let args = Args::parse();

    let code = fs::read_to_string(&args.file).expect("Unable to read source file");

    let commands = time("Loading source", args.time, || load(&mut code.chars(), None));

    let optimized_commands = time("Optimizing", args.time, || optimize(commands, &args));

    time("Running", args.time, || execute(&optimized_commands, &mut Tape::new(args.hex_output)));
}
