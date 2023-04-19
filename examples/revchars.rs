//! An example of using patharg with lexopt
//!
//! Say you're using lexopt to write a command named "revchars" that reverses
//! the order of all characters in a file, and you want this command to take a
//! path to a file to read input from and another path for the file to write
//! the output to.  Converting the command's arguments to `patharg::PathArg`
//! will make revchars treat a '-' argument as referring to stdin/stdout, and
//! you'll be able to use `PathArg`'s methods to read or write from any
//! filepath or standard stream given on the command line.
//!
//! revchars can then be invoked in the following ways:
//!
//! - `revchars` — Read input from stdin, write output to stdout
//! - `revchars file.txt` — Read input from `file.txt`, write output to stdout
//! - `revchars -` — Read input from stdin, write output to stdout
//! - `revchars -o out.txt` — Read input from stdin, write output to `out.txt`
//! - `revchars -o -` — Read input from stdin, write output to stdout
//! - `revchars -o out.txt file.txt` — Read input from `file.txt`, write output
//!    to `out.txt`
//! - `revchars -o out.txt -` — Read input from stdin, write output to
//!    `out.txt`
//! - `revchars -o - file.txt` — Read input from `file.txt`, write output to
//!    stdout
//! - `revchars -o - -` — Read input from stdin, write output to stdout

use lexopt::{Arg, Parser};
use patharg::PathArg;
use std::error::Error;

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Run { outfile: PathArg, infile: PathArg },
    Help,
    Version,
}

impl Command {
    fn from_parser(mut parser: Parser) -> Result<Command, lexopt::Error> {
        let mut infile: Option<PathArg> = None;
        let mut outfile: Option<PathArg> = None;
        while let Some(arg) = parser.next()? {
            match arg {
                Arg::Short('h') | Arg::Long("help") => return Ok(Command::Help),
                Arg::Short('V') | Arg::Long("version") => return Ok(Command::Version),
                Arg::Short('o') | Arg::Long("outfile") => {
                    outfile = Some(PathArg::from_arg(parser.value()?));
                }
                Arg::Value(val) if infile.is_none() => {
                    infile = Some(PathArg::from_arg(val));
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(Command::Run {
            infile: infile.unwrap_or_default(),
            outfile: outfile.unwrap_or_default(),
        })
    }

    fn run(self) -> std::io::Result<()> {
        match self {
            Command::Help => {
                println!("Usage: revchars [-o|--outfile <PATH>] [<PATH>]");
                println!();
                println!("Reverse the characters in a file");
                println!();
                println!("Options:");
                println!("  -o <PATH>, --outfile <PATH>");
                println!(
                    "                    The file to write the reversed text to [default: stdout]"
                );
                println!("  -h, --help        Display this help message and exit");
                println!("  -V, --version     Show the program version and exit");
                Ok(())
            }
            Command::Version => {
                println!("revchars: patharg {} example", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
            Command::Run { infile, outfile } => {
                let content = infile.read_to_string()?;
                let tnetnoc = content.chars().rev().collect::<String>();
                outfile.write(tnetnoc)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(Command::from_parser(Parser::from_env())?.run()?)
}
