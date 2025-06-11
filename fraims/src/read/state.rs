/// Internal state for reading a frame.
#[derive(Debug)]
pub struct ReadState<'buf> {
    /// The current index in the buffer.
    ///
    /// Represents the number of bytes read into the buffer.
    pub index: usize,
    /// EOF was reached while decoding.
    pub eof: bool,
    /// The buffer is currently framable.
    pub is_framable: bool,
    /// The buffer must be shifted before reading more bytes.
    ///
    /// Makes room for more bytes to be read into the buffer, keeping the already read bytes.
    pub shift: bool,
    /// Total number of bytes decoded in a framing round.
    pub total_consumed: usize,
    /// The underlying buffer to read into.
    pub buffer: &'buf mut [u8],
}

impl<'buf> ReadState<'buf> {
    #[inline]
    pub const fn new(buffer: &'buf mut [u8]) -> Self {
        Self {
            index: 0,
            eof: false,
            is_framable: false,
            shift: false,
            total_consumed: 0,
            buffer,
        }
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    #[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
    pub const fn framable(&self) -> usize {
        self.index - self.total_consumed
    }
}
