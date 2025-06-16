//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run encode
//! ```

#![no_main]

use framez::{
    codec::{
        bytes::Bytes,
        delimiter::Delimiter,
        lines::{Lines, StrLines},
    },
    encode::Encoder,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let buf = &mut [0_u8; 64];

    let mut codec = Delimiter::new(b"#");
    let _ = codec.encode(data, buf);

    let mut codec = Bytes::new();
    let _ = codec.encode(data, buf);

    let mut codec = Lines::new();
    let _ = codec.encode(data, buf);

    let mut codec = StrLines::new();
    if let Ok(str) = str::from_utf8(data) {
        let _ = codec.encode(str, buf);
    }
});
