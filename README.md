[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip) <!-- [![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active) -->
[![CI Status](https://github.com/jwodder/patharg/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/patharg/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/patharg/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/patharg)
[![MIT License](https://img.shields.io/github/license/jwodder/patharg.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/patharg) <!-- | [crates.io](https://crates.io/crates/patharg) | [Documentation](https://docs.rs/patharg) --> | [Issues](https://github.com/jwodder/patharg/issues)

Most CLI commands that take file paths as arguments follow the convention of
treating a path of `-` (a single hyphen/dash) as referring to either standard
input or standard output (depending on whether the path is read from or written
to).  The `patharg` crate lets your programs follow this convention too: it
provides a `PathArg` type that wraps a command-line argument, with methods for
reading from or writing to either the given path or — if the argument is just a
hyphen — the appropriate standard stream.

`PathArg` implements `From<OsString>` and `From<String>`, so you can use it
seamlessly with your favorite Rust source of command-line arguments, be it
[`clap`][], [`lexopt`][], plain old
[`std::env::args`][args]/[`std::env::args_os`][args_os], or whatever else is
out there.

See [`examples/flipcase.rs`][flipcase] in the source repository for an example
of how to use this crate with `clap`.

[`clap`]: https://crates.io/crates/clap
[`lexopt`]: https://crates.io/crates/lexopt
[args]: https://doc.rust-lang.org/std/env/fn.args.html
[args_os]: https://doc.rust-lang.org/std/env/fn.args_os.html
[flipcase]: https://github.com/jwodder/patharg/blob/master/examples/flipcase.rs
