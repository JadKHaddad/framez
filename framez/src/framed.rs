use embedded_io_async::{Read, Write};
use futures::{Sink, Stream};

use crate::{
    FramedCore, ReadError, WriteError,
    decode::Decoder,
    encode::Encoder,
    state::{ReadState, ReadWriteState, WriteState},
};

/// A framer that reads bytes from a [`Read`] source and decodes them into frames using a [`Decoder`].
/// And a sink that writes encoded frames into an underlying [`Write`] sink using an [`Encoder`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Framed<'buf, C, RW> {
    /// The core framed implementation.
    ///
    /// This field is made public to be used in the [`functions`](crate::functions) module for library authors.
    /// If you are using this crate as a user, you should probably not care about this field.
    pub core: FramedCore<'buf, C, RW>,
}

impl<'buf, C, RW> Framed<'buf, C, RW> {
    /// Creates a new [`Framed`] with the given `codec` and `reader/writer`.
    #[inline]
    pub const fn new(
        codec: C,
        inner: RW,
        read_buffer: &'buf mut [u8],
        write_buffer: &'buf mut [u8],
    ) -> Self {
        Self {
            core: FramedCore::new(
                codec,
                inner,
                ReadWriteState::new(ReadState::new(read_buffer), WriteState::new(write_buffer)),
            ),
        }
    }

    /// Returns reference to the codec.
    #[inline]
    pub const fn codec(&self) -> &C {
        self.core.codec()
    }

    /// Returns mutable reference to the codec.
    #[inline]
    pub const fn codec_mut(&mut self) -> &mut C {
        self.core.codec_mut()
    }

    /// Returns reference to the reader/writer.
    #[inline]
    pub const fn inner(&self) -> &RW {
        self.core.inner()
    }

    /// Returns mutable reference to the reader/writer.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut RW {
        self.core.inner_mut()
    }

    /// Consumes the [`Framed`] and returns the `codec` and `reader/writer` and state.
    #[inline]
    pub fn into_parts(self) -> (C, RW, ReadWriteState<'buf>) {
        self.core.into_parts()
    }

    #[inline]
    /// Creates a new [`Framed`] from its parts.
    pub const fn from_parts(codec: C, read_write: RW, state: ReadWriteState<'buf>) -> Self {
        Self {
            core: FramedCore::from_parts(codec, read_write, state),
        }
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    pub const fn framable(&self) -> usize {
        self.core.framable()
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
    /// use framez::{Framed, codec::lines::StrLines, mock::Noop, next};  
    ///
    /// async fn read() -> Result<(), Box<dyn Error>> {
    ///     let r_buf = &mut [0u8; 1024];
    ///     let w_buf = &mut [0u8; 1024];
    ///
    ///     let mut framed = Framed::new(StrLines::new(), Noop, r_buf, w_buf);
    ///
    ///     while let Some(item) = next!(framed).transpose()? {
    ///         println!("Frame: {}", item);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'this>,
        RW: Read,
    {
        self.core.maybe_next().await
    }

    /// Converts the [`Framed`] into a stream of frames using the given `map` function.
    ///
    /// # Example
    ///
    /// Convert bytes into a stream of Strings
    ///
    /// ```rust
    /// use core::{error::Error, pin::pin, str::FromStr};
    ///
    /// use framez::{Framed, codec::lines::StrLines, mock::Noop};  
    /// use futures::StreamExt;
    ///
    /// async fn read() -> Result<(), Box<dyn Error>> {
    ///     let r_buf = &mut [0u8; 1024];
    ///     let w_buf = &mut [0u8; 1024];
    ///
    ///     let mut framed = Framed::new(StrLines::new(), Noop, r_buf, w_buf);
    ///
    ///     let stream = framed.stream(String::from_str);
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
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<RW::Error, C::Error>>> + '_
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
    {
        self.core.stream(map)
    }

    /// Tries to read a frame from the underlying reader and converts it using the given `map` function.
    ///
    /// # Return value
    ///
    /// - `Some(Ok(U))` if a frame was successfully decoded and mapped. Call `next` again to read more frames.
    /// - `Some(Err(error))` if an error occurred. The caller should stop reading.
    /// - `None` if eof was reached. The caller should stop reading.
    pub async fn next<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> Option<Result<U, ReadError<RW::Error, C::Error>>>
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
    {
        self.core.next(map).await
    }

    /// Writes a frame to the underlying `writer` and flushes it.
    pub async fn send<I>(&mut self, item: I) -> Result<(), WriteError<RW::Error, C::Error>>
    where
        C: Encoder<I>,
        RW: Write,
    {
        self.core.send(item).await
    }

    /// Converts the [`Framed`] into a sink.
    pub fn sink<'this, I>(
        &'this mut self,
    ) -> impl Sink<I, Error = WriteError<RW::Error, C::Error>> + 'this
    where
        I: 'this,
        C: Encoder<I>,
        RW: Write,
    {
        self.core.sink()
    }
}

/// A framer that reads bytes from a [`Read`] source and decodes them into frames using a [`Decoder`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedRead<'buf, C, R> {
    /// The core framed implementation.
    ///
    /// This field is made public to be used in the [`functions`](crate::functions) module for library authors.
    /// If you are using this crate as a user, you should probably not care about this field.
    pub core: FramedCore<'buf, C, R>,
}

impl<'buf, C, R> FramedRead<'buf, C, R> {
    /// Creates a new [`FramedRead`] with the given `decoder` and `reader`.
    #[inline]
    pub const fn new(codec: C, reader: R, buffer: &'buf mut [u8]) -> Self {
        Self {
            core: FramedCore::new(
                codec,
                reader,
                ReadWriteState::new(ReadState::new(buffer), WriteState::empty()),
            ),
        }
    }

    /// Returns reference to the codec.
    #[inline]
    pub const fn codec(&self) -> &C {
        self.core.codec()
    }

    /// Returns mutable reference to the codec.
    #[inline]
    pub const fn codec_mut(&mut self) -> &mut C {
        self.core.codec_mut()
    }

    /// Returns reference to the reader.
    #[inline]
    pub const fn inner(&self) -> &R {
        self.core.inner()
    }

    /// Returns mutable reference to the reader.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut R {
        self.core.inner_mut()
    }

    /// Consumes the [`FramedRead`] and returns the `codec` and `reader` and state.
    #[inline]
    pub fn into_parts(self) -> (C, R, ReadState<'buf>) {
        let (codec, reader, state) = self.core.into_parts();

        (codec, reader, state.read)
    }

    #[inline]
    /// Creates a new [`FramedRead`] from its parts.
    pub const fn from_parts(codec: C, read: R, state: ReadState<'buf>) -> Self {
        Self {
            core: FramedCore::from_parts(
                codec,
                read,
                ReadWriteState::new(state, WriteState::empty()),
            ),
        }
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    pub const fn framable(&self) -> usize {
        self.core.framable()
    }

    /// See [`Framed::maybe_next`].
    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<R::Error, C::Error>>>
    where
        C: Decoder<'this>,
        R: Read,
    {
        self.core.maybe_next().await
    }

    /// See [`Framed::stream`].
    pub fn stream<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<R::Error, C::Error>>> + '_
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        R: Read,
    {
        self.core.stream(map)
    }

    /// See [`Framed::next`].
    pub async fn next<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> Option<Result<U, ReadError<R::Error, C::Error>>>
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        R: Read,
    {
        self.core.next(map).await
    }
}

/// A sink that writes encoded frames into an underlying [`Write`] sink using an [`Encoder`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedWrite<'buf, C, W> {
    /// The core framed implementation.
    ///
    /// This field is made public to be used in the [`functions`](crate::functions) module for library authors.
    /// If you are using this crate as a user, you should probably not care about this field.
    pub core: FramedCore<'buf, C, W>,
}

impl<'buf, C, W> FramedWrite<'buf, C, W> {
    /// Creates a new [`FramedWrite`] with the given `encoder` and `writer`.
    #[inline]
    pub const fn new(codec: C, writer: W, buffer: &'buf mut [u8]) -> Self {
        Self {
            core: FramedCore::new(
                codec,
                writer,
                ReadWriteState::new(ReadState::empty(), WriteState::new(buffer)),
            ),
        }
    }

    /// Returns reference to the codec.
    #[inline]
    pub const fn codec(&self) -> &C {
        self.core.codec()
    }

    /// Returns mutable reference to the codec.
    #[inline]
    pub const fn codec_mut(&mut self) -> &mut C {
        self.core.codec_mut()
    }

    /// Returns reference to the writer.
    #[inline]
    pub const fn inner(&self) -> &W {
        self.core.inner()
    }

    /// Returns mutable reference to the writer.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut W {
        self.core.inner_mut()
    }

    /// Consumes the [`FramedWrite`] and returns the `codec` and `writer` and state.
    #[inline]
    pub fn into_parts(self) -> (C, W, WriteState<'buf>) {
        let (codec, writer, state) = self.core.into_parts();

        (codec, writer, state.write)
    }

    #[inline]
    /// Creates a new [`FramedWrite`] from its parts.
    pub const fn from_parts(codec: C, write: W, state: WriteState<'buf>) -> Self {
        Self {
            core: FramedCore::from_parts(
                codec,
                write,
                ReadWriteState::new(ReadState::empty(), state),
            ),
        }
    }

    /// See [`Framed::send`].
    pub async fn send<I>(&mut self, item: I) -> Result<(), WriteError<W::Error, C::Error>>
    where
        C: Encoder<I>,
        W: Write,
    {
        self.core.send(item).await
    }

    /// See [`Framed::sink`].
    pub fn sink<'this, I>(
        &'this mut self,
    ) -> impl Sink<I, Error = WriteError<W::Error, C::Error>> + 'this
    where
        I: 'this,
        C: Encoder<I>,
        W: Write,
    {
        self.core.sink()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::redundant_pattern_matching)]
    #![allow(clippy::let_underscore_future)]

    use core::{pin::pin, str::FromStr};
    use std::string::String;

    use embedded_io_adapters::tokio_1::FromTokio;
    use futures::{SinkExt, StreamExt};

    use crate::{Framed, FramedRead, FramedWrite, codec::lines::StrLines, next};

    #[tokio::test]
    #[ignore = "assert that next! macro works on Framed"]
    async fn assert_next() {
        let (mut stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        {
            let mut framed = Framed::new(
                StrLines::new(),
                FromTokio::new(&mut stream),
                read_buf,
                write_buf,
            );

            while let Some(_) = next!(framed) {}

            _ = framed.send("Line").await;
        }

        {
            let mut framed =
                FramedRead::new(StrLines::new(), FromTokio::new(&mut stream), read_buf);

            while let Some(_) = next!(framed) {}
        }
    }

    #[tokio::test]
    #[ignore = "assert that stream() works on Framed"]
    async fn assert_stream() {
        let (mut stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        {
            let mut framed = Framed::new(
                StrLines::new(),
                FromTokio::new(&mut stream),
                read_buf,
                write_buf,
            );

            let stream = framed.stream(String::from_str);
            let mut stream = pin!(stream);

            while let Some(_) = stream.next().await {}
        }

        {
            let mut framed =
                FramedRead::new(StrLines::new(), FromTokio::new(&mut stream), read_buf);

            let stream = framed.stream(String::from_str);
            let mut stream = pin!(stream);

            while let Some(_) = stream.next().await {}
        }
    }

    #[tokio::test]
    #[ignore = "assert that stream() is movable"]
    async fn assert_stream_movable() {
        let (mut stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        {
            let mut framed = Framed::new(
                StrLines::new(),
                FromTokio::new(&mut stream),
                read_buf,
                write_buf,
            );

            let _ = async move {
                // We should be able to move framed and call stream on it.
                let stream = framed.stream(String::from_str);
                let mut stream = pin!(stream);

                while let Some(_) = stream.next().await {}
            };
        }

        {
            let mut framed =
                FramedRead::new(StrLines::new(), FromTokio::new(&mut stream), read_buf);

            let _ = async move {
                // We should be able to move framed and call stream on it.
                let stream = framed.stream(String::from_str);
                let mut stream = pin!(stream);

                while let Some(_) = stream.next().await {}
            };
        }
    }

    #[tokio::test]
    #[ignore = "assert that sink() works on Framed"]
    async fn assert_sink() {
        let (mut stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        {
            let mut framed = Framed::new(
                StrLines::new(),
                FromTokio::new(&mut stream),
                read_buf,
                write_buf,
            );

            let sink = framed.sink();
            let mut sink = pin!(sink);

            _ = sink.send("Line").await;
        }

        {
            let mut framed =
                FramedWrite::new(StrLines::new(), FromTokio::new(&mut stream), write_buf);

            let sink = framed.sink();
            let mut sink = pin!(sink);

            _ = sink.send("Line").await;
        }
    }

    #[tokio::test]
    #[ignore = "assert that sink() is movable"]
    async fn assert_sink_movable() {
        let (mut stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        {
            let mut framed = Framed::new(
                StrLines::new(),
                FromTokio::new(&mut stream),
                read_buf,
                write_buf,
            );

            let _ = async move {
                // We should be able to move framed and call sink on it.
                let sink = framed.sink();
                let mut sink = pin!(sink);

                _ = sink.send("Line").await;
            };
        }

        {
            let mut framed =
                FramedWrite::new(StrLines::new(), FromTokio::new(&mut stream), write_buf);

            let _ = async move {
                // We should be able to move framed and call sink on it.
                let sink = framed.sink();
                let mut sink = pin!(sink);

                _ = sink.send("Line").await;
            };
        }
    }
}
