use clap::Parser;
use patharg::PathArg;

#[derive(Parser)]
struct Arguments {
    #[clap(short = 'o', long, default_value_t)]
    outfile: PathArg,

    #[clap(default_value_t)]
    infile: PathArg,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut input = args.infile.read()?;
    input.reverse();
    args.outfile.write(input)
}
