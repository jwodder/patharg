[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/patharg/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/patharg/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/patharg/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/patharg)
[![MIT License](https://img.shields.io/github/license/jwodder/patharg.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/patharg) | [crates.io](https://crates.io/crates/patharg) | [Documentation](https://docs.rs/patharg) | [Issues](https://github.com/jwodder/patharg/issues)

Most CLI commands that take file paths as arguments follow the convention of
treating a path of `-` (a single hyphen/dash) as referring to either standard
input or standard output (depending on whether the path is read from or written
to).  The `patharg` crate lets your programs follow this convention too: it
provides `InputArg` and `OutputArg` types that wrap command-line arguments,
with methods for reading from/writing to either the given path or — if the
argument is just a hyphen — the appropriate standard stream.

`InputArg` and `OutputArg` implement `From<OsString>` and `From<String>`, so
you can use them seamlessly with your favorite Rust source of command-line
arguments, be it [`clap`][], [`lexopt`][], plain old
[`std::env::args`][args]/[`std::env::args_os`][args_os], or whatever else is
out there.  The source repository contains examples of two of these:

- [`examples/flipcase.rs`][flipcase] and
  [`examples/tokio-flipcase.rs`][tokio-flipcase] show how to use this crate
  with `clap`.
- [`examples/revchars.rs`][revchars] and
  [`examples/tokio-revchars.rs`][tokio-revchars] show how to use this crate
  with `lexopt`.

[`clap`]: https://crates.io/crates/clap
[`lexopt`]: https://crates.io/crates/lexopt
[args]: https://doc.rust-lang.org/std/env/fn.args.html
[args_os]: https://doc.rust-lang.org/std/env/fn.args_os.html
[flipcase]: https://github.com/jwodder/patharg/blob/master/examples/flipcase.rs
[tokio-flipcase]: https://github.com/jwodder/patharg/blob/master/examples/tokio-flipcase.rs
[revchars]: https://github.com/jwodder/patharg/blob/master/examples/revchars.rs
[tokio-revchars]: https://github.com/jwodder/patharg/blob/master/examples/tokio-revchars.rs

Installation
============

`patharg` requires version 1.65 of Rust or higher.  To use the `patharg`
library in your Cargo project, add the following to your `Cargo.toml`:

```toml
[dependencies]
patharg = "0.1.0"
```

Comparison with clio
====================

The only other library I am aware of that provides similar functionality to
`patharg` is [`clio`][].  Compared to `clio`, `patharg` aims to be a much
simpler, smaller library that doesn't try to be too clever.  Major differences
between the libraries include:

- When a `clio` path instance is created, `clio` will either (depending on the
  type used) open the path immediately — which can lead to empty files being
  needlessly left behind if an output file is constructed during argument
  processing but an error occurs before the file is actually used — or else
  check that the path can be opened — which is vulnerable to TOCTTOU bugs.
  `patharg` does no such thing.

- `clio` supports reading from & writing to HTTP(S) URLs and has special
  treatment for FIFOs.  `patharg` sees no need for such excesses.

[`clio`]: https://crates.io/crates/clio
