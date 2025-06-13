This is an example program demontrating use of `patharg` with
[`clap`](https://crates.io/crates/clap) and
[`tokio`](https://crates.io/crates/tokio).

Say you're using `clap` to write a command named "`tokio-flipcase`" that flips
the cases of all the letters in a file, changing uppercase to lowercase and
lowercase to uppercase, and you want this command to take a path to a file to
read input from and another path for the file to write the output to.  By using
`patharg::InputArg` and `patharg::OutputArg` as the types of the arguments in
your `clap::Parser`, `tokio-flipcase` will treat a '-' argument as referring to
stdin/stdout, and you'll be able to use the types' methods to read or write
from any filepath or standard stream given on the command line.

`tokio-flipcase` can then be invoked in the following ways:

- `tokio-flipcase` — Read input from stdin, write output to stdout
- `tokio-flipcase file.txt` — Read input from `file.txt`, write output to
  stdout
- `tokio-flipcase -` — Read input from stdin, write output to stdout
- `tokio-flipcase -o out.txt` — Read input from stdin, write output to
  `out.txt`
- `tokio-flipcase -o -` — Read input from stdin, write output to stdout
- `tokio-flipcase -o out.txt file.txt` — Read input from `file.txt`, write
  output to `out.txt`
- `tokio-flipcase -o out.txt -` — Read input from stdin, write output to
  `out.txt`
- `tokio-flipcase -o - file.txt` — Read input from `file.txt`, write output to
  stdout
- `tokio-flipcase -o - -` — Read input from stdin, write output to stdout
