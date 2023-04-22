use clap::Parser;
use patharg::{InputArg, OutputArg};

#[derive(Parser)]
struct Arguments {
    #[clap(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    #[clap(default_value_t)]
    infile: InputArg,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut input = args.infile.async_read().await?;
    input.reverse();
    args.outfile.async_write(input).await
}
