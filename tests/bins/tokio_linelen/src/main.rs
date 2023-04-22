use clap::Parser;
use patharg::{InputArg, OutputArg};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

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
    let mut output = args.outfile.async_create().await?;
    let mut stream = args.infile.async_lines().await?;
    while let Some(r) = stream.next().await {
        let line = r?;
        let s = format!("{}\n", line.len());
        output.write_all(s.as_ref()).await?;
    }
    Ok(())
}
