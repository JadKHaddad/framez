# framez

![Build Status](https://github.com/JadKHaddad/framez/actions/workflows/build-and-test.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/framez.svg)](https://crates.io/crates/framez)
[![Crates.io (MSRV)](https://img.shields.io/crates/msrv/framez)](https://crates.io/crates/framez)
[![docs.rs](https://docs.rs/framez/badge.svg)](https://docs.rs/framez)
[![Crates.io (Downloads)](https://img.shields.io/crates/d/framez)](https://crates.io/crates/framez)
[![Crates.io (License)](https://img.shields.io/crates/l/framez)](https://crates.io/crates/framez)

A `zerocopy` codec for encoding and decoding data in `no_std` environments.

This crate is based on [`embedded_io_async`](https://docs.rs/embedded-io-async/latest/embedded_io_async/)'s
[`Read`](https://docs.rs/embedded-io-async/latest/embedded_io_async/trait.Read.html) and [`Write`](https://docs.rs/embedded-io-async/latest/embedded_io_async/trait.Write.html) traits.

It's recommended to use [`embedded_io_adapters`](https://docs.rs/embedded-io-adapters/0.6.1/embedded_io_adapters/) if you are using other async `Read` and `Write` traits like [`tokio`](https://docs.rs/tokio/latest/tokio/index.html)'s [`AsyncRead`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html) and [`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html).

See the examples for more information.

## Features

- `log`: Enables logging using [`log`](https://docs.rs/log/latest/log/).
- `tracing`: Enables logging using [`tracing`](https://docs.rs/tracing/latest/tracing/).
- `defmt`: Enables logging using [`defmt`](https://docs.rs/defmt/latest/defmt/index.html)
and implements [`defmt::Format`](https://docs.rs/defmt/latest/defmt/trait.Format.html) for structs and enums.

## License

Licensed under either of

- Apache License, Version 2.0. [LICENSE-APACHE](LICENSE-APACHE) or [Apache-2.0 license](http://apache.org/licenses/LICENSE-2.0)
- MIT license. [LICENSE-MIT](LICENSE-MIT) or [MIT license](http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
