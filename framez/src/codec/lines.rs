//! Lines codecs for encoding and decoding lines.

use core::{convert::Infallible, str::FromStr};

use heapless::{String, Vec};

use crate::{
    decode::{DecodeError, Decoder, OwnedDecoder},
    encode::Encoder,
};

/// A codec that decodes `bytes` into a `line of bytes` and encodes a `line of bytes` into `bytes`.
///
/// # Note
///
/// This codec tracks progress using an internal state of the underlying buffer, and it must not be used across multiple framing sessions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Lines {
    /// The number of bytes of the slice that have been seen so far.
    seen: usize,
}

impl Lines {
    /// Creates a new [`Lines`].
    #[inline]
    pub const fn new() -> Self {
        Self { seen: 0 }
    }
}

impl DecodeError for Lines {
    type Error = Infallible;
}

impl<'buf> Decoder<'buf> for Lines {
    type Item = &'buf [u8];

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        while self.seen < src.len() {
            if src[self.seen] == b'\n' {
                let line_bytes = match &src[..self.seen].last() {
                    Some(b'\r') => &src[..self.seen - 1],
                    _ => &src[..self.seen],
                };

                let item = (line_bytes, self.seen + 1);

                self.seen = 0;

                return Ok(Some(item));
            }

            self.seen += 1;
        }

        Ok(None)
    }
}

/// Error returned by [`Lines::encode`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinesEncodeError {
    /// The input buffer is too small to fit the encoded line.
    BufferTooSmall,
}

impl core::fmt::Display for LinesEncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for LinesEncodeError {}

impl Encoder<&[u8]> for Lines {
    type Error = LinesEncodeError;

    fn encode(&mut self, item: &[u8], dst: &mut [u8]) -> Result<usize, Self::Error> {
        let size = item.len() + 2;

        if dst.len() < size {
            return Err(LinesEncodeError::BufferTooSmall);
        }

        dst[..item.len()].copy_from_slice(item);
        dst[item.len()..size].copy_from_slice(b"\r\n");

        Ok(size)
    }
}

/// A codec that decodes `bytes` into an [`str`] line and encodes an [`str`] line into `bytes`.
///
/// # Note
///
/// This codec tracks progress using an internal state of the underlying buffer, and it must not be used across multiple framing sessions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StrLines {
    inner: Lines,
}

impl StrLines {
    /// Creates a new [`StrLines`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: Lines::new(),
        }
    }
}

impl From<Lines> for StrLines {
    fn from(inner: Lines) -> Self {
        Self { inner }
    }
}

/// Error returned by [`StrLines::decode`].
#[derive(Debug)]
pub enum StrLinesDecodeError {
    /// utf8 error.
    Utf8(core::str::Utf8Error),
}

#[cfg(feature = "defmt")]
impl defmt::Format for StrLinesDecodeError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            StrLinesDecodeError::Utf8(_) => defmt::write!(fmt, "utf8 error"),
        }
    }
}

impl core::fmt::Display for StrLinesDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StrLinesDecodeError::Utf8(err) => write!(f, "utf8 error: {err}"),
        }
    }
}

impl core::error::Error for StrLinesDecodeError {}

impl DecodeError for StrLines {
    type Error = StrLinesDecodeError;
}

impl<'buf> Decoder<'buf> for StrLines {
    type Item = &'buf str;

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        match Decoder::decode(&mut self.inner, src) {
            Ok(Some((bytes, size))) => {
                let item = core::str::from_utf8(bytes).map_err(StrLinesDecodeError::Utf8)?;

                Ok(Some((item, size)))
            }
            Ok(None) => Ok(None),
            Err(_) => unreachable!(),
        }
    }
}

impl<'a> Encoder<&'a str> for StrLines {
    type Error = LinesEncodeError;

    fn encode(&mut self, item: &'a str, dst: &mut [u8]) -> Result<usize, Self::Error> {
        Encoder::encode(&mut self.inner, item.as_bytes(), dst)
    }
}

/// An owned [`Lines`].
///
/// # Note
///
/// This codec tracks progress using an internal state of the underlying buffer, and it must not be used across multiple framing sessions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OwnedLines<const N: usize> {
    inner: Lines,
}

impl<const N: usize> OwnedLines<N> {
    /// Creates a new [`OwnedLines`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: Lines::new(),
        }
    }
}

impl<const N: usize> From<Lines> for OwnedLines<N> {
    fn from(inner: Lines) -> Self {
        Self { inner }
    }
}

/// Error returned by [`OwnedLines::decode_owned`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OwnedLinesDecodeError {
    /// The buffer is too small to fit the decoded bytes.
    BufferTooSmall,
}

impl core::fmt::Display for OwnedLinesDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OwnedLinesDecodeError::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for OwnedLinesDecodeError {}

impl<const N: usize> OwnedDecoder for OwnedLines<N> {
    type Item = Vec<u8, N>;
    type Error = OwnedLinesDecodeError;

    fn decode_owned(&mut self, src: &mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        match Decoder::decode(&mut self.inner, src) {
            Ok(Some((bytes, size))) => {
                let item =
                    Vec::from_slice(bytes).map_err(|_| OwnedLinesDecodeError::BufferTooSmall)?;
                Ok(Some((item, size)))
            }
            Ok(None) => Ok(None),
            Err(_) => unreachable!(),
        }
    }
}

impl<const N: usize> Encoder<Vec<u8, N>> for OwnedLines<N> {
    type Error = LinesEncodeError;

    fn encode(&mut self, item: Vec<u8, N>, dst: &mut [u8]) -> Result<usize, Self::Error> {
        Encoder::encode(&mut self.inner, &item, dst)
    }
}

/// An owned [`StrLines`].
///
/// # Note
///
/// This codec tracks progress using an internal state of the underlying buffer, and it must not be used across multiple framing sessions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StringLines<const N: usize> {
    inner: StrLines,
}

impl<const N: usize> StringLines<N> {
    /// Creates a new [`StringLines`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: StrLines::new(),
        }
    }
}

impl<const N: usize> From<StrLines> for StringLines<N> {
    fn from(inner: StrLines) -> Self {
        Self { inner }
    }
}

/// Error returned by [`StringLines::decode_owned`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StringLinesDecodeError {
    /// Str decoding error.
    Str(StrLinesDecodeError),
    /// The buffer is too small to fit the decoded bytes.
    BufferTooSmall,
}

impl core::fmt::Display for StringLinesDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StringLinesDecodeError::Str(err) => write!(f, "str error: {err}"),
            StringLinesDecodeError::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for StringLinesDecodeError {}

impl<const N: usize> OwnedDecoder for StringLines<N> {
    type Item = String<N>;
    type Error = StringLinesDecodeError;

    fn decode_owned(&mut self, src: &mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        match Decoder::decode(&mut self.inner, src) {
            Ok(Some((bytes, size))) => {
                let item =
                    String::from_str(bytes).map_err(|_| StringLinesDecodeError::BufferTooSmall)?;
                Ok(Some((item, size)))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(StringLinesDecodeError::Str(err)),
        }
    }
}

impl<const N: usize> Encoder<String<N>> for StringLines<N> {
    type Error = LinesEncodeError;

    fn encode(&mut self, item: String<N>, dst: &mut [u8]) -> Result<usize, Self::Error> {
        Encoder::encode(&mut self.inner, &item, dst)
    }
}

#[cfg(test)]
mod test {
    use std::vec::Vec;

    use futures::{SinkExt, StreamExt, pin_mut};
    use tokio::io::AsyncWriteExt;

    use crate::{
        ReadError,
        tests::{framed_read, init_tracing, sink_stream},
    };

    use super::*;

    #[tokio::test]
    async fn framed_read() {
        init_tracing();

        let items: &[&[u8]] = &[
            b"Hel",
            b"lo\n",
            b"Hell",
            b"o, world!\n",
            b"H",
            b"ei\r\n",
            b"sup",
            b"\n",
            b"Hey\r",
            b"\n",
            b"How ",
            b"are y",
        ];

        let decoder = Lines::new();

        let expected: &[&[u8]] = &[];
        framed_read!(items, expected, decoder, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 4, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 4, BufferTooSmall);

        let expected: &[&[u8]] = &[b"Hello"];
        framed_read!(items, expected, decoder, 8, BufferTooSmall);

        let expected: &[&[u8]] = &[b"Hello", b"Hello, world!", b"Hei", b"sup", b"Hey"];
        framed_read!(items, expected, decoder, 16, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 1, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 2, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 4, BytesRemainingOnStream);

        framed_read!(items, expected, decoder);
    }

    #[tokio::test]
    async fn sink_stream() {
        init_tracing();

        let items: Vec<heapless::Vec<u8, 32>> = std::vec![
            heapless::Vec::from_slice(b"Hello").unwrap(),
            heapless::Vec::from_slice(b"Hello, world!").unwrap(),
            heapless::Vec::from_slice(b"Hei").unwrap(),
            heapless::Vec::from_slice(b"sup").unwrap(),
            heapless::Vec::from_slice(b"Hey").unwrap(),
        ];

        let decoder = OwnedLines::<32>::new();
        let encoder = OwnedLines::<32>::new();

        sink_stream!(encoder, decoder, items);
    }

    #[tokio::test]
    async fn framed_read_str() {
        init_tracing();

        let items: &[&str] = &[
            "Hel",
            "lo\n",
            "Hell",
            "o, world!\n",
            "H",
            "ei\r\n",
            "sup",
            "\n",
            "Hey\r",
            "\n",
            "How ",
            "are y",
        ];

        let decoder = StrLines::new();

        let expected: &[&[u8]] = &[];
        framed_read!(items, expected, decoder, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 4, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 4, BufferTooSmall);

        let expected: &[&[u8]] = &[b"Hello"];
        framed_read!(items, expected, decoder, 8, BufferTooSmall);

        let expected: &[&[u8]] = &[b"Hello", b"Hello, world!", b"Hei", b"sup", b"Hey"];
        framed_read!(items, expected, decoder, 16, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 1, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 2, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 16, 4, BytesRemainingOnStream);

        framed_read!(items, expected, decoder);
    }

    #[tokio::test]
    async fn sink_stream_str() {
        init_tracing();

        let items: Vec<heapless::String<32>> = std::vec![
            heapless::String::from_str("Hello").unwrap(),
            heapless::String::from_str("Hello, world!").unwrap(),
            heapless::String::from_str("Hei").unwrap(),
            heapless::String::from_str("sup").unwrap(),
            heapless::String::from_str("Hey").unwrap(),
        ];

        let decoder = StringLines::<32>::new();
        let encoder = StringLines::<32>::new();

        sink_stream!(encoder, decoder, items);
    }
}
