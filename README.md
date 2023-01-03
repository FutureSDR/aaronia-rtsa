Limitations:
- Only Linux at the moment

Usage:
- If you installed to a non-standard location, set the `RTSA_DIR` environment variable to the directory containing the RTSA Suite Pro
- Add the directory of the RTSA Suite Pro to your `LD_LIBRARY_PATH`

The latter is necessary, because Rust does not allow [setting an rpath that is picked up by transitive dependencies](https://github.com/rust-lang/cargo/issues/5077).
