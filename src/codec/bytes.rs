//! Bytes codecs for encoding and decoding bytes.

use core::convert::Infallible;

use heapless::Vec;

use crate::{
    decode::{DecodeError, Decoder, OwnedDecoder},
    encode::Encoder,
};

/// A codec that decodes bytes into bytes and encodes bytes into bytes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bytes {}

impl Bytes {
    /// Creates a new [`Bytes`].
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

/// An owned [`Bytes`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OwnedBytes<const N: usize> {
    inner: Bytes,
}

impl<const N: usize> OwnedBytes<N> {
    /// Creates a new [`OwnedBytes`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }
}

impl<const N: usize> From<Bytes> for OwnedBytes<N> {
    fn from(inner: Bytes) -> Self {
        Self { inner }
    }
}

/// Error returned by [`OwnedBytes::decode_owned`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OwnedBytesDecodeError {
    /// The buffer is too small to fit the decoded bytes.
    BufferTooSmall,
}

impl core::fmt::Display for OwnedBytesDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OwnedBytesDecodeError::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for OwnedBytesDecodeError {}

impl<const N: usize> OwnedDecoder for OwnedBytes<N> {
    type Item = Vec<u8, N>;
    type Error = OwnedBytesDecodeError;

    fn decode_owned(&mut self, src: &mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        match Decoder::decode(&mut self.inner, src) {
            Ok(Some((bytes, size))) => {
                let item =
                    Vec::from_slice(bytes).map_err(|_| OwnedBytesDecodeError::BufferTooSmall)?;
                Ok(Some((item, size)))
            }
            Ok(None) => Ok(None),
            Err(_) => unreachable!(),
        }
    }
}

impl<const N: usize> Encoder<Vec<u8, N>> for OwnedBytes<N> {
    type Error = BytesEncodeError;

    fn encode(&mut self, item: Vec<u8, N>, dst: &mut [u8]) -> Result<usize, Self::Error> {
        Encoder::encode(&mut self.inner, &item, dst)
    }
}
