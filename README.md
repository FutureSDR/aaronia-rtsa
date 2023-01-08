Limitations:
- Windows binaries have to be executed from the RTSA directory. This is a limitation of the Aaronia Windows SDK.

Usage:
- If you installed to a non-standard location, set the `RTSA_DIR` environment variable to the directory containing the RTSA Suite Pro. The default path on Linux is `~/Aaronia/RTSA/Aaronia-RTSA-Suite-PRO`; the default path on Windows is `C:\Program Files\Aaronia AG\Aaronia RTSA-Suite PRO`.
- On Linux, add the directory of the RTSA Suite Pro to your `LD_LIBRARY_PATH`. This is necessary, because Rust does not allow [setting an rpath that is picked up by transitive dependencies](https://github.com/rust-lang/cargo/issues/5077), i.e., we cannot set the runtime library search path in aaronia-rtsa-sys and have it picked up by all applications that use it as a direct or indirect dependency.

## Todo
- better understand packets and queues, and adapt Packet API accordingly.

