use embedded_io_async::{Read, Write};
use futures::{Sink, Stream};

use crate::{
    ReadError, WriteError, decode::Decoder, encode::Encoder, functions, state::ReadWriteState,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FramedCore<'buf, C, RW> {
    pub codec: C,
    pub inner: RW,
    pub state: ReadWriteState<'buf>,
}

impl<'buf, C, RW> FramedCore<'buf, C, RW> {
    pub const fn new(codec: C, inner: RW, state: ReadWriteState<'buf>) -> Self {
        Self {
            codec,
            inner,
            state,
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
        &self.inner
    }

    /// Returns mutable reference to the reader/writer.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut RW {
        &mut self.inner
    }

    /// Consumes the [`FramedCore`] and returns the `codec` and `reader/writer` and state.
    #[inline]
    pub fn into_parts(self) -> (C, RW, ReadWriteState<'buf>) {
        (self.codec, self.inner, self.state)
    }

    #[inline]
    /// Creates a new [`FramedCore`] from its parts.
    pub const fn from_parts(codec: C, inner: RW, state: ReadWriteState<'buf>) -> Self {
        Self {
            codec,
            inner,
            state,
        }
    }

    /// Returns the number of bytes that can be framed.
    #[inline]
    pub const fn framable(&self) -> usize {
        self.state.read.framable()
    }

    /// See [`Framed::maybe_next`](crate::Framed::maybe_next) for docs.
    pub async fn maybe_next<'this>(
        &'this mut self,
    ) -> Option<Result<Option<C::Item>, ReadError<RW::Error, C::Error>>>
    where
        C: Decoder<'this>,
        RW: Read,
    {
        functions::maybe_next(&mut self.state.read, &mut self.codec, &mut self.inner).await
    }

    /// See [`Framed::next`](crate::Framed::next) for docs.
    pub async fn next<'this, U>(
        &'this mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> Option<Result<U, ReadError<RW::Error, C::Error>>>
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
    {
        functions::next(&mut self.state.read, &mut self.codec, &mut self.inner, map).await
    }

    /// See [`Framed::stream`](crate::Framed::stream) for docs.
    pub fn stream<U>(
        &mut self,
        map: fn(<C as Decoder<'_>>::Item) -> U,
    ) -> impl Stream<Item = Result<U, ReadError<RW::Error, C::Error>>> + '_
    where
        U: 'static,
        C: for<'a> Decoder<'a>,
        RW: Read,
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

    /// See [`Framed::send`](crate::Framed::send) for docs.
    pub async fn send<I>(&mut self, item: I) -> Result<(), WriteError<RW::Error, C::Error>>
    where
        C: Encoder<I>,
        RW: Write,
    {
        functions::send(
            &mut self.state.write,
            &mut self.codec,
            &mut self.inner,
            item,
        )
        .await
    }

    /// See [`Framed::sink`](crate::Framed::sink) for docs.
    pub fn sink<'this, I>(
        &'this mut self,
    ) -> impl Sink<I, Error = WriteError<RW::Error, C::Error>> + 'this
    where
        I: 'this,
        C: Encoder<I>,
        RW: Write,
    {
        futures::sink::unfold(self, |this, item: I| async move {
            this.send(item).await?;

            Ok::<_, WriteError<RW::Error, C::Error>>(this)
        })
    }
}
