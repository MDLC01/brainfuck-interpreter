use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// A Brainfuck interpreter that uses an intermediate representation to optimize some patterns in
/// order to make the execution faster.
#[derive(Debug, Parser)]
pub struct Args {
    /// A path to the file containing the Brainfuck source code to execute.
    pub file: PathBuf,
    /// Whether to enable loop optimizations (resets and moves).
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub optimize_loops: bool,
    /// Whether to optimize chunk resets (not recommended).
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    pub optimize_chunk_resets: bool,
    /// If passed, prints timing information to `stderr`.
    #[arg(long)]
    pub time: bool,
}
