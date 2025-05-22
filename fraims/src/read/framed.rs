use embedded_io_async::Read;
use futures::Stream;

use crate::{
    ReadError,
    decode::Decoder,
    logging::{debug, error, trace, warn},
};

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
use crate::logging::Formatter;

use super::State;

/// A framer that reads bytes from a [`Read`] source and decodes them into frames using a [`Decoder`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedRead<'buf, D, R> {
    state: State<'buf>,
    decoder: D,
    reader: R,
}

impl<'buf, D, R> FramedRead<'buf, D, R> {
    /// Creates a new [`FramedRead`] with the given `decoder` and `reader`.
    #[inline]
    pub fn new(decoder: D, reader: R, buffer: &'buf mut [u8]) -> Self {
        Self {
            state: State::new(buffer),
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

    /// Consumes the [`FramedRead`] and returns the `decoder` and `reader`.
    #[inline]
    pub fn into_parts(self) -> (D, R) {
        (self.decoder, self.reader)
    }

    /// Tries to read a frame from the underlying reader.
    ///
    /// # Return value
    ///
    /// - `Some(Ok(None))` if the buffer is not framable. Call `maybe_next` again to read more bytes.
    /// - `Some(Ok(Some(frame)))` if a frame was successfully decoded. Call `maybe_next` again to read more bytes.
    /// - `Some(Err(error))` if an error occurred. The caller should stop reading.
    /// - `None` if eof was reached. The caller should stop reading.
    ///
    /// # Usage
    ///
    /// See [`next!`](crate::next!).
    ///
    /// # Example
    ///
    /// Convert bytes into [`str`] frames
    ///
    /// ```rust
    /// use core::{error::Error};
    ///
    /// use fraims::{FramedRead, codec::lines::StrLines, mock::Noop, next};  
    ///
    /// async fn read() -> Result<(), Box<dyn Error>> {
    ///     let buf = &mut [0u8; 1024];
    ///
    ///     let mut framed_read = FramedRead::new(StrLines::new(), Noop, buf);
    ///
    ///     while let Some(item) = next!(framed_read).transpose()? {
    ///         println!("Frame: {}", item);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<D::Item>, ReadError<R::Error, D::Error>>>
    where
        D: Decoder<'this>,
        R: Read,
    {
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

            return Some(Ok(None));
        }

        if self.state.is_framable {
            if self.state.eof {
                trace!("Framing on EOF");

                match self
                    .decoder
                    .decode_eof(&mut self.state.buffer[self.state.total_consumed..self.state.index])
                {
                    Ok(Some((item, size))) => {
                        self.state.total_consumed += size;

                        debug!(
                            "Frame decoded, consumed: {}, total_consumed: {}",
                            size, self.state.total_consumed,
                        );

                        return Some(Ok(Some(item)));
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

            match self
                .decoder
                .decode(&mut self.state.buffer[self.state.total_consumed..self.state.index])
            {
                Ok(Some((item, size))) => {
                    self.state.total_consumed += size;

                    debug!(
                        "Frame decoded, consumed: {}, total_consumed: {}",
                        size, self.state.total_consumed,
                    );

                    return Some(Ok(Some(item)));
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

                    return Some(Ok(None));
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

                Some(Err(ReadError::IO(err)))
            }
            Ok(0) => {
                warn!("Got EOF");

                self.state.eof = true;

                self.state.is_framable = true;

                Some(Ok(None))
            }
            Ok(n) => {
                debug!("Bytes read. bytes: {}", n);

                self.state.index += n;

                self.state.is_framable = true;

                Some(Ok(None))
            }
        }
    }

    /// Converts the [`FramedRead`] into a stream of frames using the given `map` function.
    ///
    /// # Example
    ///
    /// Convert bytes into a stream of Strings
    ///
    /// ```rust
    /// use core::{error::Error, pin::pin, str::FromStr};
    ///
    /// use fraims::{FramedRead, codec::lines::StrLines, mock::Noop};  
    /// use futures::StreamExt;
    ///
    /// async fn read() -> Result<(), Box<dyn Error>> {
    ///     let buf = &mut [0u8; 1024];
    ///
    ///     let mut framed_read = FramedRead::new(StrLines::new(), Noop, buf);
    ///
    ///     let stream = framed_read.stream(String::from_str);
    ///     let mut stream = pin!(stream);
    ///
    ///     while let Some(item) = stream.next().await.transpose()?.transpose()? {
    ///         println!("Frame: {}", item);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn stream<U>(
        &mut self,
        map: fn(<D as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<R::Error, D::Error>>> + '_
    where
        D: for<'a> Decoder<'a>,
        R: Read,
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
}
