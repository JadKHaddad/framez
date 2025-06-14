//! # framez
//!
//! A `zerocopy` codec for encoding and decoding data in `no_std` environments.
//!
//! This crate is based on [`embedded_io_async`](https://docs.rs/embedded-io-async/latest/embedded_io_async/)'s
//! [`Read`](https://docs.rs/embedded-io-async/latest/embedded_io_async/trait.Read.html) and [`Write`](https://docs.rs/embedded-io-async/latest/embedded_io_async/trait.Write.html) traits.
//!
//! It's recommended to use [`embedded_io_adapters`](https://docs.rs/embedded-io-adapters/0.6.1/embedded_io_adapters/) if you are using other async `Read` and `Write` traits like [`tokio`](https://docs.rs/tokio/latest/tokio/index.html)'s [`AsyncRead`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html) and [`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html).
//!
//! See the examples for more information.
//!
//! ## Features
//!
//! - `log`: Enables logging using [`log`](https://docs.rs/log/latest/log/).
//! - `tracing`: Enables logging using [`tracing`](https://docs.rs/tracing/latest/tracing/).
//! - `defmt`: Enables logging using [`defmt`](https://docs.rs/defmt/latest/defmt/index.html)
//!   and implements [`defmt::Format`](https://docs.rs/defmt/latest/defmt/trait.Format.html) for structs and enums.
//! - `buffer-early-shift`: Moves the bytes in the encode buffer to the start of the buffer after the first successful decoded frame.

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] # TODO
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codec;
pub mod decode;
pub mod encode;

mod framed;
pub use framed::{Framed, FramedRead, FramedWrite};

mod framed_core;
use framed_core::FramedCore;

mod error;
pub use error::{ReadError, WriteError};

pub mod state;

pub(crate) mod logging;

mod next;

#[doc(hidden)]
pub mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate std;
