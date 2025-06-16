//! Decoder trait definition.

/// An error that can occur while decoding a frame.
pub trait DecodeError {
    /// The type of error that a decoder returns.
    type Error;
}

impl<D> DecodeError for &mut D
where
    D: DecodeError,
{
    type Error = D::Error;
}

/// A decoder that decodes a frame from a buffer.
pub trait Decoder<'buf>: DecodeError {
    /// The type of item that this decoder decodes.
    type Item;

    /// Decodes a frame from the provided buffer.
    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error>;

    /// Decodes a frame from the provided buffer at the end of the stream.
    fn decode_eof(
        &mut self,
        src: &'buf mut [u8],
    ) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        self.decode(src)
    }
}

impl<'buf, D> Decoder<'buf> for &mut D
where
    D: Decoder<'buf>,
{
    type Item = D::Item;

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        (*self).decode(src)
    }

    fn decode_eof(
        &mut self,
        src: &'buf mut [u8],
    ) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        (*self).decode_eof(src)
    }
}
