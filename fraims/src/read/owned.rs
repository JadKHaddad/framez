use embedded_io_async::Read;
use futures::Stream;

use crate::{
    ReadError,
    decode::OwnedDecoder,
    logging::{debug, error, trace, warn},
};

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
use crate::logging::Formatter;

use super::ReadState;

/// A framer that reads bytes from a [`Read`] source and decodes them into frames using a [`OwnedDecoder`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedReadOwned<'buf, D, R> {
    state: ReadState<'buf>,
    decoder: D,
    reader: R,
}

impl<'buf, D, R> FramedReadOwned<'buf, D, R> {
    /// Creates a new [`FramedReadOwned`] with the given `decoder` and `reader`.
    #[inline]
    pub fn new(decoder: D, reader: R, buffer: &'buf mut [u8]) -> Self {
        Self {
            state: ReadState::new(buffer),
            decoder,
            reader,
        }
    }

    /// Returns reference to the decoder.
    #[inline]
    pub const fn decoder(&self) -> &D {
        &self.decoder
    }

    /// Returns mutable reference to the decoder.
    #[inline]
    pub fn decoder_mut(&mut self) -> &mut D {
        &mut self.decoder
    }

    /// Returns reference to the reader.
    #[inline]
    pub const fn reader(&self) -> &R {
        &self.reader
    }

    /// Returns mutable reference to the reader.
    #[inline]
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the [`FramedReadOwned`] and returns the `decoder` and `reader`.
    #[inline]
    pub fn into_parts(self) -> (D, R) {
        (self.decoder, self.reader)
    }

    /// Tries to read a frame from the underlying reader.
    ///
    /// # Return value
    ///
    /// - `Some(Ok(frame))` if a frame was successfully decoded. Call `next` again to read more frames.
    /// - `Some(Err(error))` if an error occurred. The caller should stop reading.
    /// - `None` if eof was reached. The caller should stop reading.
    pub async fn next(&mut self) -> Option<Result<D::Item, ReadError<R::Error, D::Error>>>
    where
        D: OwnedDecoder,
        R: Read,
    {
        loop {
            debug!(
                "total_consumed: {}, index: {}, buffer: {:?}",
                self.state.total_consumed,
                self.state.index,
                Formatter(&self.state.buffer[self.state.total_consumed..self.state.index])
            );

            if self.state.shift {
                self.state
                    .buffer
                    .copy_within(self.state.total_consumed..self.state.index, 0);

                self.state.index -= self.state.total_consumed;
                self.state.total_consumed = 0;

                debug!("Buffer shifted. copied: {}", self.state.framable());

                self.state.shift = false;

                continue;
            }

            if self.state.is_framable {
                if self.state.eof {
                    trace!("Framing on EOF");

                    match self.decoder.decode_eof_owned(
                        &mut self.state.buffer[self.state.total_consumed..self.state.index],
                    ) {
                        Ok(Some((item, size))) => {
                            self.state.total_consumed += size;

                            debug!(
                                "Frame decoded, consumed: {}, total_consumed: {}",
                                size, self.state.total_consumed,
                            );

                            return Some(Ok(item));
                        }
                        Ok(None) => {
                            debug!("No frame decoded");

                            self.state.is_framable = false;

                            if self.state.index != self.state.total_consumed {
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
                let buf_len = self.state.buffer.len();

                match self.decoder.decode_owned(
                    &mut self.state.buffer[self.state.total_consumed..self.state.index],
                ) {
                    Ok(Some((item, size))) => {
                        self.state.total_consumed += size;

                        debug!(
                            "Frame decoded, consumed: {}, total_consumed: {}",
                            size, self.state.total_consumed,
                        );

                        return Some(Ok(item));
                    }
                    Ok(None) => {
                        debug!("No frame decoded");
                        #[cfg(feature = "buffer-early-shift")]
                        {
                            self.state.shift = self.state.total_consumed > 0;
                        }

                        #[cfg(not(feature = "buffer-early-shift"))]
                        {
                            self.state.shift = self.state.index >= buf_len;
                        }

                        self.state.is_framable = false;

                        continue;
                    }
                    Err(err) => {
                        error!("Failed to decode frame");

                        return Some(Err(ReadError::Decode(err)));
                    }
                }
            }
            if self.state.index >= self.state.buffer.len() {
                error!("Buffer too small");

                return Some(Err(ReadError::BufferTooSmall));
            }

            trace!("Reading");

            match self
                .reader
                .read(&mut self.state.buffer[self.state.index..])
                .await
            {
                Err(err) => {
                    error!("Failed to read");

                    return Some(Err(ReadError::IO(err)));
                }
                Ok(0) => {
                    warn!("Got EOF");

                    self.state.eof = true;

                    self.state.is_framable = true;

                    continue;
                }
                Ok(n) => {
                    debug!("Bytes read. bytes: {}", n);

                    self.state.index += n;

                    self.state.is_framable = true;

                    continue;
                }
            }
        }
    }

    /// Converts the [`FramedReadOwned`] into a stream of frames.
    pub fn stream(
        &mut self,
    ) -> impl Stream<Item = Result<D::Item, ReadError<R::Error, D::Error>>> + '_
    where
        D: OwnedDecoder,
        R: Read,
    {
        futures::stream::unfold((self, false), |(this, errored)| async move {
            if errored {
                return None;
            }

            match this.next().await {
                Some(Ok(item)) => Some((Ok(item), (this, false))),
                Some(Err(err)) => Some((Err(err), (this, true))),
                None => None,
            }
        })
    }
}
