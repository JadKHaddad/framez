//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run decode
//! ```

#![no_main]

use fraims::{
    codec::{
        bytes::{Bytes, OwnedBytes},
        delimiter::{Delimiter, OwnedDelimiter},
        lines::{Lines, OwnedLines, StrLines, StringLines},
    },
    decode::{Decoder, OwnedDecoder},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let buf = &mut [0_u8; 64];
    let data = &mut std::vec::Vec::from(data);

    let mut codec = Delimiter::new(b"#");
    let _ = codec.decode(data).expect("Must be Infallible");

    let mut codec = OwnedDelimiter::<64>::new(b"#");
    let _ = codec.decode_owned(buf);

    let mut codec = Bytes::new();
    let _ = codec.decode(buf).expect("Must be Infallible");

    let mut codec = OwnedBytes::<64>::new();
    let _ = codec.decode_owned(buf);

    let mut codec = Lines::new();
    let _ = codec.decode(buf).expect("Must be Infallible");

    let mut codec = OwnedLines::<64>::new();
    let _ = codec.decode_owned(buf);

    let mut codec = StrLines::new();

    match core::str::from_utf8(data) {
        Ok(_) => {
            let _ = codec.decode(buf).expect("Must be Infallible");
        }
        Err(_) => {
            let _ = codec.decode(buf);
        }
    }

    let mut codec = StringLines::<64>::new();
    let _ = codec.decode_owned(buf);
});
