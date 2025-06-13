This is an example program demontrating use of `patharg` with
[`clap`](https://crates.io/crates/clap).

Say you're using `clap` to write a command named "`flipcase`" that flips the
cases of all the letters in a file, changing uppercase to lowercase and
lowercase to uppercase, and you want this command to take a path to a file to
read input from and another path for the file to write the output to.  By using
`patharg::InputArg` and `patharg::OutputArg` as the types of the arguments in
your `clap::Parser`, `flipcase` will treat a '-' argument as referring to
stdin/stdout, and you'll be able to use the types' methods to read or write
from any filepath or standard stream given on the command line.

`flipcase` can then be invoked in the following ways:

- `flipcase` — Read input from stdin, write output to stdout
- `flipcase file.txt` — Read input from `file.txt`, write output to stdout
- `flipcase -` — Read input from stdin, write output to stdout
- `flipcase -o out.txt` — Read input from stdin, write output to `out.txt`
- `flipcase -o -` — Read input from stdin, write output to stdout
- `flipcase -o out.txt file.txt` — Read input from `file.txt`, write output to
  `out.txt`
- `flipcase -o out.txt -` — Read input from stdin, write output to `out.txt`
- `flipcase -o - file.txt` — Read input from `file.txt`, write output to stdout
- `flipcase -o - -` — Read input from stdin, write output to stdout
