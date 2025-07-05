//! Internal states for reading and writing frames.

/// Internal state for reading frames.
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
    /// Creates a new [`ReadState`].
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

    /// Resets the state to its initial values.
    #[inline]
    pub const fn reset(self) -> Self {
        Self::new(self.buffer)
    }

    /// Creates an empty [`ReadState`].
    #[inline]
    pub const fn empty() -> Self {
        Self::new(&mut [])
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    pub const fn framable(&self) -> usize {
        self.index - self.total_consumed
    }
}

/// Internal state for writing frames.
#[derive(Debug)]
pub struct WriteState<'buf> {
    /// The underlying buffer to write to.
    pub buffer: &'buf mut [u8],
}

impl<'buf> WriteState<'buf> {
    /// Creates a new [`WriteState`].
    #[inline]
    pub const fn new(buffer: &'buf mut [u8]) -> Self {
        Self { buffer }
    }

    /// Resets the state to its initial values.
    #[inline]
    pub const fn reset(self) -> Self {
        Self::new(self.buffer)
    }

    /// Creates an empty [`WriteState`].
    #[inline]
    pub const fn empty() -> Self {
        Self::new(&mut [])
    }
}

/// Internal state for reading and writing frames.
#[derive(Debug)]
pub struct ReadWriteState<'buf> {
    /// Internal read state.
    pub read: ReadState<'buf>,
    /// Internal write state.
    pub write: WriteState<'buf>,
}

impl<'buf> ReadWriteState<'buf> {
    /// Creates a new [`ReadWriteState`] with the given [`ReadState`] and [`WriteState`].
    #[inline]
    pub const fn new(read: ReadState<'buf>, write: WriteState<'buf>) -> Self {
        Self { read, write }
    }

    /// Creates a new [`ReadWriteState`] with empty [`ReadState`] and [`WriteState`].
    #[inline]
    pub const fn reset(self) -> Self {
        Self::new(self.read.reset(), self.write.reset())
    }
}
