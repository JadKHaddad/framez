use core::borrow::BorrowMut;

use embedded_io_async::{Read, Write};
use futures::{Sink, Stream};

use crate::{
    ReadError, WriteError,
    decode::{Decoder, OwnedDecoder},
    encode::Encoder,
    logging::{debug, error, trace, warn},
    read::ReadState,
    write::WriteState,
};

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
use crate::logging::Formatter;

#[derive(Debug)]
pub struct FramedImpl<C, RW, S> {
    codec: C,
    read_write: RW,
    state: S,
}

impl<C, RW, S> FramedImpl<C, RW, S> {
    pub fn new(codec: C, read_write: RW, state: S) -> Self {
        Self {
            codec,
            read_write,
            state,
        }
    }

    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'this>,
        RW: Read,
        S: BorrowMut<ReadState<'this>>,
    {
        let state: &mut ReadState = self.state.borrow_mut();

        debug!(
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

            debug!("Buffer shifted. copied: {}", state.framable());

            state.shift = false;

            return Some(Ok(None));
        }

        if state.is_framable {
            if state.eof {
                trace!("Framing on EOF");

                match self
                    .codec
                    .decode_eof(&mut state.buffer[state.total_consumed..state.index])
                {
                    Ok(Some((item, size))) => {
                        state.total_consumed += size;

                        debug!(
                            "Frame decoded, consumed: {}, total_consumed: {}",
                            size, state.total_consumed,
                        );

                        return Some(Ok(Some(item)));
                    }
                    Ok(None) => {
                        debug!("No frame decoded");

                        state.is_framable = false;

                        if state.index != state.total_consumed {
                            error!("Bytes remaining on stream");

                            return Some(Err(ReadError::BytesRemainingOnStream));
                        }

                        return None;
                    }
                    Err(err) => {
                        error!("Failed to decode frame");

                        return Some(Err(ReadError::Decode(err)));
                    }
                };
            }

            trace!("Framing");

            #[cfg(not(feature = "buffer-early-shift"))]
            let buf_len = state.buffer.len();

            match self
                .codec
                .decode(&mut state.buffer[state.total_consumed..state.index])
            {
                Ok(Some((item, size))) => {
                    state.total_consumed += size;

                    debug!(
                        "Frame decoded, consumed: {}, total_consumed: {}",
                        size, state.total_consumed,
                    );

                    return Some(Ok(Some(item)));
                }
                Ok(None) => {
                    debug!("No frame decoded");

                    #[cfg(feature = "buffer-early-shift")]
                    {
                        state.shift = state.total_consumed > 0;
                    }

                    #[cfg(not(feature = "buffer-early-shift"))]
                    {
                        state.shift = state.index >= buf_len;
                    }

                    state.is_framable = false;

                    return Some(Ok(None));
                }
                Err(err) => {
                    error!("Failed to decode frame");

                    return Some(Err(ReadError::Decode(err)));
                }
            }
        }

        if state.index >= state.buffer.len() {
            error!("Buffer too small");

            return Some(Err(ReadError::BufferTooSmall));
        }

        trace!("Reading");

        match self.read_write.read(&mut state.buffer[state.index..]).await {
            Err(err) => {
                error!("Failed to read");

                Some(Err(ReadError::IO(err)))
            }
            Ok(0) => {
                warn!("Got EOF");

                state.eof = true;

                state.is_framable = true;

                Some(Ok(None))
            }
            Ok(n) => {
                debug!("Bytes read. bytes: {}", n);

                state.index += n;

                state.is_framable = true;

                Some(Ok(None))
            }
        }
    }

    pub fn stream<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<RW::Error, C::Error>>> + '_
    where
        C: for<'a> Decoder<'a>,
        RW: Read,
        S: for<'a> BorrowMut<ReadState<'a>>,
        U: 'static,
    {
        futures::stream::unfold((self, false), move |(this, errored)| async move {
            if errored {
                return None;
            }

            let item = crate::next!(this).map(|res| res.map(map));

            match item {
                Some(Ok(item)) => Some((Ok(item), (this, false))),
                Some(Err(err)) => Some((Err(err), (this, true))),
                None => None,
            }
        })
    }

    pub async fn next_owned(&mut self) -> Option<Result<C::Item, ReadError<RW::Error, C::Error>>>
    where
        C: OwnedDecoder,
        RW: Read,
        S: for<'a> BorrowMut<ReadState<'a>>,
    {
        loop {
            let state: &mut ReadState = self.state.borrow_mut();

            debug!(
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

                debug!("Buffer shifted. copied: {}", state.framable());

                state.shift = false;

                continue;
            }

            if state.is_framable {
                if state.eof {
                    trace!("Framing on EOF");

                    match self
                        .codec
                        .decode_eof_owned(&mut state.buffer[state.total_consumed..state.index])
                    {
                        Ok(Some((item, size))) => {
                            state.total_consumed += size;

                            debug!(
                                "Frame decoded, consumed: {}, total_consumed: {}",
                                size, state.total_consumed,
                            );

                            return Some(Ok(item));
                        }
                        Ok(None) => {
                            debug!("No frame decoded");

                            state.is_framable = false;

                            if state.index != state.total_consumed {
                                error!("Bytes remaining on stream");

                                return Some(Err(ReadError::BytesRemainingOnStream));
                            }

                            return None;
                        }
                        Err(err) => {
                            error!("Failed to decode frame");

                            return Some(Err(ReadError::Decode(err)));
                        }
                    };
                }

                trace!("Framing");

                #[cfg(not(feature = "buffer-early-shift"))]
                let buf_len = state.buffer.len();

                match self
                    .codec
                    .decode_owned(&mut state.buffer[state.total_consumed..state.index])
                {
                    Ok(Some((item, size))) => {
                        state.total_consumed += size;

                        debug!(
                            "Frame decoded, consumed: {}, total_consumed: {}",
                            size, state.total_consumed,
                        );

                        return Some(Ok(item));
                    }
                    Ok(None) => {
                        debug!("No frame decoded");
                        #[cfg(feature = "buffer-early-shift")]
                        {
                            state.shift = state.total_consumed > 0;
                        }

                        #[cfg(not(feature = "buffer-early-shift"))]
                        {
                            state.shift = state.index >= buf_len;
                        }

                        state.is_framable = false;

                        continue;
                    }
                    Err(err) => {
                        error!("Failed to decode frame");

                        return Some(Err(ReadError::Decode(err)));
                    }
                }
            }
            if state.index >= state.buffer.len() {
                error!("Buffer too small");

                return Some(Err(ReadError::BufferTooSmall));
            }

            trace!("Reading");

            match self.read_write.read(&mut state.buffer[state.index..]).await {
                Err(err) => {
                    error!("Failed to read");

                    return Some(Err(ReadError::IO(err)));
                }
                Ok(0) => {
                    warn!("Got EOF");

                    state.eof = true;

                    state.is_framable = true;

                    continue;
                }
                Ok(n) => {
                    debug!("Bytes read. bytes: {}", n);

                    state.index += n;

                    state.is_framable = true;

                    continue;
                }
            }
        }
    }

    pub fn stream_owned(
        &mut self,
    ) -> impl Stream<Item = Result<C::Item, ReadError<RW::Error, C::Error>>> + '_
    where
        C: OwnedDecoder,
        RW: Read,
        S: for<'a> BorrowMut<ReadState<'a>>,
    {
        futures::stream::unfold((self, false), |(this, errored)| async move {
            if errored {
                return None;
            }

            match this.next_owned().await {
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
        S: for<'a> BorrowMut<WriteState<'a>>,
    {
        let state: &mut WriteState = self.state.borrow_mut();

        match self.codec.encode(item, state.buffer) {
            Ok(size) => match self.read_write.write_all(&state.buffer[..size]).await {
                Ok(_) => {
                    debug!("Wrote. buffer: {:?}", Formatter(&state.buffer[..size]));

                    match self.read_write.flush().await {
                        Ok(_) => {
                            trace!("Flushed");

                            Ok(())
                        }
                        Err(err) => {
                            error!("Failed to flush");

                            Err(WriteError::IO(err))
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to write frame");

                    Err(WriteError::IO(err))
                }
            },
            Err(err) => {
                error!("Failed to encode frame");

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
        S: for<'a> BorrowMut<WriteState<'a>>,
    {
        futures::sink::unfold(self, |this, item: I| async move {
            this.send(item).await?;

            Ok::<_, WriteError<RW::Error, C::Error>>(this)
        })
    }
}
