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

/// An error that can occur while writing a frame.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WriteError<I, E> {
    /// An IO error occurred while writing to the underlying sink.
    IO(I),
    /// An error occurred while encoding a frame.
    Encode(E),
}

impl<I, E> core::fmt::Display for WriteError<I, E>
where
    I: core::fmt::Display,
    E: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {err}"),
            Self::Encode(err) => write!(f, "Encode error: {err}"),
        }
    }
}

impl<I, E> core::error::Error for WriteError<I, E>
where
    I: core::fmt::Display + core::fmt::Debug,
    E: core::fmt::Display + core::fmt::Debug,
{
}

/// An error that can occur while reading or writing a frame.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadWriteError<I, A, B> {
    /// An error occurred while reading a frame.
    Read(ReadError<I, A>),
    /// An error occurred while writing a frame.
    Write(WriteError<I, B>),
}

impl<I, A, B> core::fmt::Display for ReadWriteError<I, A, B>
where
    I: core::fmt::Display,
    A: core::fmt::Display,
    B: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read(err) => write!(f, "Read error: {err}"),
            Self::Write(err) => write!(f, "Write error: {err}"),
        }
    }
}

impl<I, A, B> core::error::Error for ReadWriteError<I, A, B>
where
    I: core::fmt::Display + core::fmt::Debug,
    A: core::fmt::Display + core::fmt::Debug,
    B: core::fmt::Display + core::fmt::Debug,
{
}
