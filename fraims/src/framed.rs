use embedded_io_async::{Read, Write};
use futures::Stream;

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

    pub async fn maybe_next(
        &'buf mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'buf>,
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
}
