//! Bytes codecs for encoding and decoding bytes.

use core::convert::Infallible;

use crate::{
    decode::{DecodeError, Decoder},
    encode::Encoder,
};

/// A codec that decodes bytes into bytes and encodes bytes into bytes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bytes {}

impl Bytes {
    /// Creates a new [`BytesCodec`].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

impl DecodeError for Bytes {
    type Error = Infallible;
}

impl<'buf> Decoder<'buf> for Bytes {
    type Item = &'buf [u8];

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        Ok(Some((src, src.len())))
    }
}

/// Error returned by [`Bytes::encode`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BytesEncodeError {
    /// The input buffer is too small to fit the bytes.
    BufferTooSmall,
}

impl core::fmt::Display for BytesEncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for BytesEncodeError {}

impl Encoder<&[u8]> for Bytes {
    type Error = BytesEncodeError;

    fn encode(&mut self, item: &[u8], dst: &mut [u8]) -> Result<usize, Self::Error> {
        let size = item.len();

        if dst.len() < size {
            return Err(BytesEncodeError::BufferTooSmall);
        }

        dst[..item.len()].copy_from_slice(item);

        Ok(size)
    }
}
