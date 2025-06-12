//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run encode
//! ```

#![no_main]

use framez::{
    codec::{
        bytes::{Bytes, OwnedBytes},
        delimiter::{Delimiter, OwnedDelimiter},
        lines::{Lines, OwnedLines, StrLines, StringLines},
    },
    encode::Encoder,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let buf = &mut [0_u8; 64];

    let mut codec = Delimiter::new(b"#");
    let _ = codec.encode(data, buf);

    let mut codec = OwnedDelimiter::<64>::new(b"#");
    if let Ok(vector) = heapless::Vec::<u8, 64>::from_slice(data) {
        let _ = codec.encode(vector, buf);
    }

    let mut codec = Bytes::new();
    let _ = codec.encode(data, buf);

    let mut codec = OwnedBytes::<64>::new();
    if let Ok(vector) = heapless::Vec::<u8, 64>::from_slice(data) {
        let _ = codec.encode(vector, buf);
    }

    let mut codec = Lines::new();
    let _ = codec.encode(data, buf);

    let mut codec = OwnedLines::<64>::new();
    if let Ok(vector) = heapless::Vec::<u8, 64>::from_slice(data) {
        let _ = codec.encode(vector, buf);
    }

    let mut codec = StrLines::new();
    if let Ok(str) = str::from_utf8(data) {
        let _ = codec.encode(str, buf);
    }

    let mut codec = StringLines::<64>::new();
    if let Ok(vector) = heapless::Vec::<u8, 64>::from_slice(data) {
        if let Ok(string) = heapless::String::<64>::from_utf8(vector) {
            let _ = codec.encode(string, buf);
        }
    }
});
