/// An error that can occur while reading a frame.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadError<I, D> {
    /// An IO error occurred while reading from the underlying source.
    IO(I),
    /// An error occurred while decoding a frame.
    Decode(D),
    /// The buffer is too small to read a frame.
    BufferTooSmall,
    /// There are bytes remaining on the stream after decoding.
    BytesRemainingOnStream,
}

impl<I, D> core::fmt::Display for ReadError<I, D>
where
    I: core::fmt::Display,
    D: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "Buffer too small"),
            Self::IO(err) => write!(f, "IO error: {err}"),
            Self::BytesRemainingOnStream => write!(f, "Bytes remaining on stream"),
            Self::Decode(err) => write!(f, "Decode error: {err}"),
        }
    }
}

impl<I, D> core::error::Error for ReadError<I, D>
where
    I: core::fmt::Display + core::fmt::Debug,
    D: core::fmt::Display + core::fmt::Debug,
{
}
