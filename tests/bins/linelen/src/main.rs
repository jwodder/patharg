use clap::Parser;
use patharg::{InputArg, OutputArg};
use std::io::Write;

#[derive(Parser)]
struct Arguments {
    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    #[arg(default_value_t)]
    infile: InputArg,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut output = args.outfile.create()?;
    for r in args.infile.lines()? {
        let line = r?;
        writeln!(&mut output, "{}", line.len())?;
    }
    Ok(())
}
