use embedded_io_async::{Read, Write};
use futures::{Sink, Stream};

use crate::{
    ReadError, WriteError,
    decode::{Decoder, OwnedDecoder},
    encode::Encoder,
    framed_core::FramedCore,
    read::ReadState,
    state::ReadWriteState,
    write::WriteState,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Framed<'buf, C, RW> {
    core: FramedCore<'buf, C, RW, ReadWriteState<'buf>>,
}

impl<'buf, C, RW> Framed<'buf, C, RW> {
    #[inline]
    pub fn new(
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
        &self.core.codec()
    }

    /// Returns mutable reference to the codec.
    #[inline]
    pub fn codec_mut(&mut self) -> &mut C {
        self.core.codec_mut()
    }

    /// Returns reference to the reader/writer.
    #[inline]
    pub const fn inner(&self) -> &RW {
        &self.core.inner()
    }

    /// Returns mutable reference to the reader/writer.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut RW {
        self.core.inner_mut()
    }

    /// Consumes the [`Framed`] and returns the `codec` and `reader/writer`.
    #[inline]
    pub fn into_parts(self) -> (C, RW) {
        self.core.into_parts()
    }

    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'this>,
        RW: Read,
    {
        self.core.maybe_next().await
    }

    pub fn stream_mapped<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<RW::Error, C::Error>>> + '_
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
    {
        self.core.stream_mapped(map)
    }

    pub async fn next(&mut self) -> Option<Result<C::Item, ReadError<RW::Error, C::Error>>>
    where
        C: OwnedDecoder,
        RW: Read,
    {
        self.core.next().await
    }

    pub fn stream(
        &'buf mut self,
    ) -> impl Stream<Item = Result<C::Item, ReadError<RW::Error, C::Error>>> + 'buf
    where
        C: OwnedDecoder,
        RW: Read,
    {
        self.core.stream()
    }

    pub async fn send<I>(&mut self, item: I) -> Result<(), WriteError<RW::Error, C::Error>>
    where
        C: Encoder<I>,
        RW: Write,
    {
        self.core.send(item).await
    }

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

#[cfg(test)]
mod tests {
    use core::{pin::pin, str::FromStr};
    use std::string::String;

    use embedded_io_adapters::tokio_1::FromTokio;
    use futures::{SinkExt, StreamExt};

    use crate::{
        Framed,
        codec::lines::{StrLines, StringLines},
        next,
    };

    #[tokio::test]
    #[ignore = "assert that next! macro works on Framed"]
    async fn assert_next() {
        let (stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        let mut framed = Framed::new(StrLines::new(), FromTokio::new(stream), read_buf, write_buf);

        while let Some(_) = next!(framed) {}

        _ = framed.send("Line").await;
    }

    #[tokio::test]
    #[ignore = "assert that stream_mapped() works on Framed"]
    async fn assert_stream_mapped() {
        let (stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        let mut framed = Framed::new(StrLines::new(), FromTokio::new(stream), read_buf, write_buf);

        let stream = framed.stream_mapped(String::from_str);
        let mut stream = pin!(stream);

        while let Some(_) = stream.next().await {}
    }

    #[tokio::test]
    #[ignore = "assert that stream() works on Framed"]
    async fn assert_stream() {
        let (stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        let mut framed = Framed::new(
            StringLines::<10>::new(),
            FromTokio::new(stream),
            read_buf,
            write_buf,
        );

        let stream = framed.stream();
        let mut stream = pin!(stream);

        while let Some(_) = stream.next().await {}
    }

    #[tokio::test]
    #[ignore = "assert that sink() works on Framed"]
    async fn assert_sink() {
        let (stream, _) = tokio::io::duplex(1024);

        let read_buf = &mut [0u8; 1024];
        let write_buf = &mut [0u8; 1024];

        let mut framed = Framed::new(StrLines::new(), FromTokio::new(stream), read_buf, write_buf);

        let sink = framed.sink();
        let mut sink = pin!(sink);

        _ = sink.send("Line").await;
    }
}
