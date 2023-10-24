use clap::Parser;
use patharg::{InputArg, OutputArg};

#[derive(Parser)]
struct Arguments {
    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    #[arg(default_value_t)]
    infile: InputArg,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut input = args.infile.async_read().await?;
    input.reverse();
    args.outfile.async_write(input).await
}
