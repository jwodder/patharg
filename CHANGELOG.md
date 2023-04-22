v0.3.0 (in development)
-----------------------
- Formatting `InputArg::Stdin` or `OutputArg::Stdout` with `{:#}` will now
  produce `"<stdin>"` or `"<stdout>"`, respectively
- Adjusted the trait bounds on the `from_arg()` methods and the `From`
  implementations.  Pre-existing code that uses these features will still work,
  but now some more types are accepted, and there should be no more needless
  copying of data.

v0.2.0 (2023-04-21)
-------------------
- Added "tokio" feature to enable async methods

v0.1.0 (2023-04-19)
-------------------
Initial release
