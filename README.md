# Brainfuck Interpreter

_This was one of my first Rust programs. Please excuse the ugly code._

A Brainfuck interpreter that uses an intermediate representation to optimize some patterns in order to make the execution faster.

## About Brainfuck

Brainfuck is an extremely basic programming language created by Urban Müller in 1993. A Brainfuck program consists of a string of characters each representing an instruction (or a comment). For the list of valid instructions, see [the Wikipedia article on Brainfuck](https://en.wikipedia.org/wiki/Brainfuck). Unlike most languages, there is no official specification for Brainfuck. This means behavior may vary slightly between interpreters. This interpreter uses the original interpreter as the definition of Brainfuck, with the exception that the array is expected to be infinitely expandable in both directions.

## How it works

There are multiple kinds of patterns the compiler is able to recognize and optimize. This list might be extended later, but it is hard to come up with patterns that are worth optimizing.

Here is a non-exhaustive list of criteria a pattern should fulfill in order for it to be worth optimizing:

- **High cost / complexity ratio.** A good pattern is very expensive to execute normally, but not too complex, making it easy to detect.
- **Commonness.** A good pattern should be reasonably common, as we don't want to spend ages looking for a pattern we will only encounter once.

The following paragraphs describe patterns that are currently detected and optimized.

### Successive `+`, `-`, `<`, and `>`

When a sequence of more than one `+` (resp., `-`, `<`, `>`) is found, the amount of times the instruction appears is only counted once, and the sequence is reduced to a single command that increments (resp., decrements, moves the pointer to the left, to the right) by this amount. For example, the following sequences are reduced to a single operation:

| Original instructions | Optimized command |
|-----------------------|-------------------|
| `++++++`              | `Add(6)`          |
| `----`                | `Add(252)`        |
| `-++-+`               | `Add(1)`          |
| `>>>`                 | `Right(3)`        |
| `<<<<`                | `Right(-4)`       |
| `<<<>`                | `Right(-2)`       |

This is especially efficient for deeply-nested instructions, as we only need to count them once, instead of every time we execute them.

As a side effect, instructions that trivially cancel each other (namely: `+-`, `-+`, `><`, and `<>`) are completely removed.

### Resets

A very common pattern in Brainfuck is to reset a cell with `[-]`, or `[+]`. This kind of pattern, where the body of a loop can be reduced to a single `Add(2n + 1)`, is optimized to a single `Reset` command.

While we are at it, we can also detect when a chunk of adjacent cells are reset. This is usually not worth the cost, which is why this specific optimization is disabled by default. For example, the following piece of code is detected as resetting a chunk of 5 adjacent cells:

```brainfuck
[-]>[+]>[---]>[+++]>[+-+]
```

### Moves

The interpreter is also able to recognize moves. That is, when the value of a cell is added to one or multiple other cells, with an optional scaling factor. For example, consider the following piece of code:

```brainfuck
[->+<]
```

This code fragment moves (or rather, adds) the value of the initial cell to the cell to its right. This pattern is detected and optimized, by the interpreter.

A more convoluted example would be the following:

```brainfuck
[->+>++>+++<<<]
```

Here, if we let `n` be the initial value of the initial cell, we compute `n` in the second cell, `2 * n` in the third one, and `3 * n` in the fourth one. This pattern is also detected and optimized.

In general, any loop that:

- Is balanced (ends on the same cell as the one where it started),
- Decrements its origin by one each iteration,
- Increments other cells one or multiple times each iteration,
- And *nothing else*;

is optimized.

## Why bother optimizing?

You might think performing those optimizations is useless. After all, we need to read the whole source code to optimize it. Why not just execute it instead?

Although this reasoning is true for top-level optimizations, this is not the case for deeply-nested loops. For example, consider the following piece of code:

```brainfuck
>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
```

This piece of code moves the pointer to the right by 200 cells. If this is in a loop, we need to count all those `>` every time we enter the loop. But it is obvious that, once we have counted them once, we should just "remember" how many there were, and move the cursor by this amount in a single command next time. This is exactly what we do, by first optimizing all those patterns throughout the whole source code, and then executing the optimized commands.

Empirically, you can try running a complex Brainfuck program with minimal optimizations, and notice how slow it is compared to with optimizations enabled.

## Usage

The interpreter accepts a path to a file containing Brainfuck source code as a command line argument. You can run the program with `--help` to get a list of available options.

### Example

The following command runs the interpreter (`./brainfuck-interpreter`) on `program.bf` with minimal optimizations:

```shell
$ ./brainfuck-interpreter program.bf --optimize-loops false --optimize-chunk-resets false
```

## Build from sources

If you have installed the [Rust toolchain](https://www.rust-lang.org/tools/install) on your machine, you can build an executable version of the interpreter with:

```shell
$ cargo build --release
```
