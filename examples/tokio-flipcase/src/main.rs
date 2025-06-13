#![allow(missing_docs)]
use clap::Parser;
use patharg::{InputArg, OutputArg};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

/// Flip the cases of all the letters in a file.
#[derive(Parser)]
struct Arguments {
    /// The file to write the case-flipped text to.
    #[arg(short = 'o', long, default_value_t)]
    // The `default_value_t` attribute causes the default value of the argument
    // to be `OutputArg::default()`, which equals `OutputArg::Stdout`.
    outfile: OutputArg,

    /// The file to read the text to case-flip from.
    #[arg(default_value_t)]
    // The `default_value_t` attribute causes the default value of the argument
    // to be `InputArg::default()`, which equals `InputArg::Stdin`.
    infile: InputArg,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let mut output = args.outfile.async_create().await?;
    let mut stream = args.infile.async_lines().await?;
    while let Some(r) = stream.next().await {
        let line = r?;
        let mut flipped = flipcase(line);
        flipped.push('\n');
        output.write_all(flipped.as_ref()).await?;
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
