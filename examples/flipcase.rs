//! An example of using patharg with clap
//!
//! Run with `cargo run --example flipcase --features examples`
//!
//! Say you're using clap to write a command named "flipcase" that flips the
//! cases of all the letters in a file, changing uppercase to lowercase and
//! lowercase to uppercase, and you want this command to take a path to a file
//! to read input from and another path for the file to write the output to.
//! By using `patharg::InputArg` and `patharg::OutputArg` as the types of the
//! arguments in your `clap::Parser`, flipcase will treat a '-' argument as
//! referring to stdin/stdout, and you'll be able to use the types' methods to
//! read or write from any filepath or standard stream given on the command
//! line.
//!
//! flipcase can then be invoked in the following ways:
//!
//! - `flipcase` — Read input from stdin, write output to stdout
//! - `flipcase file.txt` — Read input from `file.txt`, write output to stdout
//! - `flipcase -` — Read input from stdin, write output to stdout
//! - `flipcase -o out.txt` — Read input from stdin, write output to `out.txt`
//! - `flipcase -o -` — Read input from stdin, write output to stdout
//! - `flipcase -o out.txt file.txt` — Read input from `file.txt`, write output
//!    to `out.txt`
//! - `flipcase -o out.txt -` — Read input from stdin, write output to
//!   `out.txt`
//! - `flipcase -o - file.txt` — Read input from `file.txt`, write output to
//!    stdout
//! - `flipcase -o - -` — Read input from stdin, write output to stdout

use clap::Parser;
use patharg::{InputArg, OutputArg};
use std::io::Write;

/// Flip the cases of all the letters in a file.
#[derive(Parser)]
struct Arguments {
    /// The file to write the case-flipped text to.
    #[clap(short = 'o', long, default_value_t)]
    // The `default_value_t` attribute causes the default value of the argument
    // to be `OutputArg::default()`, which equals `OutputArg::Stdout`.
    outfile: OutputArg,

    /// The file to read the text to case-flip from.
    #[clap(default_value_t)]
    // The `default_value_t` attribute causes the default value of the argument
    // to be `InputArg::default()`, which equals `InputArg::Stdin`.
    infile: InputArg,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut output = args.outfile.create()?;
    for r in args.infile.lines()? {
        let line = r?;
        writeln!(&mut output, "{}", flipcase(line))?;
    }
    Ok(())
}

fn flipcase(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_uppercase() {
            out.extend(c.to_lowercase());
        } else if c.is_lowercase() {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    out
}
