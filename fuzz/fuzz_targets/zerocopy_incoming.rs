//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run zerocopy_incoming
//! ```

#![no_main]

use libfuzzer_sys::fuzz_target;

// TODO: test decoding arbitrary data from the buffer.
// 3 Step decoding to test the framing logic: each step adds a 1/3 of the data to the buffer
// Decode the thirds in a loop. If we get an error break, if we get a None, continue.
fuzz_target!(|data: &[u8]| {});
