This is an example program demontrating use of `patharg` with
[`lexopt`](https://crates.io/crates/lexopt) and
[`tokio`](https://crates.io/crates/tokio).

Say you're using `lexopt` to write a command named "`tokio-revchars`" that
reverses the order of all characters in a file, and you want this command to
take a path to a file to read input from and another path for the file to write
the output to.  Converting the command's arguments to `patharg::InputArg` and
`patharg::OutputArg` will make `tokio-revchars` treat a '-' argument as
referring to stdin/stdout, and you'll be able to use the types' methods to read
or write from any filepath or standard stream given on the command line.

`tokio-revchars` can then be invoked in the following ways:

- `tokio-revchars` — Read input from stdin, write output to stdout
- `tokio-revchars file.txt` — Read input from `file.txt`, write output to
  stdout
- `tokio-revchars -` — Read input from stdin, write output to stdout
- `tokio-revchars -o out.txt` — Read input from stdin, write output to
  `out.txt`
- `tokio-revchars -o -` — Read input from stdin, write output to stdout
- `tokio-revchars -o out.txt file.txt` — Read input from `file.txt`, write
  output to `out.txt`
- `tokio-revchars -o out.txt -` — Read input from stdin, write output to
  `out.txt`
- `tokio-revchars -o - file.txt` — Read input from `file.txt`, write output to
  stdout
- `tokio-revchars -o - -` — Read input from stdin, write output to stdout
