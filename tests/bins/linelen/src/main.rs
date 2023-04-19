use clap::Parser;
use patharg::PathArg;
use std::io::Write;

#[derive(Parser)]
struct Arguments {
    #[clap(short = 'o', long, default_value_t)]
    outfile: PathArg,

    #[clap(default_value_t)]
    infile: PathArg,
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
