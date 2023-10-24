use clap::Parser;
use patharg::{InputArg, OutputArg};

#[derive(Parser)]
struct Arguments {
    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    #[arg(default_value_t)]
    infile: InputArg,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut input = args.infile.read()?;
    input.reverse();
    args.outfile.write(input)
}
