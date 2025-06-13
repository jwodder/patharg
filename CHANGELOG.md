v0.4.1 (2025-06-13)
-------------------
- Increased MSRV to 1.78
- Moved example programs to workspace crates and removed "examples" feature

v0.4.0 (2023-12-18)
-------------------
- Added `asynchronous` category to crate metadata
- Remove installation instructions from README
- List all features in the library documentation
- Increased MSRV to 1.70
- `InputArg` and `OutputArg` now implement `FromStr`

v0.3.0 (2023-04-22)
-------------------
- Formatting `InputArg::Stdin` or `OutputArg::Stdout` with `{:#}` will now
  produce `"<stdin>"` or `"<stdout>"`, respectively
- Adjusted the trait bounds on the `from_arg()` methods and the `From`
  implementations.  Pre-existing code that uses these features will still work,
  but now some more types are accepted, and there should be no more needless
  copying of data.
- Added "serde" feature to enabling serializing & deserializing with `serde`

v0.2.0 (2023-04-21)
-------------------
- Added "tokio" feature to enable async methods

v0.1.0 (2023-04-19)
-------------------
Initial release
