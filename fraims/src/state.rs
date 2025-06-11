use core::borrow::{Borrow, BorrowMut};

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

/// Internal state for writing a frame.
#[derive(Debug)]
pub struct WriteState<'buf> {
    /// The underlying buffer to write to.
    pub buffer: &'buf mut [u8],
}

impl<'buf> WriteState<'buf> {
    /// Creates a new [`WriteFrame`].
    #[inline]
    pub const fn new(buffer: &'buf mut [u8]) -> Self {
        Self { buffer }
    }
}

#[derive(Debug)]
pub struct ReadWriteState<'buf> {
    read: ReadState<'buf>,
    write: WriteState<'buf>,
}

impl<'buf> ReadWriteState<'buf> {
    pub const fn new(read: ReadState<'buf>, write: WriteState<'buf>) -> Self {
        Self { read, write }
    }
}

impl<'buf> Borrow<ReadState<'buf>> for ReadWriteState<'buf> {
    fn borrow(&self) -> &ReadState<'buf> {
        &self.read
    }
}

impl<'buf> BorrowMut<ReadState<'buf>> for ReadWriteState<'buf> {
    fn borrow_mut(&mut self) -> &mut ReadState<'buf> {
        &mut self.read
    }
}

impl<'buf> Borrow<WriteState<'buf>> for ReadWriteState<'buf> {
    fn borrow(&self) -> &WriteState<'buf> {
        &self.write
    }
}

impl<'buf> BorrowMut<WriteState<'buf>> for ReadWriteState<'buf> {
    fn borrow_mut(&mut self) -> &mut WriteState<'buf> {
        &mut self.write
    }
}
