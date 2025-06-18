use core::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
};

use embedded_io_async::{Read, Write};
use futures::{Sink, Stream};

use crate::{
    ReadError, WriteError,
    decode::Decoder,
    encode::Encoder,
    logging::{debug, error, trace, warn},
    state::{ReadState, WriteState},
};

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
use crate::logging::Formatter;

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
const READ: &str = "framez::read";

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
const WRITE: &str = "framez::write";

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedCore<'this, C, RW, S> {
    codec: C,
    read_write: RW,
    state: S,
    buf: PhantomData<&'this ()>,
}

impl<'buf, C, RW, S> FramedCore<'buf, C, RW, S> {
    pub const fn new(codec: C, read_write: RW, state: S) -> Self {
        Self {
            codec,
            read_write,
            state,
            buf: PhantomData,
        }
    }

    /// Returns reference to the codec.
    #[inline]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns mutable reference to the codec.
    #[inline]
    pub const fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Returns reference to the reader/writer.
    #[inline]
    pub const fn inner(&self) -> &RW {
        &self.read_write
    }

    /// Returns mutable reference to the reader/writer.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut RW {
        &mut self.read_write
    }

    /// Consumes the [`FramedCore`] and returns the `codec` and `reader/writer` and state.
    #[inline]
    pub fn into_parts(self) -> (C, RW, S) {
        (self.codec, self.read_write, self.state)
    }

    #[inline]
    /// Creates a new [`FramedCore`] from its parts.
    pub const fn from_parts(codec: C, read_write: RW, state: S) -> Self {
        Self {
            codec,
            read_write,
            state,
            buf: PhantomData,
        }
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    pub fn framable(&self) -> usize
    where
        S: Borrow<ReadState<'buf>>,
    {
        self.state.borrow().framable()
    }

    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'this>,
        RW: Read,
        S: BorrowMut<ReadState<'buf>>,
    {
        let state: &mut ReadState = self.state.borrow_mut();

        trace!(target: READ, "maybe_next called");

        debug!(
            target: READ,
            "total_consumed: {}, index: {}, buffer: {:?}",
            state.total_consumed,
            state.index,
            Formatter(&state.buffer[state.total_consumed..state.index])
        );

        if state.shift {
            state
                .buffer
                .copy_within(state.total_consumed..state.index, 0);

            state.index -= state.total_consumed;
            state.total_consumed = 0;

            trace!(target: READ, "Buffer shifted. copied: {}", state.framable());

            state.shift = false;

            return Some(Ok(None));
        }

        if state.is_framable {
            if state.eof {
                trace!(target: READ, "Framing on EOF");

                match self
                    .codec
                    .decode_eof(&mut state.buffer[state.total_consumed..state.index])
                {
                    Ok(Some((item, size))) => {
                        state.total_consumed += size;

                        debug!(
                            target: READ,
                            "Frame decoded, consumed: {}, total_consumed: {}",
                            size, state.total_consumed,
                        );

                        return Some(Ok(Some(item)));
                    }
                    Ok(None) => {
                        debug!(target: READ, "No frame decoded");

                        state.is_framable = false;

                        if state.index != state.total_consumed {
                            error!(target: READ, "Bytes remaining on stream");

                            return Some(Err(ReadError::BytesRemainingOnStream));
                        }

                        return None;
                    }
                    Err(err) => {
                        error!(target: READ, "Failed to decode frame");

                        return Some(Err(ReadError::Decode(err)));
                    }
                };
            }

            trace!(target: READ, "Framing");

            let buf_len = state.buffer.len();

            match self
                .codec
                .decode(&mut state.buffer[state.total_consumed..state.index])
            {
                Ok(Some((item, size))) => {
                    state.total_consumed += size;

                    debug!(
                        target: READ,
                        "Frame decoded, consumed: {}, total_consumed: {}",
                        size, state.total_consumed,
                    );

                    return Some(Ok(Some(item)));
                }
                Ok(None) => {
                    debug!(target: READ, "No frame decoded");

                    state.shift = state.index >= buf_len;

                    state.is_framable = false;

                    return Some(Ok(None));
                }
                Err(err) => {
                    error!(target: READ, "Failed to decode frame");

                    return Some(Err(ReadError::Decode(err)));
                }
            }
        }

        if state.index >= state.buffer.len() {
            error!(target: READ, "Buffer too small");

            return Some(Err(ReadError::BufferTooSmall));
        }

        trace!(target: READ, "Reading");

        match self.read_write.read(&mut state.buffer[state.index..]).await {
            Err(err) => {
                error!(target: READ, "Failed to read");

                Some(Err(ReadError::IO(err)))
            }
            Ok(0) => {
                warn!(target: READ, "Got EOF");

                state.eof = true;

                state.is_framable = true;

                Some(Ok(None))
            }
            Ok(n) => {
                debug!(target: READ, "Bytes read. bytes: {}", n);

                state.index += n;

                state.is_framable = true;

                Some(Ok(None))
            }
        }
    }

    async fn maybe_next_mapped<'this, U>(
        &'this mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> Option<Result<Option<U>, ReadError<RW::Error, C::Error>>>
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
        S: BorrowMut<ReadState<'buf>>,
    {
        match self.maybe_next().await {
            Some(Ok(Some(item))) => Some(Ok(Some(map(item)))),
            Some(Ok(None)) => Some(Ok(None)),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }

    pub async fn next<'this, U>(
        &'this mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> Option<Result<U, ReadError<RW::Error, C::Error>>>
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
        S: BorrowMut<ReadState<'buf>>,
    {
        loop {
            match self.maybe_next_mapped(map).await {
                Some(Ok(None)) => continue,
                Some(Ok(Some(item))) => return Some(Ok(item)),
                Some(Err(err)) => return Some(Err(err)),
                None => return None,
            }
        }
    }

    pub fn stream<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<RW::Error, C::Error>>> + '_
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
        S: BorrowMut<ReadState<'buf>>,
    {
        futures::stream::unfold((self, false), move |(this, errored)| async move {
            if errored {
                return None;
            }

            match this.next(map).await {
                Some(Ok(item)) => Some((Ok(item), (this, false))),
                Some(Err(err)) => Some((Err(err), (this, true))),
                None => None,
            }
        })
    }

    pub async fn send<I>(&mut self, item: I) -> Result<(), WriteError<RW::Error, C::Error>>
    where
        C: Encoder<I>,
        RW: Write,
        S: BorrowMut<WriteState<'buf>>,
    {
        let state: &mut WriteState = self.state.borrow_mut();

        match self.codec.encode(item, state.buffer) {
            Ok(size) => match self.read_write.write_all(&state.buffer[..size]).await {
                Ok(_) => {
                    trace!(target: WRITE, "Wrote. buffer: {:?}", Formatter(&state.buffer[..size]));

                    match self.read_write.flush().await {
                        Ok(_) => {
                            debug!(target: WRITE, "Flushed. bytes: {}", size);

                            Ok(())
                        }
                        Err(err) => {
                            error!(target: WRITE, "Failed to flush");

                            Err(WriteError::IO(err))
                        }
                    }
                }
                Err(err) => {
                    error!(target: WRITE, "Failed to write frame");

                    Err(WriteError::IO(err))
                }
            },
            Err(err) => {
                error!(target: WRITE, "Failed to encode frame");

                Err(WriteError::Encode(err))
            }
        }
    }

    pub fn sink<'this, I>(
        &'this mut self,
    ) -> impl Sink<I, Error = WriteError<RW::Error, C::Error>> + 'this
    where
        I: 'this,
        C: Encoder<I>,
        RW: Write,
        S: BorrowMut<WriteState<'buf>>,
    {
        futures::sink::unfold(self, |this, item: I| async move {
            this.send(item).await?;

            Ok::<_, WriteError<RW::Error, C::Error>>(this)
        })
    }
}
